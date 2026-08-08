use std::{cmp::Reverse, collections::HashMap, fs, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}, path::{Path, PathBuf}};

use bytemuck::{Pod, Zeroable};

use crate::{LAYOUT_CACHE_CAPACITY, RendererResult, rgb, rgba, text::TextLayout, vec::Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable, PartialEq)]
pub struct Color {
    /// Red component of the color
    pub r: f32,
    /// Green component of the color
    pub g: f32,
    /// Blue component of the color
    pub b: f32,
    /// Alpha component of the color
    pub a: f32,
}

impl Color {
    /// Returns the color with its RGB channels multiplied by its alpha to be used in contexts that
    /// expect premultiplied color.
    pub fn premultiplied(self) -> Self {
        Self {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }

    /// Converts the color from linear to sRGB color space using a 2.2 gamma.
    // TODO: this is imprecise and should be replaced with the real sRGB formula
    pub fn to_srgb(self) -> Self {
        Self {
            r: self.r.powf(1. / 2.2),
            g: self.g.powf(1. / 2.2),
            b: self.b.powf(1. / 2.2),
            a: self.a,
        }
    }

    /// Returns the color with its RGB channels inverted, preserving the alpha.
    pub fn inverted(self) -> Self {
        Self {
            r: 1. - self.r,
            g: 1. - self.g,
            b: 1. - self.b,
            a: self.a,
        }
    }

    /// Returns the color with all channels inverted, including alpha.
    pub fn fully_inverted(self) -> Self {
        Self {
            r: 1. - self.r,
            g: 1. - self.g,
            b: 1. - self.b,
            a: 1. - self.a,
        }
    }

    /// Sets the red channel of this [`Color`]
    pub fn red(mut self, r: f32) -> Self {
        self.r = r;
        self
    }

    /// Sets the green channel of this [`Color`]
    pub fn green(mut self, g: f32) -> Self {
        self.g = g;
        self
    }

    /// Sets the blue channel of this [`Color`]
    pub fn blue(mut self, b: f32) -> Self {
        self.b = b;
        self
    }

    /// Sets the alpha channel of this [`Color`]
    pub fn alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }
}

impl From<Color> for wgpu::Color {
    fn from(v: Color) -> Self {
        Self { r: v.r as f64, g: v.g as f64, b: v.b as f64, a: v.a as f64 }
    }
}

impl From<[f32; 4]> for Color {
    fn from(v: [f32; 4]) -> Self {
        Self { r: v[0], g: v[1], b: v[2], a: v[3] }
    }
}

impl From<Vec4> for Color {
    fn from(v: Vec4) -> Self {
        Self { r: v.x, g: v.y, b: v.z, a: v.w }
    }
}

impl From<Color> for [f32; 4] {
    fn from(v: Color) -> Self {
        [v.r, v.g, v.b, v.a]
    }
}

impl Add for Color {
    type Output = Color;
    fn add(self, other: Self) -> Self {
        let r = (self.r + other.r).clamp(0., 1.);
        let g = (self.g + other.g).clamp(0., 1.);
        let b = (self.b + other.b).clamp(0., 1.);
        let a = (self.a + other.a).clamp(0., 1.);
        Self { r, g, b, a }
    }
}

impl Sub for Color {
    type Output = Color;
    fn sub(self, other: Self) -> Self {
        let r = (self.r - other.r).clamp(0., 1.);
        let g = (self.g - other.g).clamp(0., 1.);
        let b = (self.b - other.b).clamp(0., 1.);
        let a = (self.a - other.a).clamp(0., 1.);
        Self { r, g, b, a }
    }
}

impl Mul for Color {
    type Output = Color;
    fn mul(self, other: Self) -> Self {
        let r = (self.r * other.r).clamp(0., 1.);
        let g = (self.g * other.g).clamp(0., 1.);
        let b = (self.b * other.b).clamp(0., 1.);
        let a = (self.a * other.a).clamp(0., 1.);
        Self { r, g, b, a }
    }
}

impl Div for Color {
    type Output = Color;
    fn div(self, other: Self) -> Self {
        let r = (self.r / other.r).clamp(0., 1.);
        let g = (self.g / other.g).clamp(0., 1.);
        let b = (self.b / other.b).clamp(0., 1.);
        let a = (self.a / other.a).clamp(0., 1.);
        Self { r, g, b, a }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, other: Self) {
        self.r = (self.r + other.r).clamp(0., 1.);
        self.g = (self.g + other.g).clamp(0., 1.);
        self.b = (self.b + other.b).clamp(0., 1.);
        self.a = (self.a + other.a).clamp(0., 1.);
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, other: Self) {
        self.r = (self.r - other.r).clamp(0., 1.);
        self.g = (self.g - other.g).clamp(0., 1.);
        self.b = (self.b - other.b).clamp(0., 1.);
        self.a = (self.a - other.a).clamp(0., 1.);
    }
}

impl MulAssign for Color {
    fn mul_assign(&mut self, other: Self) {
        self.r = (self.r * other.r).clamp(0., 1.);
        self.g = (self.g * other.g).clamp(0., 1.);
        self.b = (self.b * other.b).clamp(0., 1.);
        self.a = (self.a * other.a).clamp(0., 1.);
    }
}

impl DivAssign for Color {
    fn div_assign(&mut self, other: Self) {
        self.r = (self.r / other.r).clamp(0., 1.);
        self.g = (self.g / other.g).clamp(0., 1.);
        self.b = (self.b / other.b).clamp(0., 1.);
        self.a = (self.a / other.a).clamp(0., 1.);
    }
}

// based on web/CSS conventions, but absolutely does not follow the spec
impl Color {
    pub const TRANSPARENT: Self = rgba(0., 0., 0., 0.);

    // neutrals
    pub const BLACK:       Self = rgb(0.0,  0.0,  0.0 );
    pub const DARK_GRAY:   Self = rgb(0.2,  0.2,  0.2 );
    pub const GRAY:        Self = rgb(0.5,  0.5,  0.5 );
    pub const LIGHT_GRAY:  Self = rgb(0.8,  0.8,  0.8 );
    pub const SILVER:      Self = rgb(0.75, 0.76, 0.78);
    pub const WHITE:       Self = rgb(1.0,  1.0,  1.0 );

    // reds
    pub const RED:         Self = rgb(1.0,  0.0,  0.0 );
    pub const DARK_RED:    Self = rgb(0.55, 0.0,  0.05);
    pub const CRIMSON:     Self = rgb(0.85, 0.1,  0.3 );
    pub const TOMATO:      Self = rgb(1.0,  0.35, 0.2 );
    pub const SALMON:      Self = rgb(0.95, 0.5,  0.4 );

    // pinks & magentas
    pub const PINK:        Self = rgb(1.0,  0.4,  0.6 );
    pub const HOT_PINK:    Self = rgb(1.0,  0.1,  0.5 );
    pub const DEEP_PINK:   Self = rgb(0.9,  0.05, 0.4 );
    pub const MAGENTA:     Self = rgb(1.0,  0.0,  1.0 );

    // oranges & yellows
    pub const ORANGE_RED:  Self = rgb(1.0,  0.25, 0.0 );
    pub const ORANGE:      Self = rgb(1.0,  0.5,  0.0 );
    pub const GOLD:        Self = rgb(1.0,  0.8,  0.1 );
    pub const YELLOW:      Self = rgb(1.0,  1.0,  0.0 );
    pub const KHAKI:       Self = rgb(0.7,  0.6,  0.3 );

    // greens
    pub const GREEN:       Self = rgb(0.0,  1.0,  0.0 );
    pub const LIME:        Self = rgb(0.75, 1.0,  0.0 );
    pub const DARK_GREEN:  Self = rgb(0.0,  0.4,  0.1 );
    pub const FOREST_GREEN:Self = rgb(0.1,  0.4,  0.15);
    pub const OLIVE:       Self = rgb(0.4,  0.5,  0.1 );
    pub const YELLOW_GREEN:Self = rgb(0.6,  0.8,  0.2 );
    pub const SPRING_GREEN:Self = rgb(0.0,  0.9,  0.4 );

    // cyans & teals
    pub const CYAN:        Self = rgb(0.0,  1.0,  1.0 );
    pub const TEAL:        Self = rgb(0.0,  0.4,  0.4 );
    pub const TURQUOISE:   Self = rgb(0.2,  0.8,  0.7 );
    pub const AQUAMARINE:  Self = rgb(0.3,  0.9,  0.8 );

    // blues
    pub const BLUE:        Self = rgb(0.0,  0.0,  1.0 );
    pub const DARK_BLUE:   Self = rgb(0.0,  0.0,  0.5 );
    pub const NAVY:        Self = rgb(0.0,  0.05, 0.3 );
    pub const ROYAL_BLUE:  Self = rgb(0.25, 0.4,  0.9 );
    pub const STEEL_BLUE:  Self = rgb(0.3,  0.5,  0.7 );
    pub const SKY_BLUE:    Self = rgb(0.4,  0.7,  1.0 );
    pub const CORNFLOWER:  Self = rgb(0.4,  0.6,  0.9 );
    pub const DODGER_BLUE: Self = rgb(0.1,  0.6,  1.0 );

    // purples & violets
    pub const PURPLE:      Self = rgb(0.5,  0.2,  0.8 );
    pub const VIOLET:      Self = rgb(0.6,  0.2,  0.9 );
    pub const INDIGO:      Self = rgb(0.2,  0.0,  0.5 );
    pub const LAVENDER:    Self = rgb(0.7,  0.7,  1.0 );
    pub const ORCHID:      Self = rgb(0.7,  0.3,  0.8 );
    pub const PLUM:        Self = rgb(0.5,  0.2,  0.5 );

    // browns
    pub const BROWN:       Self = rgb(0.5,  0.3,  0.1 );
    pub const SIENNA:      Self = rgb(0.6,  0.3,  0.15);
    pub const SADDLE_BROWN:Self = rgb(0.4,  0.2,  0.05);
    pub const TAN:         Self = rgb(0.8,  0.7,  0.5 );
    pub const BEIGE:       Self = rgb(0.9,  0.85, 0.7 );
}

pub trait IntoArray<T, const N: usize> {
    fn into_array(self) -> [T; N];
}

impl<T, U, const N: usize> IntoArray<U, N> for [T; N]
where
    U: From<T>,
{
    fn into_array(self) -> [U; N] {
        self.map(U::from)
    }
}

/// A source of raw bytes, either from a file path or existing byte data.
/// Accepts strings and paths (read from disk) and byte slices, arrays, and vecs (used directly).
/// This means you can pass either a file path or `include_bytes!(...)` anywhere this is accepted.
pub trait ByteSource {
    /// Returns the bytes from this source.
    fn load(self) -> RendererResult<Vec<u8>>;
}

impl ByteSource for &str {
    fn load(self) -> RendererResult<Vec<u8>> { Ok(fs::read(self)?) }
}

impl ByteSource for String {
    fn load(self) -> RendererResult<Vec<u8>> { Ok(fs::read(self)?) }
}

impl ByteSource for &Path {
    fn load(self) -> RendererResult<Vec<u8>> { Ok(fs::read(self)?) }
}

impl ByteSource for PathBuf {
    fn load(self) -> RendererResult<Vec<u8>> { Ok(fs::read(self)?) }
}

impl ByteSource for &[u8] {
    fn load(self) -> RendererResult<Vec<u8>> { Ok(self.to_vec()) }
}

impl<const N: usize> ByteSource for [u8; N] {
    fn load(self) -> RendererResult<Vec<u8>> { Ok(self.to_vec()) }
}

impl<const N: usize> ByteSource for &[u8; N] {
    fn load(self) -> RendererResult<Vec<u8>> { Ok(self.to_vec()) }
}

impl ByteSource for Vec<u8> {
    fn load(self) -> RendererResult<Vec<u8>> { Ok(self) }
}

#[derive(Debug)]
struct CachedLayout {
    layout: TextLayout,
    last_used: u64,
}

#[derive(Debug)]
pub(crate) struct TextLayoutCache {
    map: HashMap<u64, CachedLayout>,
    tick: u64,
    capacity: usize,
}

impl Default for TextLayoutCache {
    fn default() -> Self {
        Self {
            map: HashMap::with_capacity(LAYOUT_CACHE_CAPACITY),
            tick: 0,
            capacity: LAYOUT_CACHE_CAPACITY,
        }
    }
}

impl TextLayoutCache {
    pub fn get(&mut self, key: u64) -> Option<&TextLayout> {
        self.tick += 1;
        self.map.get_mut(&key).map(|cached| {
            cached.last_used = self.tick;
            &cached.layout
        })
    }

    pub fn insert(&mut self, key: u64, layout: TextLayout) -> &TextLayout {
        self.tick += 1;

        if self.map.len() >= self.capacity {
            self.evict_oldest();
        }

        &self.map.entry(key).or_insert(
            CachedLayout {
                layout,
                last_used: self.tick,
            }
        ).layout
    }

    fn evict_oldest(&mut self) {
        let mut items: Vec<(u64, u64)> = self.map
            .iter()
            .map(|(k, v)| (*k, v.last_used))
            .collect();

        items.sort_unstable_by_key(|&(_, tick)| Reverse(tick));

        for (key, _) in items.into_iter().skip(self.capacity / 2) {
            self.map.remove(&key);
        }
    }
}
