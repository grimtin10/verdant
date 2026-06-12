use crate::{canvas::RenderSurface, image::Image, text::{RichText, Text}, transform::Transform2d, types::Color, vec::Vec2};

/// Trait for types that can draw themselves onto a [`RenderSurface`].
pub trait Drawable {
    /// Draws this shape onto the given window.
    fn draw(&self, window: &mut impl RenderSurface);

    /// Draws this shape onto the given window at the given position.
    /// Transforms are still applied.
    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32);
}

/// The scaling mode for outlines and corner radii.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScalingMode {
    /// Scales with both local transforms and camera zoom, behaving like physical geometry.
    /// The default.
    #[default]
    Geometric,

    /// Stays at a constant screen pixel size, completely unaffected by zoom or transforms.
    Constant,

    /// Scales only with local transforms, remaining unaffected by view scaling.
    WithTransform,

    /// Scales only with view scaling, remaining unaffected by local transforms.
    WithView,
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The fill color of this shape.
    pub fill_color: Color,

    /// The outline color of this shape.
    pub outline_color: Color,
    /// The outline width of this shape.
    pub outline_width: f32,
    /// How the outline of this shape scales with local transform and zoom (default: None).
    pub outline_scaling: ScalingMode,

    /// The corner radius of this shape.
    pub corner_radius: f32,
    /// How the corner radius of this shape scales with local transform and zoom (default: None).
    pub corner_scaling: ScalingMode,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill_color: Color::WHITE,

            outline_color: Color::default(),
            outline_width: f32::default(),
            outline_scaling: ScalingMode::default(),

            corner_radius: f32::default(),
            corner_scaling: ScalingMode::default(),
        }
    }
}

impl Style {
    /// Sets the fill color.
    pub fn fill(&mut self, color: Color) -> Self {
        self.fill_color = color;
        *self
    }

    /// Sets the outline color.
    pub fn outline_color(&mut self, color: Color) -> Self {
        self.outline_color = color;
        *self
    }

    /// Sets the outline width.
    pub fn outline_width(&mut self, width: f32) -> Self {
        self.outline_width = width;
        *self
    }

    /// Sets the outline scaling mode.
    pub fn outline_scaling(&mut self, scaling: ScalingMode) -> Self {
        self.outline_scaling = scaling;
        *self
    }

    /// Sets the outline color and width.
    pub fn outline(&mut self, color: Color, width: f32) -> Self {
        self.outline_color = color;
        self.outline_width = width;
        *self
    }

    /// Sets the outline color, width, and scaling mode.
    pub fn outline_style(&mut self, color: Color, width: f32, scaling: ScalingMode) -> Self {
        self.outline_color = color;
        self.outline_width = width;
        self.outline_scaling = scaling;
        *self
    }

    /// Sets the corner radius.
    pub fn corner_radius(&mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        *self
    }

    /// Sets the corner scaling mode.
    pub fn corner_scaling(&mut self, scaling: ScalingMode) -> Self {
        self.corner_scaling = scaling;
        *self
    }

    /// Sets the corner radius and scaling mode.
    pub fn corner_style(&mut self, radius: f32, scaling: ScalingMode) -> Self {
        self.corner_radius = radius;
        self.corner_scaling = scaling;
        *self
    }
}

macro_rules! impl_new {
    ($name:ident) => {
        impl $name {
            #[doc = concat!("Creates a fully specified [`", stringify!($name), "`] with position, size, style, and transform.")]
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
    };
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
                $map(self)
            }
        }
    };
    (no_at, $name:ident, $map:expr) => {
        impl $name {
            #[doc = concat!("Sets the position of this [`", stringify!($name), "`].")]
            pub fn position(&mut self, x: f32, y: f32) -> Self {
                self.position = Vec2 { x, y };
                $map(self)
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
                $map(self)
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
                $map(self)
            }

            #[doc = concat!("Sets the outline color of this [`", stringify!($name), "`].")]
            pub fn outline_color(&mut self, color: Color) -> Self {
                self.style.outline_color = color;
                $map(self)
            }

            #[doc = concat!("Sets the outline width of this [`", stringify!($name), "`].")]
            pub fn outline_width(&mut self, width: f32) -> Self {
                self.style.outline_width = width;
                $map(self)
            }

            #[doc = concat!("Sets the outline color and width of this [`", stringify!($name), "`].")]
            pub fn outline(&mut self, color: Color, width: f32) -> Self {
                self.style.outline(color, width);
                $map(self)
            }

            #[doc = concat!("Sets the outline scaling mode of this [`", stringify!($name), "`].")]
            pub fn outline_scaling(&mut self, scaling: ScalingMode) -> Self {
                self.style.outline_scaling = scaling;
                $map(self)
            }

            #[doc = concat!("Sets the outline color, width, and scaling mode of this [`", stringify!($name), "`].")]
            pub fn outline_style(&mut self, color: Color, width: f32, scaling: ScalingMode) -> Self {
                self.style.outline_style(color, width, scaling);
                $map(self)
            }
        }
    };
    ($name:ident, no_outline, $map:expr) => {
        impl $name {
            #[doc = concat!("Sets the fill color of this [`", stringify!($name), "`].")]
            pub fn fill(&mut self, color: Color) -> Self {
                self.style.fill_color = color;
                $map(self)
            }
        }
    };
}

macro_rules! impl_rounded {
    ($name:ident, $map:expr) => {
        impl $name {
            #[doc = concat!("Sets the corner radius of this [`", stringify!($name), "`].")]
            pub fn corner_radius(&mut self, radius: f32) -> Self {
                self.style.corner_radius = radius;
                *self
            }

            #[doc = concat!("Sets the corner scaling mode of this [`", stringify!($name), "`].")]
            pub fn corner_scaling(&mut self, scaling: ScalingMode) -> Self {
                self.style.corner_scaling = scaling;
                *self
            }

            #[doc = concat!("Sets the corner radius and scaling mode of this [`", stringify!($name), "`].")]
            pub fn corner_style(&mut self, radius: f32, scaling: ScalingMode) -> Self {
                self.style.corner_style(radius, scaling);
                *self
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
                $map(self)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
    pub style: Style,
    pub transform: Transform2d,
}

impl_new!(Rect);
impl_position!(Rect, |s: &mut Rect| *s);
impl_size!(Rect, |s: &mut Rect| *s);
impl_styled!(Rect, |s: &mut Rect| *s);
impl_rounded!(Rect, |s: &mut Rect| *s);
impl_transformed!(Rect, |s: &mut Rect| *s);

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
                    window.corner_radius(self.style.corner_radius);
                    window.rect(0., 0., self.size.x, self.size.y);
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

impl_new!(Ellipse);
impl_position!(Ellipse, |s: &mut Ellipse| *s);
impl_size!(Ellipse, |s: &mut Ellipse| *s);
impl_styled!(Ellipse, |s: &mut Ellipse| *s);
impl_transformed!(Ellipse, |s: &mut Ellipse| *s);

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

// `.clone()` here is fine, because it's just cloning an `Arc` and everything else is `Copy`
impl_position!(no_at, ImageRect, |s: &mut ImageRect| s.clone());
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

// `.clone()` here is fine, because it's just cloning an `Arc` and everything else is `Copy`
impl_position!(no_at, Text, |s: &mut Text| s.clone());
impl_transformed!(Text, |s: &mut Text| s.clone());

impl_position!(no_at, RichText, |s: &mut RichText| s.clone());
impl_transformed!(RichText, |s: &mut RichText| s.clone());
