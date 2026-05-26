// TODO: font fallback

use std::{collections::HashMap, hash::{Hash, Hasher}};

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

pub struct Font {
    pub atlas: Image,

    font: fontdue::Font,

    current_x: u32,
    current_y: u32,
    row_height: u32,
    padding: u32,

    cache: HashMap<GlyphInfo, CachedGlyph>,
}

impl Font {
    pub fn load(source: impl ByteSource) -> RendererResult<Self> {
        let data = source.load()?;
        let settings = FontSettings::default();
        let font = fontdue::Font::from_bytes(data, settings).map_err(Error::FontError)?;

        let atlas_width = 1024;
        let atlas_height = 1024;

        let atlas = Image::new_empty(atlas_width, atlas_height);

        Ok(Self {
            font,

            atlas,

            current_x: 0,
            current_y: 0,
            row_height: 0 ,
            padding: 2,
            cache: HashMap::new(),
        })
    }

    fn resize_atlas(&mut self) -> RendererResult<()> {
        let width = self.atlas.width;
        let old_height = self.atlas.height;

        let new_height = old_height * 2;

        let mut old_data = vec![0; (width * old_height * 4) as usize];
        self.atlas.read_rect(
            Bounds::new(0, 0, width, old_height),
            &mut old_data,
        )?;

        let mut new_atlas = Image::new_empty(width, new_height);
        new_atlas.blit(0, 0, width, old_height, &old_data)?;

        let height_ratio = old_height as f32 / new_height as f32;
        for glyph in self.cache.values_mut() {
            glyph.uv_min.y *= height_ratio;
            glyph.uv_max.y *= height_ratio;
        }

        self.atlas = new_atlas;

        Ok(())
    }

    pub(crate) fn get_or_load_glyph(&mut self, char: char, size_px: f32) -> RendererResult<Option<CachedGlyph>> {
        let key = GlyphInfo(char, size_px);

        if let Some(glyph) = self.cache.get(&key) {
            return Ok(Some(*glyph));
        }

        let (metrics, bitmap) = self.font.rasterize(char, size_px);

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
            self.cache.insert(key, empty_glyph);
            return Ok(Some(empty_glyph));
        }

        if self.current_x + width + self.padding > self.atlas.width {
            self.current_x = 0;
            self.current_y += self.row_height + self.padding;
            self.row_height = 0;
        }

        while self.current_y + height + self.padding > self.atlas.height {
            if self.atlas.height >= MAX_ATLAS_SIZE {
                self.atlas = Image::new_empty(1024, 1024);
                self.cache.clear();
                self.current_x = 0;
                self.current_y = 0;
                self.row_height = 0;
                return Ok(None);
            }
            self.resize_atlas()?;
        }

        let rgba: Vec<u8> = bitmap.iter()
            .flat_map(|&a| [255, 255, 255, a])
            .collect();

        self.atlas.blit(self.current_x, self.current_y, width, height, &rgba)?;

        let u0 = self.current_x as f32 / self.atlas.width as f32;
        let v0 = self.current_y as f32 / self.atlas.height as f32;
        let u1 = (self.current_x + width) as f32 / self.atlas.width as f32;
        let v1 = (self.current_y + height) as f32 / self.atlas.height as f32;

        let cached = CachedGlyph {
            uv_min: Vec2::new(u0, v0),
            uv_max: Vec2::new(u1, v1),
            width: width as f32,
            height: height as f32,
            xmin: metrics.xmin as f32,
            ymin: metrics.ymin as f32,
            advance: metrics.advance_width,
        };

        self.current_x += width + self.padding;
        self.row_height = self.row_height.max(height);

        self.cache.insert(key, cached);
        Ok(Some(cached))
    }
}
