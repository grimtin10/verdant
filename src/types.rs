use std::{fs, path::{Path, PathBuf}, sync::atomic::{AtomicUsize, Ordering}};

use bytemuck::{Pod, Zeroable};

use crate::{RendererResult, font::Font, rgb, rgba, vec::Vec4};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) usize);

impl WindowId {
    pub(crate) fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

// TODO: windows have quite a few more properties than this
#[derive(Debug, Clone, Default)]
pub struct WindowProperties {
    pub width: u32,
    pub height: u32,

    pub title: String,

    pub resizable: bool,

    pub transparent: bool,
}

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
    pub fn premultiplied(self) -> Self {
        Self {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }

    pub fn to_srgb(self) -> Self {
        Self {
            r: self.r.powf(1. / 2.2),
            g: self.g.powf(1. / 2.2),
            b: self.b.powf(1. / 2.2),
            a: self.a,
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

pub trait ByteSource {
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

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub fill_color: Color,
    pub outline_color: Color,
    pub outline_width: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill_color: Color::WHITE,

            outline_color: Color::default(),
            outline_width: f32::default(),
        }
    }
}

impl Style {
    /// Sets the fill color.
    pub fn fill(&mut self, color: Color) -> &mut Self {
        self.fill_color = color;
        self
    }

    pub fn outline_color(&mut self, color: Color) -> &mut Self {
        self.outline_color = color;
        self
    }

    pub fn outline_width(&mut self, width: f32) -> &mut Self {
        self.outline_width = width;
        self
    }

    /// Sets the outline color and width.
    pub fn outline(&mut self, color: Color, width: f32) -> &mut Self {
        self.outline_color = color;
        self.outline_width = width;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub size: f32,
    pub font: Font,
    pub color: Color,
}

impl TextStyle {
    /// Sets the font size (in pixels).
    pub fn size(&mut self, size_px: f32) -> &mut Self {
        self.size = size_px;
        self
    }

    /// Sets the font.
    pub fn font(&mut self, font: &Font) -> &mut Self {
        self.font = font.clone();
        self
    }

    /// Sets the color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}
