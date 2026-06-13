use std::{collections::HashMap, hash::{Hash, Hasher}, sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard}};

use fontdue::{FontSettings, Metrics};

use crate::{RendererResult, canvas::RenderSurface, errors::Error, image::{Bounds, Image}, shapes::Drawable, transform::Transform2d, types::{ByteSource, Color}, vec::Vec2};

const MAX_ATLAS_SIZE: u32 = 8192;
const PADDING: u32 = 2;

// TODO: refactor this to be less big and ugly
/// Get the width and height of a set of [`Span`]s.
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

/// Get the width of a set of [`Span`]s.
pub fn rich_text_width(spans: &[Span]) -> RendererResult<f32> {
    Ok(rich_text_size(spans)?.0)
}

/// Get the height of a set of [`Span`]s.
pub fn rich_text_height(spans: &[Span]) -> RendererResult<f32> {
    Ok(rich_text_size(spans)?.1)
}

/// Get the width and height of text as rendered by the given [`Font`] at the given size.
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

/// Get the width of text as rendered by the given [`Font`] at the given size.
pub fn text_width(text: impl ToString, font: impl AsRef<Font>, size_px: f32) -> RendererResult<f32> {
    Ok(text_size(text, font, size_px)?.0)
}

/// Get the height of text as rendered by the given [`Font`] at the given size.
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

    fallback: RwLock<Vec<Font>>,

    position: Mutex<FontPosition>,
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

    fn rasterize(&self, glyph: GlyphInfo) -> RendererResult<(Metrics, Vec<u8>)> {
        if !self.font.has_glyph(glyph.0) {
            for font in self.fallback.read()?.iter() {
                if font.has_glyph(glyph.0) {
                    return font.inner.rasterize(glyph);
                }
            }
        }

        Ok(self.font.rasterize(glyph.0, glyph.1))
    }

    fn get_or_load_glyph(&self, glyph: GlyphInfo) -> RendererResult<Option<CachedGlyph>> {
        if let Some(glyph) = self.cache.read()?.get(&glyph) {
            return Ok(Some(*glyph));
        }

        if glyph.0 == '\n' && let Some(metrics) = self.font.horizontal_line_metrics(glyph.1) {
            let newline_glyph = CachedGlyph {
                uv_min: Vec2::ZERO,
                uv_max: Vec2::ZERO,
                width: 0.,
                height: metrics.new_line_size,
                xmin: 0.,
                ymin: 0.,
                advance: 0.,
            };
            self.cache.write()?.insert(glyph, newline_glyph);
            return Ok(Some(newline_glyph));
        }

        let (metrics, bitmap) = self.rasterize(glyph)?;

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
            self.cache.write()?.insert(glyph, empty_glyph);
            return Ok(Some(empty_glyph));
        }

        let mut position = self.position.lock()?;

        if position.current_x + width + PADDING > self.atlas.read()?.width {
            position.current_x = 0;
            position.current_y += position.row_height + PADDING;
            position.row_height = 0;
        }

        while position.current_y + height + PADDING > self.atlas.read()?.height {
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

        position.current_x += width + PADDING;
        position.row_height = position.row_height.max(height);

        self.cache.write()?.insert(glyph, cached);
        Ok(Some(cached))
    }

    fn fallbacks(&self) -> RwLockReadGuard<'_, Vec<Font>> {
        match self.fallback.read() {
            Ok(guard) => guard,
            Err(err) => err.into_inner(),
        }
    }

    fn fallbacks_mut(&self) -> RwLockWriteGuard<'_, Vec<Font>> {
        match self.fallback.write() {
            Ok(guard) => guard,
            Err(err) => err.into_inner(),
        }
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
                atlas: RwLock::new(atlas),

                font,

                fallback: RwLock::default(),

                position: Mutex::default(),
                cache: RwLock::default(),
            }),
        })
    }

    /// Returns a read guard for the texture atlas, locking the atlas until the guard is dropped.
    pub fn atlas(&self) -> RwLockReadGuard<'_, Image> {
        match self.inner.atlas.read() {
            Ok(guard) => guard,
            Err(err) => err.into_inner(),
        }
    }

    /// Returns a write guard for the texture atlas, locking the atlas until the guard is dropped.
    pub fn atlas_mut(&self) -> RwLockWriteGuard<'_, Image> {
        match self.inner.atlas.write() {
            Ok(guard) => guard,
            Err(err) => err.into_inner(),
        }
    }

    /// Returns a [`CachedGlyph`] for the `char` at `size_px`, rasterizing it if it hasn't been loaded yet.
    /// Returns `None` if the atlas overflowed and was cleared; re-request all glyphs from scratch.
    pub fn get_or_load_glyph(&self, char: char, size_px: f32) -> RendererResult<Option<CachedGlyph>> {
        let key = GlyphInfo(char, size_px);

        self.inner.get_or_load_glyph(key)
    }

    /// Returns the line height for text at `size_px`.
    /// Falls back to `size_px` if the font doesn't provide horizontal line metrics.
    pub fn line_spacing(&self, size_px: f32) -> f32 {
        if let Some(metrics) = self.inner.font.horizontal_line_metrics(size_px) {
            metrics.new_line_size
        } else {
            size_px
        }
    }

    /// Returns whether or not this [`Font`] has this glyph or not.
    /// Includes fallback fonts.
    pub fn has_glyph(&self, glyph: char) -> bool {
        if !self.inner.font.has_glyph(glyph) {
            for font in self.inner.fallbacks().iter() {
                if font.has_glyph(glyph) {
                    return true;
                }
            }

            false
        } else { true }
    }

    /// Adds a fallback font to this [`Font`].
    /// Used whenever the main font doesn't support a given glyph.
    /// If no fallback fonts support the glyph, the main font's `notdef` glyph will be displayed instead.
    // TODO: if someone adds cycling fallbacks it will probably explode
    pub fn add_fallback(&self, font: impl AsRef<Font>) {
        self.inner.fallbacks_mut().push(font.as_ref().clone());
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
    /// Sets the font size (in pixels) of this [`TextStyle`].
    pub fn size(&mut self, size_px: f32) -> &mut Self {
        self.size = size_px;
        self
    }

    /// Sets the color of this [`TextStyle`].
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Sets the offset of this [`TextStyle`].
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

#[derive(Debug, Clone, Copy, Default)]
pub struct TextAlignment {
    pub horizontal: HorizontalAlign,
    pub vertical: VerticalAlign,
    pub line: HorizontalAlign,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub position: Vec2,
    pub style: TextStyle,
    pub transform: Transform2d,

    pub align: TextAlignment,

    pub font: Font,
    pub text: Arc<String>,
}

impl Text {
    #[allow(clippy::too_many_arguments)]
    /// Creates a fully specified [`Text`] with position, style, transform, alignment, font, and text.
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
            text: text.to_string().into(),
        }
    }

    /// Creates a new [`Text`] with the given font.
    /// Everything else is set to defaults.
    pub fn with_font(font: impl AsRef<Font>) -> Self {
        Self {
            position: Vec2::default(),
            style: TextStyle::default(),
            transform: Transform2d::identity(),

            align: TextAlignment::default(),

            font: font.as_ref().clone(),
            text: Arc::default(),
        }
    }

    /// Sets the alignment of this [`Text`].
    pub fn align(&mut self, align: TextAlignment) -> Self {
        self.align = align;
        self.clone() // this is fine, because everything is either `Arc` or implements `Copy`
    }

    /// Sets the horizontal alignment of this [`Text`].
    pub fn horizontal_align(&mut self, align: HorizontalAlign) -> Self {
        self.align.horizontal = align;
        self.clone()
    }

    /// Sets the vertical alignment of this [`Text`].
    pub fn vertical_align(&mut self, align: VerticalAlign) -> Self {
        self.align.vertical = align;
        self.clone()
    }

    /// Sets the line alignment of this [`Text`].
    pub fn line_align(&mut self, align: HorizontalAlign) -> Self {
        self.align.line = align;
        self.clone()
    }

    /// Sets the font of this [`Text`].
    pub fn font(&mut self, font: impl AsRef<Font>) -> Self {
        self.font = font.as_ref().clone();
        self.clone()
    }

    /// Sets the font size (in pixels) of this [`Text`].
    pub fn size(&mut self, size_px: f32) -> Self {
        self.style.size = size_px;
        self.clone()
    }

    /// Sets the color of this [`Text`].
    pub fn color(&mut self, color: Color) -> Self {
        self.style.color = color;
        self.clone()
    }

    /// Sets the style of this [`Text`].
    pub fn style(&mut self, style: TextStyle) -> Self {
        self.style = style;
        self.clone()
    }

    /// Sets the text of this [`Text`].
    pub fn text(&mut self, text: impl ToString) -> Self {
        self.text = text.to_string().into();
        self.clone()
    }

    /// Mutates the underlying [`String`] of this [`Text`] in-place.
    pub fn mutate(&mut self, f: impl FnOnce(&mut String)) -> Self {
        f(Arc::make_mut(&mut self.text));
        self.clone()
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
                    window.text_align(self.align.horizontal, self.align.vertical);
                    window.line_align(self.align.line);
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
    pub spans: Arc<Vec<Span>>,
    pub align: TextAlignment,
}

impl RichText {
    /// Creates a fully specified [`RichText`] with position, transform, spans, and alignment.
    pub fn new(
        position: Vec2,
        transform: Transform2d,
        spans: Vec<Span>,
        align: TextAlignment,
    ) -> Self {
        Self {
            position,
            transform,
            spans: spans.into(),
            align,
        }
    }

    /// Creates a new [`RichText`] with the given [`Span`]s.
    pub fn with_spans(spans: Vec<Span>) -> Self {
        Self {
            position: Vec2::default(),
            transform: Transform2d::identity(),
            spans: spans.into(),

            align: TextAlignment::default(),
        }
    }

    /// Sets the alignment of this [`RichText`].
    pub fn align(&mut self, align: TextAlignment) -> Self {
        self.align = align;
        self.clone() // this is fine, because everything is either `Arc` or implements `Copy`
    }

    /// Sets the horizontal alignment of this [`RichText`].
    pub fn horizontal_align(&mut self, align: HorizontalAlign) -> Self {
        self.align.horizontal = align;
        self.clone()
    }

    /// Sets the vertical alignment of this [`RichText`].
    pub fn vertical_align(&mut self, align: VerticalAlign) -> Self {
        self.align.vertical = align;
        self.clone()
    }

    /// Sets the line alignment of this [`RichText`].
    pub fn line_align(&mut self, align: HorizontalAlign) -> Self {
        self.align.line = align;
        self.clone()
    }

    /// Pushes a new [`Span`] to this [`RichText`].
    pub fn push_span(&mut self, span: Span) -> Self {
        Arc::make_mut(&mut self.spans).push(span);
        self.clone()
    }

    /// Modifies the underlying [`Span`]s of this [`RichText`].
    pub fn modify(&mut self, f: impl FnOnce(&mut Vec<Span>)) -> Self {
        f(Arc::make_mut(&mut self.spans));
        self.clone()
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
                    window.text_align(self.align.horizontal, self.align.vertical);
                    window.line_align(self.align.line);
                    window.rich_text(0., 0., &self.spans);
                }
            );
        });
    }
}
