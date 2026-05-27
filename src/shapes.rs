// TODO: text

use crate::{KIND_ELLIPSE, KIND_LINE, KIND_RECT, KIND_TEXTURED, Vertex, image::Image, transform::Transform2d, types::Color, vec::Vec2, window::Window};

/// Trait for types that can draw themselves onto a [`Window`].
pub trait Drawable {
    /// Draws this shape onto the given window.
    fn draw(&self, window: &mut Window);

    /// Draws this shape onto the given window at the given position.
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
            #[doc = concat!("Sets the fill color of this [`", stringify!($name), "`].")]
            pub fn fill(&mut self, color: Color) -> &mut Self {
                self.style.fill_color = color;
                self
            }

            #[doc = concat!("Sets the outline color of this [`", stringify!($name), "`].")]
            pub fn outline_color(&mut self, color: Color) -> &mut Self {
                self.style.outline_color = color;
                self
            }

            #[doc = concat!("Sets the outline width of this [`", stringify!($name), "`].")]
            pub fn outline_width(&mut self, width: f32) -> &mut Self {
                self.style.outline_width = width;
                self
            }

            #[doc = concat!("Sets the outline color and width of this [`", stringify!($name), "`].")]
            pub fn outline(&mut self, color: Color, width: f32) -> &mut Self {
                self.style.outline(color, width);
                self
            }
        }
    };
    ($name:ident, no_outline) => {
        impl $name {
            #[doc = concat!("Sets the fill color of this [`", stringify!($name), "`].")]
            pub fn fill(&mut self, color: Color) -> &mut Self {
                self.style.fill_color = color;
                self
            }
        }
    };
}

macro_rules! impl_transformed {
    ($name:ident) => {
        impl $name {
            #[doc = concat!("Sets the transform of this [`", stringify!($name), "`].")]
            pub fn transform(&mut self, transform: Transform2d) -> &mut Self {
                self.transform = transform;
                self
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
    pub transform: Transform2d,
}

impl_position!(Rect);
impl_size!(Rect);
impl_styled!(Rect);
impl_transformed!(Rect);

impl Rect {
    #[allow(clippy::too_many_arguments)]
    /// Creates a fully specified [`Rect`] with position, size, style, and transform.
    pub fn new(
        position: Vec2,
        size: Vec2,
        style: Style,
        corner_radius: f32,
        transform: Transform2d,
    ) -> Self {
        Self {
            position,
            size,
            style,
            corner_radius,
            transform,
        }
    }

    /// Sets the corner radius of this [`Rect`].
    pub fn corner_radius(&mut self, corner_radius: f32) -> &mut Self {
        self.corner_radius = corner_radius;
        self
    }
}

impl Drawable for Rect {
    fn draw(&self, window: &mut Window) {
        window.with_style(|window| {
            window.with_transform(self.transform, |window| {
                window.fill(self.style.fill_color);
                window.outline_color(self.style.outline_color);
                window.outline_width(self.style.outline_width);
                window.round_rect(self.position.x, self.position.y, self.size.x, self.size.y, self.corner_radius);
            });
        });
    }

    fn draw_at(&self, window: &mut Window, x: f32, y: f32) {
        window.with_style(|window| {
            window.with_transform(self.transform, |window| {
                window.fill(self.style.fill_color);
                window.outline_color(self.style.outline_color);
                window.outline_width(self.style.outline_width);
                window.round_rect(x, y, self.size.x, self.size.y, self.corner_radius);
            });
        });
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Ellipse {
    pub position: Vec2,
    pub size: Vec2,
    pub style: Style,
    pub transform: Transform2d,
}

impl Ellipse {
    /// Creates a fully specified [`Ellipse`] with position, size, style and transform.
    pub fn new(
        position: Vec2,
        size: Vec2,
        style: Style,
        transform: Transform2d,
    ) -> Self {
        Self {
            position,
            size,
            style,
            transform,
        }
    }
}

impl Drawable for Ellipse {
    fn draw(&self, window: &mut Window) {
        window.with_style(|window| {
            window.with_transform(self.transform, |window| {
                window.fill(self.style.fill_color);
                window.outline_color(self.style.outline_color);
                window.outline_width(self.style.outline_width);
                window.ellipse(self.position.x, self.position.y, self.size.x, self.size.y);
            });
        });
    }

    fn draw_at(&self, window: &mut Window, x: f32, y: f32) {
        window.with_style(|window| {
            window.with_transform(self.transform, |window| {
                window.fill(self.style.fill_color);
                window.outline_color(self.style.outline_color);
                window.outline_width(self.style.outline_width);
                window.ellipse(x, y, self.size.x, self.size.y);
            });
        });
    }
}

impl_position!(Ellipse);
impl_size!(Ellipse);
impl_styled!(Ellipse);
impl_transformed!(Ellipse);

#[derive(Debug, Clone, Copy, Default)]
pub struct Line {
    pub start: Vec2,
    pub end: Vec2,
    pub style: Style,
    pub transform: Transform2d,
}

impl Line {
    /// Creates a fully specified [`Line`] with start point, end point, style, and transform.
    pub fn new(
        start: Vec2,
        end: Vec2,
        outline_width: f32,
        outline_color: Color,
        transform: Transform2d,
    ) -> Self {
        Self {
            start,
            end,
            style: Style { outline_width, outline_color, ..Default::default() },
            transform,
        }
    }

    /// Creates a [`Line`] between two points.
    pub fn between(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            start: Vec2::new(x1, y1),
            end: Vec2::new(x2, y2),
            ..Default::default()
        }
    }

    /// Sets the start point of this [`Line`].
    pub fn start(&mut self, x: f32, y: f32) -> &mut Self {
        self.start = Vec2 { x, y };
        self
    }

    /// Sets the end point of this [`Line`].
    pub fn end(&mut self, x: f32, y: f32) -> &mut Self {
        self.end = Vec2 { x, y };
        self
    }

    /// Sets the start and end points of this [`Line`].
    pub fn points(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> &mut Self {
        self.start = Vec2::new(x1, y1);
        self.end = Vec2::new(x2, y2);
        self
    }

    /// Sets the color of this [`Line`].
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.style.outline_color = color;
        self
    }

    /// Sets the width of this [`Line`].
    pub fn width(&mut self, width: f32) -> &mut Self {
        self.style.outline_width = width;
        self
    }

    /// Sets the color and width of this [`Line`].
    pub fn style(&mut self, color: Color, width: f32) -> &mut Self {
        self.style.outline_color = color;
        self.style.outline_width = width;
        self
    }
}

impl Drawable for Line {
    fn draw(&self, window: &mut Window) {
        window.with_style(|window| {
            window.with_transform(self.transform, |window| {
                window.outline(self.style.outline_color, self.style.outline_width);
                window.line(self.start.x, self.start.y, self.end.x, self.end.y);
            });
        });
    }

    /// Draws this line onto the given window at the given position, relative to the starting point.
    fn draw_at(&self, window: &mut Window, x: f32, y: f32) {
        let offset = self.end - self.start;
        window.with_style(|window| {
            window.with_transform(self.transform, |window| {
                window.outline(self.style.outline_color, self.style.outline_width);
                window.line(x, y, x + offset.x, y + offset.y);
            });
        });
    }
}

#[derive(Debug, Clone)]
pub struct ImageRect {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub image: Image,
    pub transform: Transform2d,
}

impl_size!(ImageRect);
impl_transformed!(ImageRect);

impl ImageRect {
    /// Creates a fully specified [`ImageRect`] with start point, end point, color, image, and transform.
    pub fn new(
        position: Vec2,
        size: Vec2,
        color: Color,
        image: Image,
        transform: Transform2d,
    ) -> Self {
        Self {
            position,
            size,
            color,
            image,
            transform,
        }
    }

    /// Creates a [`ImageRect`] with a given image.
    pub fn from_image(image: Image) -> Self {
        Self {
            position: Vec2::default(),
            size: Vec2::default(),
            color: Color::default(),
            image,
            transform: Transform2d::default(),
        }
    }

    /// Sets the position of this [`ImageRect`].
    pub fn position(&mut self, x: f32, y: f32) -> &mut Self {
        self.position = Vec2 { x, y };
        self
    }

    /// Sets the image displayed by this [`ImageRect`].
    pub fn image(&mut self, image: Image) -> &mut Self {
        self.image = image;
        self
    }

    /// Sets the color of this [`ImageRect`].
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}

impl Drawable for ImageRect {
    fn draw(&self, window: &mut Window) {
        window.with_style(|window| {
            window.with_transform(self.transform, |window| {
                window.fill(self.color);
                window.image(self.image.clone(), self.position.x, self.position.y, self.size.x, self.size.y);
            });
        });
    }

    fn draw_at(&self, window: &mut Window, x: f32, y: f32) {
        window.with_style(|window| {
            window.with_transform(self.transform, |window| {
                window.fill(self.color);
                window.image(self.image.clone(), x, y, self.size.x, self.size.y);
            });
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
        uv:       Vec2::new(ux, uy),
        radii:    Vec2::new(rx, ry),

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
        uv:       uvs[i],
        radii:    Vec2::new(hl, 0.),

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
        uv:       Vec2::new(us[x], vs[y]),
        radii:    Vec2::ZERO,

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
