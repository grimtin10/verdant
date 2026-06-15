use std::{cmp::Reverse, collections::HashMap, fs, path::{Path, PathBuf}};

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

// based on web/CSS conventions
impl Color {
    pub const TRANSPARENT: Self = rgba(0., 0., 0., 0.);

    // Neutrals
    pub const BLACK:       Self = rgb(0.,    0.,    0.   );
    pub const WHITE:       Self = rgb(1.,    1.,    1.   );
    pub const DARK_GRAY:   Self = rgb(0.25,  0.25,  0.25 );
    pub const GRAY:        Self = rgb(0.5,   0.5,   0.5  );
    pub const LIGHT_GRAY:  Self = rgb(0.75,  0.75,  0.75 );
    pub const SILVER:      Self = rgb(0.753, 0.753, 0.753);

    // Reds
    pub const RED:         Self = rgb(1.,    0.,    0.   );
    pub const DARK_RED:    Self = rgb(0.545, 0.,    0.   );
    pub const CRIMSON:     Self = rgb(0.863, 0.078, 0.235);
    pub const TOMATO:      Self = rgb(1.,    0.388, 0.278);
    pub const SALMON:      Self = rgb(0.980, 0.502, 0.447);

    // Pinks & Magentas
    pub const PINK:        Self = rgb(1.,    0.753, 0.796);
    pub const HOT_PINK:    Self = rgb(1.,    0.412, 0.706);
    pub const DEEP_PINK:   Self = rgb(1.,    0.078, 0.576);
    pub const MAGENTA:     Self = rgb(1.,    0.,    1.   );

    // Oranges
    pub const ORANGE_RED:  Self = rgb(1.,    0.271, 0.   );
    pub const ORANGE:      Self = rgb(1.,    0.647, 0.   );
    pub const GOLD:        Self = rgb(1.,    0.843, 0.   );

    // Yellows
    pub const YELLOW:      Self = rgb(1.,    1.,    0.   );
    pub const KHAKI:       Self = rgb(0.941, 0.902, 0.549);

    // Greens
    pub const LIME:        Self = rgb(0.,    1.,    0.   );
    pub const GREEN:       Self = rgb(0.,    0.502, 0.   );
    pub const DARK_GREEN:  Self = rgb(0.,    0.392, 0.   );
    pub const FOREST_GREEN:Self = rgb(0.133, 0.545, 0.133);
    pub const OLIVE:       Self = rgb(0.502, 0.502, 0.   );
    pub const YELLOW_GREEN:Self = rgb(0.604, 0.804, 0.196);
    pub const SPRING_GREEN:Self = rgb(0.,    1.,    0.498);

    // Cyans & Teals
    pub const CYAN:        Self = rgb(0.,    1.,    1.   );
    pub const TEAL:        Self = rgb(0.,    0.502, 0.502);
    pub const TURQUOISE:   Self = rgb(0.251, 0.878, 0.816);
    pub const AQUAMARINE:  Self = rgb(0.498, 1.,    0.831);

    // Blues
    pub const BLUE:        Self = rgb(0.,    0.,    1.   );
    pub const DARK_BLUE:   Self = rgb(0.,    0.,    0.545);
    pub const NAVY:        Self = rgb(0.,    0.,    0.502);
    pub const ROYAL_BLUE:  Self = rgb(0.255, 0.412, 0.882);
    pub const STEEL_BLUE:  Self = rgb(0.275, 0.510, 0.706);
    pub const SKY_BLUE:    Self = rgb(0.529, 0.808, 0.922);
    pub const CORNFLOWER:  Self = rgb(0.392, 0.584, 0.929);
    pub const DODGER_BLUE: Self = rgb(0.118, 0.565, 1.   );

    // Purples & Violets
    pub const PURPLE:      Self = rgb(0.502, 0.,    0.502);
    pub const VIOLET:      Self = rgb(0.933, 0.510, 0.933);
    pub const INDIGO:      Self = rgb(0.294, 0.,    0.510);
    pub const LAVENDER:    Self = rgb(0.902, 0.902, 0.980);
    pub const ORCHID:      Self = rgb(0.855, 0.439, 0.839);
    pub const PLUM:        Self = rgb(0.867, 0.627, 0.867);

    // Browns
    pub const BROWN:       Self = rgb(0.647, 0.165, 0.165);
    pub const SIENNA:      Self = rgb(0.627, 0.322, 0.176);
    pub const SADDLE_BROWN:Self = rgb(0.545, 0.271, 0.075);
    pub const TAN:         Self = rgb(0.824, 0.706, 0.549);
    pub const BEIGE:       Self = rgb(0.961, 0.961, 0.863);
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
