// TODO: right now, the shape API is technically a bit slower than the state machine API
//       it could get optimized out but i'm unsure if it would be
//       not a huge issue, just something to note

use crate::{canvas::RenderSurface, image::Image, text::{RichText, Text}, transform::Transform2d, types::Color, vec::Vec2};

/// Trait for types that can draw themselves onto a [`RenderSurface`].
pub trait Drawable {
    /// Draws this shape onto the given window.
    fn draw(&self, window: &mut impl RenderSurface);

    /// Draws this shape onto the given window at the given position.
    /// Transforms are still applied.
    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32);
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

    /// Sets the outline color.
    pub fn outline_color(&mut self, color: Color) -> &mut Self {
        self.outline_color = color;
        self
    }

    /// Sets the outline width.
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

macro_rules! impl_position {
    ($name:ident, $map:expr) => {
        impl $name {
            #[doc = concat!("Creates a [`", stringify!($name), "`] at `(x, y)` with nothing else set.")]
            pub fn at(x: f32, y: f32) -> Self {
                Self {
                    position: Vec2 { x, y },
                    ..Default::default()
                }
            }

            #[doc = concat!("Sets the position of this [`", stringify!($name), "`].")]
            pub fn position(&mut self, x: f32, y: f32) -> Self {
                self.position = Vec2 { x, y };
                ($map)(self)
            }
        }
    };
}

macro_rules! impl_size {
    ($name:ident, $map:expr) => {
        impl $name {
            #[doc = concat!("Sets the size of this [`", stringify!($name), "`].")]
            pub fn size(&mut self, width: f32, height: f32) -> Self {
                self.size = Vec2::new(width, height);
                ($map)(self)
            }
        }
    };
}

macro_rules! impl_styled {
    ($name:ident, $map:expr) => {
        impl $name {
            #[doc = concat!("Sets the fill color of this [`", stringify!($name), "`].")]
            pub fn fill(&mut self, color: Color) -> Self {
                self.style.fill_color = color;
                ($map)(self)
            }

            #[doc = concat!("Sets the outline color of this [`", stringify!($name), "`].")]
            pub fn outline_color(&mut self, color: Color) -> Self {
                self.style.outline_color = color;
                ($map)(self)
            }

            #[doc = concat!("Sets the outline width of this [`", stringify!($name), "`].")]
            pub fn outline_width(&mut self, width: f32) -> Self {
                self.style.outline_width = width;
                ($map)(self)
            }

            #[doc = concat!("Sets the outline color and width of this [`", stringify!($name), "`].")]
            pub fn outline(&mut self, color: Color, width: f32) -> Self {
                self.style.outline(color, width);
                ($map)(self)
            }
        }
    };
    ($name:ident, no_outline, $map:expr) => {
        impl $name {
            #[doc = concat!("Sets the fill color of this [`", stringify!($name), "`].")]
            pub fn fill(&mut self, color: Color) -> Self {
                self.style.fill_color = color;
                ($map)(self)
            }
        }
    };
}

macro_rules! impl_transformed {
    ($name:ident, $map:expr) => {
        impl $name {
            #[doc = concat!("Sets the transform of this [`", stringify!($name), "`].")]
            pub fn transform(&mut self, transform: Transform2d) -> Self {
                self.transform = transform;
                ($map)(self)
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

impl_position!(Rect, |s: &mut Rect| *s);
impl_size!(Rect, |s: &mut Rect| *s);
impl_styled!(Rect, |s: &mut Rect| *s);
impl_transformed!(Rect, |s: &mut Rect| *s);

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
    pub fn corner_radius(&mut self, corner_radius: f32) -> Self {
        self.corner_radius = corner_radius;
        *self
    }
}

impl Drawable for Rect {
    fn draw(&self, window: &mut impl RenderSurface) {
        self.draw_at(window, self.position.x, self.position.y);
    }

    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32) {
        window.with_style(|window| {
            window.with_transform(
                self.transform
                    .then(Transform2d::translation(x, y)),
                |window| {
                    window.fill(self.style.fill_color);
                    window.outline_color(self.style.outline_color);
                    window.outline_width(self.style.outline_width);
                    window.round_rect(0., 0., self.size.x, self.size.y, self.corner_radius);
                }
            );
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
    fn draw(&self, window: &mut impl RenderSurface) {
        self.draw_at(window, self.position.x, self.position.y);
    }

    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32) {
        window.with_style(|window| {
            window.with_transform(
                self.transform
                    .then(Transform2d::translation(x, y)),
                |window| {
                    window.fill(self.style.fill_color);
                    window.outline_color(self.style.outline_color);
                    window.outline_width(self.style.outline_width);
                    window.ellipse(0., 0., self.size.x, self.size.y);
                }
            );
        });
    }
}

impl_position!(Ellipse, |s: &mut Ellipse| *s);
impl_size!(Ellipse, |s: &mut Ellipse| *s);
impl_styled!(Ellipse, |s: &mut Ellipse| *s);
impl_transformed!(Ellipse, |s: &mut Ellipse| *s);

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
    pub fn start(&mut self, x: f32, y: f32) -> Self {
        self.start = Vec2 { x, y };
        *self
    }

    /// Sets the end point of this [`Line`].
    pub fn end(&mut self, x: f32, y: f32) -> Self {
        self.end = Vec2 { x, y };
        *self
    }

    /// Sets the start and end points of this [`Line`].
    pub fn points(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        self.start = Vec2::new(x1, y1);
        self.end = Vec2::new(x2, y2);
        *self
    }

    /// Sets the color of this [`Line`].
    pub fn color(&mut self, color: Color) -> Self {
        self.style.outline_color = color;
        *self
    }

    /// Sets the width of this [`Line`].
    pub fn width(&mut self, width: f32) -> Self {
        self.style.outline_width = width;
        *self
    }

    /// Sets the color and width of this [`Line`].
    pub fn style(&mut self, color: Color, width: f32) -> Self {
        self.style.outline_color = color;
        self.style.outline_width = width;
        *self
    }
}

impl Drawable for Line {
    fn draw(&self, window: &mut impl RenderSurface) {
        self.draw_at(window, self.start.x, self.start.y);
    }

    /// Draws this line onto the given window at the given position, relative to the starting point.
    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32) {
        let offset = self.end - self.start;
        window.with_style(|window| {
            window.with_transform(
                self.transform
                    .then(Transform2d::translation(x, y)),
                |window| {
                    window.outline(self.style.outline_color, self.style.outline_width);
                    window.line(0., 0., offset.x, offset.y);
                }
            );
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

impl_size!(ImageRect, |s: &mut ImageRect| s.clone());
impl_transformed!(ImageRect, |s: &mut ImageRect| s.clone());

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
            color: Color::WHITE,
            image,
            transform: Transform2d::default(),
        }
    }

    /// Sets the position of this [`ImageRect`].
    pub fn position(&mut self, x: f32, y: f32) -> Self {
        self.position = Vec2 { x, y };
        self.clone()
    }

    /// Sets the image displayed by this [`ImageRect`].
    pub fn image(&mut self, image: Image) -> Self {
        self.image = image;
        self.clone()
    }

    /// Sets the color of this [`ImageRect`].
    pub fn color(&mut self, color: Color) -> Self {
        self.color = color;
        self.clone()
    }
}

impl Drawable for ImageRect {
    fn draw(&self, window: &mut impl RenderSurface) {
        self.draw_at(window, self.position.x, self.position.y);
    }

    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32) {
        window.with_style(|window| {
            window.with_transform(
                self.transform
                    .then(Transform2d::translation(x, y)),
                |window| {
                    window.fill(self.color);
                    window.image(&self.image, 0., 0., self.size.x, self.size.y);
                }
            );
        });
    }
}

// TODO: something something performance....
impl_transformed!(Text, |s: &mut Text| s.clone());
impl_transformed!(RichText, |s: &mut RichText| s.clone());
