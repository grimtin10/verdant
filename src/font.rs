// TODO: font fallback

use std::{collections::HashMap, hash::{Hash, Hasher}, sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard}};

use fontdue::FontSettings;

use crate::{RendererResult, errors::Error, image::{Bounds, Image}, types::ByteSource, vec::Vec2};

const MAX_ATLAS_SIZE: u32 = 8192;

#[derive(Debug, Clone, Copy)]
struct GlyphInfo(char, f32);

impl PartialEq for GlyphInfo {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for GlyphInfo {}

impl Hash for GlyphInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.to_bits().hash(state);
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CachedGlyph {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub width: f32,
    pub height: f32,
    pub xmin: f32,
    pub ymin: f32,
    pub advance: f32,
}

#[derive(Default)]
struct FontPosition {
    current_x: u32,
    current_y: u32,
    row_height: u32,
}

struct FontInner {
    pub atlas: RwLock<Image>,

    font: fontdue::Font,

    position: Mutex<FontPosition>,
    padding: u32,

    cache: RwLock<HashMap<GlyphInfo, CachedGlyph>>,
}

impl FontInner {
    fn resize_atlas(&self) -> RendererResult<()> {
        let width = self.atlas.read()?.width;
        let old_height = self.atlas.read()?.height;

        let new_height = old_height * 2;

        let mut old_data = vec![0; (width * old_height * 4) as usize];
        self.atlas.read()?.read_rect(
            Bounds::new(0, 0, width, old_height),
            &mut old_data,
        )?;

        let mut new_atlas = Image::new_empty(width, new_height);
        new_atlas.blit(0, 0, width, old_height, &old_data)?;

        let height_ratio = old_height as f32 / new_height as f32;
        for glyph in self.cache.write()?.values_mut() {
            glyph.uv_min.y *= height_ratio;
            glyph.uv_max.y *= height_ratio;
        }

        *self.atlas.write()? = new_atlas;

        Ok(())
    }

    fn get_or_load_glyph(&self, key: GlyphInfo) -> RendererResult<Option<CachedGlyph>> {
        if let Some(glyph) = self.cache.read()?.get(&key) {
            return Ok(Some(*glyph));
        }

        // TODO: technically we could decide to not hold this lock for the whole function to
        //       allow for two glyphs to be rasterized at the same time, but for now, i'm lazy
        let mut position = self.position.lock()?;

        let (metrics, bitmap) = self.font.rasterize(key.0, key.1);

        let width = metrics.width as u32;
        let height = metrics.height as u32;

        if width == 0 || height == 0 {
            let empty_glyph = CachedGlyph {
                uv_min: Vec2::ZERO,
                uv_max: Vec2::ZERO,
                width: 0.,
                height: 0.,
                xmin: 0.,
                ymin: 0.,
                advance: metrics.advance_width,
            };
            self.cache.write()?.insert(key, empty_glyph);
            return Ok(Some(empty_glyph));
        }

        if position.current_x + width + self.padding > self.atlas.read()?.width {
            position.current_x = 0;
            position.current_y += position.row_height + self.padding;
            position.row_height = 0;
        }

        while position.current_y + height + self.padding > self.atlas.read()?.height {
            if self.atlas.read()?.height >= MAX_ATLAS_SIZE {
                *self.atlas.write()? = Image::new_empty(1024, 1024);
                self.cache.write()?.clear();
                position.current_x = 0;
                position.current_y = 0;
                position.row_height = 0;
                return Ok(None);
            }
            self.resize_atlas()?;
        }

        let rgba: Vec<u8> = bitmap.iter()
            .flat_map(|&a| [255, 255, 255, a])
            .collect();

        self.atlas.write()?.blit(position.current_x, position.current_y, width, height, &rgba)?;

        let u0 = position.current_x as f32 / self.atlas.read()?.width as f32;
        let v0 = position.current_y as f32 / self.atlas.read()?.height as f32;
        let u1 = (position.current_x + width) as f32 / self.atlas.read()?.width as f32;
        let v1 = (position.current_y + height) as f32 / self.atlas.read()?.height as f32;

        let cached = CachedGlyph {
            uv_min: Vec2::new(u0, v0),
            uv_max: Vec2::new(u1, v1),
            width: width as f32,
            height: height as f32,
            xmin: metrics.xmin as f32,
            ymin: metrics.ymin as f32,
            advance: metrics.advance_width,
        };

        position.current_x += width + self.padding;
        position.row_height = position.row_height.max(height);

        self.cache.write()?.insert(key, cached);
        Ok(Some(cached))
    }
}

#[derive(Clone)]
pub struct Font {
    inner: Arc<FontInner>,
}

impl Font {
    /// Loads a font from the given source (byte array or path to font file).
    pub fn load(source: impl ByteSource) -> RendererResult<Self> {
        let data = source.load()?;
        let settings = FontSettings::default();
        let font = fontdue::Font::from_bytes(data, settings).map_err(Error::FontError)?;

        let atlas_width = 1024;
        let atlas_height = 1024;

        let atlas = Image::new_empty(atlas_width, atlas_height);

        Ok(Self {
            inner: Arc::new(FontInner {
                font,

                atlas: RwLock::new(atlas),

                position: Mutex::new(FontPosition::default()),
                padding: 2,
                cache: RwLock::new(HashMap::new()),
            }),
        })
    }

    // TODO: i'm not a fan of using `.expect` here but it's a sacrifice
    //       i'm willing to make to make the API nice and consistent
    pub fn atlas(&self) -> RwLockReadGuard<'_, Image> {
        self.inner.atlas.read().expect("text atlas lock is poisoned")
    }

    pub fn atlas_mut(&self) -> RwLockWriteGuard<'_, Image> {
        self.inner.atlas.write().expect("text atlas lock is poisoned")
    }

    pub(crate) fn get_or_load_glyph(&self, char: char, size_px: f32) -> RendererResult<Option<CachedGlyph>> {
        let key = GlyphInfo(char, size_px);

        self.inner.get_or_load_glyph(key)
    }
}
