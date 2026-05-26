// TODO: transforms on shapes
// TODO: image
// TODO: text

use crate::{KIND_ELLIPSE, KIND_LINE, KIND_RECT, KIND_TEXTURED, Vertex, types::Color, vec::Vec2, window::Window};

/// Trait for types that can draw themselves onto a [`Window`].
pub trait Drawable {
    /// Draws this shape onto the given window using its position and style.
    fn draw(&self, window: &mut Window);

    /// Draws this shape onto the given window at the given position using its style.
    fn draw_at(&self, window: &mut Window, x: f32, y: f32);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub fill_color: Color,
    pub outline_color: Color,
    pub outline_width: f32,
}

impl Style {
    /// Sets the fill color.
    pub fn fill(&mut self, color: Color) -> &mut Self {
        self.fill_color = color;
        self
    }

    /// Sets the outline color and width.
    pub fn outline(&mut self, color: Color, width: f32) -> &mut Self {
        self.outline_color = color;
        self.outline_width = width;
        self
    }
}

macro_rules! impl_position {
    ($name:ident) => {
        impl $name {
            #[doc = concat!("Creates a [`", stringify!($name), "`] at `(x, y)` with no size or style set.")]
            pub fn at(x: f32, y: f32) -> Self {
                Self {
                    position: Vec2 { x, y },
                    ..Default::default()
                }
            }

            #[doc = concat!("Sets the position of this [`", stringify!($name), "`].")]
            pub fn position(&mut self, x: f32, y: f32) -> &mut Self {
                self.position = Vec2 { x, y };
                self
            }
        }
    };
}

macro_rules! impl_size {
    ($name:ident) => {
        impl $name {
            #[doc = concat!("Creates a [`", stringify!($name), "`] at `(x, y)` with the given size.")]
            pub fn sized(x: f32, y: f32, width: f32, height: f32) -> Self {
                Self {
                    position: Vec2 { x, y },
                    size: Vec2::new(width, height),
                    ..Default::default()
                }
            }

            #[doc = concat!("Sets the size of this [`", stringify!($name), "`].")]
            pub fn size(&mut self, width: f32, height: f32) -> &mut Self {
                self.size = Vec2::new(width, height);
                self
            }
        }
    };
}

macro_rules! impl_styled {
    ($name:ident) => {
        impl $name {
            #[doc = concat!("Creates a fully specified [`", stringify!($name), "`] with position, size, and style.")]
            pub fn new(
                x: f32,
                y: f32,
                width: f32,
                height: f32,
                fill_color: Color,
                outline_color: Color,
                outline_width: f32
            ) -> Self {
                Self {
                    position: Vec2 { x, y },
                    size: Vec2::new(width, height),
                    style: Style { fill_color, outline_color, outline_width },
                }
            }

            #[doc = concat!("Sets the fill color of this [`", stringify!($name), "`].")]
            pub fn fill(&mut self, color: Color) -> &mut Self {
                self.style.fill_color = color;
                self
            }

            #[doc = concat!("Sets the outline color and width of this [`", stringify!($name), "`].")]
            pub fn outline(&mut self, color: Color, width: f32) -> &mut Self {
                self.style.outline(color, width);
                self
            }
        }
    };
}

macro_rules! impl_drawable {
    ($name:ident, $func:ident) => {
        impl Drawable for $name {
            fn draw(&self, window: &mut Window) {
                window.with_style(|window| {
                    window.fill(self.style.fill_color);
                    window.outline_color(self.style.outline_color);
                    window.outline_width(self.style.outline_width);
                    window.$func(self.position.x, self.position.y, self.size.x, self.size.y);
                });
            }

            fn draw_at(&self, window: &mut Window, x: f32, y: f32) {
                window.with_style(|window| {
                    window.fill(self.style.fill_color);
                    window.outline_color(self.style.outline_color);
                    window.outline_width(self.style.outline_width);
                    window.$func(x, y, self.size.x, self.size.y);
                });
            }
        }
    };
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
    pub style: Style,
    pub corner_radius: f32,
}

impl Rect {
    #[allow(clippy::too_many_arguments)]
    #[doc = concat!("Creates a fully specified [`Rect`] with position, size, and style.")]
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fill_color: Color,
        outline_color: Color,
        outline_width: f32,
        corner_radius: f32,
    ) -> Self {
        Self {
            position: Vec2 { x, y },
            size: Vec2::new(width, height),
            style: Style { fill_color, outline_color, outline_width },
            corner_radius,
        }
    }

    #[doc = concat!("Sets the fill color of this [`Rect`].")]
    pub fn fill(&mut self, color: Color) -> &mut Self {
        self.style.fill_color = color;
        self
    }

    #[doc = concat!("Sets the outline color and width of this [`Rect`].")]
    pub fn outline(&mut self, color: Color, width: f32) -> &mut Self {
        self.style.outline(color, width);
        self
    }

    pub fn corner_radius(&mut self, corner_radius: f32) -> &mut Self {
        self.corner_radius = corner_radius;
        self
    }
}

impl Drawable for Rect {
    fn draw(&self, window: &mut Window) {
        window.with_style(|window| {
            window.fill(self.style.fill_color);
            window.outline_color(self.style.outline_color);
            window.outline_width(self.style.outline_width);
            window.round_rect(self.position.x, self.position.y, self.size.x, self.size.y, self.corner_radius);
        });
    }

    fn draw_at(&self, window: &mut Window, x: f32, y: f32) {
        window.with_style(|window| {
            window.fill(self.style.fill_color);
            window.outline_color(self.style.outline_color);
            window.outline_width(self.style.outline_width);
            window.round_rect(x, y, self.size.x, self.size.y, self.corner_radius);
        });
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Ellipse {
    pub position: Vec2,
    pub size: Vec2,
    pub style: Style,
}

impl_position!(Rect);
impl_size!(Rect);

impl_position!(Ellipse);
impl_size!(Ellipse);
impl_styled!(Ellipse);
impl_drawable!(Ellipse, ellipse);

#[derive(Debug, Clone, Copy, Default)]
pub struct Line {
    pub start: Vec2,
    pub end: Vec2,
    pub style: Style,
}

impl Line {
    pub fn between(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            start: Vec2::new(x1, y1),
            end: Vec2::new(x2, y2),
            ..Default::default()
        }
    }

    pub fn new(
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        outline_width: f32,
        outline_color: Color
    ) -> Self {
        Self {
            start: Vec2::new(x1, y1),
            end: Vec2::new(x2, y2),
            style: Style { outline_width, outline_color, fill_color: Color::TRANSPARENT },
        }
    }

    pub fn width(
        &mut self,
        width: f32,
    ) -> &mut Self {
        self.style.outline_width = width;
        self
    }

    pub fn color(
        &mut self,
        color: Color,
    ) -> &mut Self {
        self.style.outline_color = color;
        self
    }
}

impl Drawable for Line {
    fn draw(&self, window: &mut Window) {
        window.with_style(|window| {
            window.outline(self.style.outline_color, self.style.outline_width);
            window.line(self.start.x, self.start.y, self.end.x, self.end.y);
        });
    }

    fn draw_at(&self, window: &mut Window, x: f32, y: f32) {
        let offset = self.end - self.start;
        window.with_style(|window| {
            window.outline(self.style.outline_color, self.style.outline_width);
            window.line(x, y, x + offset.x, y + offset.y);
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn rect_vertices(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fill_color: Color,
    outline_color: Color,
    outline_width: f32,
    corner_radius: f32,
) -> [Vertex; 6] {
    let (hw, hh) = (w / 2.0, h / 2.0);
    let (center_x, center_y) = (x + hw, y + hh);

    // half the outline width + an extra 1.5 for anti-aliasing
    let padding = (outline_width / 2.0) + 1.5;
    let pad_x = hw + padding;
    let pad_y = hh + padding;

    let v = |x, y| Vertex {
        position: Vec2::new(x + center_x, y + center_y),
        uv:       Vec2::new(x, y),
        radii:    Vec2::new(hw, hh),

        fill_color,
        outline_color,
        outline_width,

        corner_radius,

        kind: KIND_RECT,
    };
    [
        v(-pad_x, -pad_y), v( pad_x, -pad_y), v( pad_x,  pad_y), // triangle 1
        v(-pad_x, -pad_y), v( pad_x,  pad_y), v(-pad_x,  pad_y), // triangle 2
    ]
}

pub(crate) fn ellipse_vertices(
    x: f32,
    y: f32,
    rx: f32,
    ry: f32,
    fill_color: Color,
    outline_color: Color,
    outline_width: f32,
) -> [Vertex; 6] {
    // half the outline width + an extra 1.5 for anti-aliasing
    let padding = (outline_width / 2.0) + 1.5;
    let pad_x = rx + padding;
    let pad_y = ry + padding;

    let (x1, y1, x2, y2) = (x - pad_x, y - pad_y, x + pad_x, y + pad_y);

    let v = |x, y, ux, uy| Vertex {
        position: Vec2::new(x, y),
        uv: Vec2::new(ux, uy),
        radii: Vec2::new(rx, ry),

        fill_color,
        outline_color,
        outline_width,

        corner_radius: 0.,

        kind: KIND_ELLIPSE,
    };
    [
        v(x1, y1, -pad_x, -pad_y), v(x2, y1,  pad_x, -pad_y), v(x2, y2,  pad_x,  pad_y),
        v(x1, y1, -pad_x, -pad_y), v(x2, y2,  pad_x,  pad_y), v(x1, y2, -pad_x,  pad_y),
    ]
}

pub(crate) fn line_vertices(
    a: Vec2,
    b: Vec2,
    color: Color,
    width: f32,
) -> [Vertex; 6] {
    let displacement = b - a;
    let length = displacement.length();
    const EPSILON: f32 = 1e-6;

    if length < EPSILON {
        let radius = width / 2.;
        return ellipse_vertices(
            a.x,
            a.y,
            radius,
            radius,
            color,
            Color::TRANSPARENT,
            0.,
        )
    }

    let dir = displacement.normalize();
    let perp = Vec2::new(-dir.y, dir.x);

    let hw = width / 2.;
    let hl = length / 2.;

    let padding = 1.5;
    let pad_hw = hw + padding;
    let pad_hl = hl + hw + padding;

    let offset_x = dir * pad_hl;
    let offset_y = perp * pad_hw;
    let center = (a + b) / 2.;

    let corners = [
        center - offset_x - offset_y,
        center - offset_x + offset_y,
        center + offset_x + offset_y,
        center + offset_x - offset_y,
    ];

    let uvs = [
        Vec2::new(-pad_hl, -pad_hw),
        Vec2::new(-pad_hl,  pad_hw),
        Vec2::new( pad_hl,  pad_hw),
        Vec2::new( pad_hl, -pad_hw),
    ];

    let v = |i: usize| Vertex {
        position: corners[i],
        uv: uvs[i],
        radii: Vec2::new(hl, 0.),

        fill_color: color,
        outline_color: color,
        outline_width: width,

        corner_radius: 0.,

        kind: KIND_LINE,
    };

    [v(0), v(3), v(2), v(0), v(2), v(1)]
}

pub(crate) fn textured_vertices(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    uv_min: Vec2,
    uv_max: Vec2,
    fill_color: Color,
) -> [Vertex; 6] {
    let xs = [x, x + w];
    let ys = [y, y + h];
    let us = [uv_min.x, uv_max.x];
    let vs = [uv_min.y, uv_max.y];

    let v = |x, y| Vertex {
        position: Vec2::new(xs[x], ys[y]),
        uv: Vec2::new(us[x], vs[y]),
        radii: Vec2::ZERO,

        fill_color,
        outline_color: Color::TRANSPARENT,
        outline_width: 0.,

        corner_radius: 0.,

        kind: KIND_TEXTURED,
    };

    [
        v(0, 0), v(1, 0), v(1, 1),
        v(0, 0), v(1, 1), v(0, 1),
    ]
}
