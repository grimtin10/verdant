// TODO: font fallback

use std::{collections::HashMap, hash::{Hash, Hasher}, sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard}};

use fontdue::FontSettings;

use crate::{RendererResult, canvas::RenderSurface, errors::Error, image::{Bounds, Image}, shapes::Drawable, transform::Transform2d, types::{ByteSource, Color}, vec::Vec2};

const MAX_ATLAS_SIZE: u32 = 8192;

pub fn rich_text_size(spans: &[Span]) -> RendererResult<(f32, f32)> {
    // because the hash of a `Font` is just the `Arc` pointer, this is fine
    #[allow(clippy::mutable_key_type)]
    let mut fonts = HashMap::new();

    let mut total_width: f32 = 0.;
    let mut total_height = 0.;
    let mut cx = 0.;
    let mut width: f32 = 0.;
    for span in spans {
        let key = (span.style, span.font.clone());
        let glyphs: &mut HashMap<_, _> = fonts.entry(key).or_default();

        let mut height = 0.;
        let mut line_height: f32 = 0.;

        let mut retries = 0;
        'outer: loop {
            if retries > 1 {
                return Err(Error::TextTooBig);
            }

            for char in span.text.chars() {
                let Ok(Some(glyph)) = span.font.get_or_load_glyph(char, span.style.size) else {
                    glyphs.clear();
                    retries += 1;
                    continue 'outer;
                };

                cx += glyph.advance;
                if char == '\n' {
                    height += glyph.height;
                    width = 0.;
                    line_height = 0.;
                    cx = 0.;
                } else {
                    line_height = line_height.max(glyph.height);
                }

                width = width.max(cx);
                total_width = total_width.max(width);

                glyphs.insert(char, glyph);
            }
            break;
        }
        height += line_height;

        total_height += height;
    }

    Ok((total_width, total_height))
}

pub fn rich_text_width(spans: &[Span]) -> RendererResult<f32> {
    Ok(rich_text_size(spans)?.0)
}

pub fn rich_text_height(spans: &[Span]) -> RendererResult<f32> {
    Ok(rich_text_size(spans)?.1)
}

pub fn text_size(text: impl ToString, font: impl AsRef<Font>, size_px: f32) -> RendererResult<(f32, f32)> {
    rich_text_size(&[
        Span {
            text: text.to_string(),
            font: font.as_ref().clone(),
            style: TextStyle {
                size: size_px,
                ..Default::default()
            }
        }
    ])
}

pub fn text_width(text: impl ToString, font: impl AsRef<Font>, size_px: f32) -> RendererResult<f32> {
    Ok(text_size(text, font, size_px)?.0)
}

pub fn text_height(text: impl ToString, font: impl AsRef<Font>, size_px: f32) -> RendererResult<f32> {
    Ok(text_size(text, font, size_px)?.1)
}

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
pub struct CachedGlyph {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub width: f32,
    pub height: f32,
    pub xmin: f32,
    pub ymin: f32,
    pub advance: f32,
}

#[derive(Debug, Default)]
struct FontPosition {
    current_x: u32,
    current_y: u32,
    row_height: u32,
}

#[derive(Debug)]
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

        if key.0 == '\n' && let Some(metrics) = self.font.horizontal_line_metrics(key.1) {
            let newline_glyph = CachedGlyph {
                uv_min: Vec2::ZERO,
                uv_max: Vec2::ZERO,
                width: 0.,
                height: metrics.new_line_size,
                xmin: 0.,
                ymin: 0.,
                advance: 0.,
            };
            self.cache.write()?.insert(key, newline_glyph);
            return Ok(Some(newline_glyph));
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

#[derive(Debug, Clone)]
pub struct Font {
    inner: Arc<FontInner>,
}

impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for Font {}

impl Hash for Font {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.inner), state);
    }
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

    pub fn get_or_load_glyph(&self, char: char, size_px: f32) -> RendererResult<Option<CachedGlyph>> {
        let key = GlyphInfo(char, size_px);

        self.inner.get_or_load_glyph(key)
    }

    pub fn line_distance(&self, size_px: f32) -> f32 {
        if let Some(metrics) = self.inner.font.horizontal_line_metrics(size_px) {
            metrics.new_line_size
        } else {
            size_px
        }
    }
}

impl AsRef<Font> for Font {
    fn as_ref(&self) -> &Font {
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum HorizontalAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum VerticalAlign {
    #[default]
    Bottom,
    Center,
    Top,
}

#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub size: f32,
    pub color: Color,
    pub offset: Vec2,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.,
            color: Color::WHITE,
            offset: Vec2::default(),
        }
    }
}

impl Hash for TextStyle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.to_bits().hash(state);
    }
}

impl PartialEq for TextStyle {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}

impl Eq for TextStyle {}

impl TextStyle {
    /// Sets the font size (in pixels).
    pub fn size(&mut self, size_px: f32) -> &mut Self {
        self.size = size_px;
        self
    }

    /// Sets the color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the offset.
    pub fn offset(&mut self, offset: Vec2) -> &mut Self {
        self.offset = offset;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub font: Font,
    pub style: TextStyle,
}

impl Span {
    /// Sets the text displayed by this [`Span`].
    pub fn text(&mut self, text: impl ToString) -> &mut Self {
        self.text = text.to_string();
        self
    }

    /// Sets the font of this [`Span`].
    pub fn font(&mut self, font: &Font) -> &mut Self {
        self.font = font.clone();
        self
    }

    /// Sets the font size (in pixels) of this [`Span`].
    pub fn size(&mut self, size_px: f32) -> &mut Self {
        self.style.size = size_px;
        self
    }

    /// Sets the color of this [`Span`].
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.style.color = color;
        self
    }

    /// Sets the offset of this [`Span`].
    pub fn offset(&mut self, offset: Vec2) -> &mut Self {
        self.style.offset = offset;
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextAlignment {
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub line_align: HorizontalAlign,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub position: Vec2,
    pub style: TextStyle,
    pub transform: Transform2d,

    pub align: TextAlignment,

    pub font: Font,
    pub text: String,
}

impl Text {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: Vec2,
        style: TextStyle,
        transform: Transform2d,

        align: TextAlignment,

        font: impl AsRef<Font>,
        text: impl ToString,
    ) -> Self {
        Self {
            position,
            style,
            transform,

            align,

            font: font.as_ref().clone(),
            text: text.to_string(),
        }
    }

    pub fn with_font(font: impl AsRef<Font>) -> Self {
        Self {
            position: Vec2::default(),
            style: TextStyle::default(),
            transform: Transform2d::identity(),

            align: TextAlignment::default(),

            font: font.as_ref().clone(),
            text: String::default(),
        }
    }

    pub fn align(&mut self, align: TextAlignment) -> &mut Self {
        self.align = align;
        self
    }

    pub fn horizontal_align(&mut self, align: HorizontalAlign) -> &mut Self {
        self.align.horizontal_align = align;
        self
    }

    pub fn vertical_align(&mut self, align: VerticalAlign) -> &mut Self {
        self.align.vertical_align = align;
        self
    }

    pub fn line_align(&mut self, align: HorizontalAlign) -> &mut Self {
        self.align.line_align = align;
        self
    }

    /// Sets the position of this [`Text`].
    pub fn position(&mut self, x: f32, y: f32) -> &mut Self {
        self.position = Vec2 { x, y };
        self
    }

    /// Sets the font of this [`Text`].
    pub fn font(&mut self, font: impl AsRef<Font>) -> &mut Self {
        self.font = font.as_ref().clone();
        self
    }

    /// Sets the font size (in pixels) of this [`Text`].
    pub fn size(&mut self, size_px: f32) -> &mut Self {
        self.style.size = size_px;
        self
    }

    /// Sets the color of this [`Text`].
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.style.color = color;
        self
    }

    /// Sets the style of this [`Text`].
    pub fn style(&mut self, style: TextStyle) -> &mut Self {
        self.style = style;
        self
    }

    /// Sets the text of this [`Text`].
    pub fn text(&mut self, text: impl ToString) -> &mut Self {
        self.text = text.to_string();
        self
    }
}

impl Drawable for Text {
    fn draw(&self, window: &mut impl RenderSurface) {
        self.draw_at(window, self.position.x, self.position.y);
    }

    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32) {
        window.with_style(|window| {
            window.with_transform(
                self.transform
                    .then(Transform2d::translation(x, y)),
                |window| {
                    window.fill(self.style.color);
                    window.text_size(self.style.size);
                    window.text(&self.font, 0., 0., &self.text);
                }
            );
        });
    }
}

#[derive(Debug, Clone)]
pub struct RichText {
    pub position: Vec2,
    pub transform: Transform2d,
    pub spans: Vec<Span>,

    pub align: TextAlignment,
}

impl RichText {
    pub fn new(
        position: Vec2,
        transform: Transform2d,
        spans: Vec<Span>,

        align: TextAlignment,
    ) -> Self {
        Self {
            position,
            transform,
            spans,

            align,
        }
    }

    /// Creates a new [`RichText`] with the given [`Span`]s.
    pub fn with_spans(spans: Vec<Span>) -> Self {
        Self {
            position: Vec2::default(),
            transform: Transform2d::identity(),
            spans,

            align: TextAlignment::default(),
        }
    }

    pub fn align(&mut self, align: TextAlignment) -> &mut Self {
        self.align = align;
        self
    }

    pub fn horizontal_align(&mut self, align: HorizontalAlign) -> &mut Self {
        self.align.horizontal_align = align;
        self
    }

    pub fn vertical_align(&mut self, align: VerticalAlign) -> &mut Self {
        self.align.vertical_align = align;
        self
    }

    pub fn line_align(&mut self, align: HorizontalAlign) -> &mut Self {
        self.align.line_align = align;
        self
    }

    /// Sets the position of this [`RichText`].
    pub fn position(&mut self, x: f32, y: f32) -> &mut Self {
        self.position = Vec2 { x, y };
        self
    }
}

impl Drawable for RichText {
    fn draw(&self, window: &mut impl RenderSurface) {
        self.draw_at(window, self.position.x, self.position.y);
    }

    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32) {
        window.with_style(|window| {
            window.with_transform(
                self.transform
                    .then(Transform2d::translation(x, y)),
                |window| {
                    window.rich_text(0., 0., &self.spans);
                }
            );
        });
    }
}
