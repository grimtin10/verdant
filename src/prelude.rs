//! The Verdant Prelude.
//!
//! This module re-exports the most commonly used types and traits for quick
//! access. It is designed for application developers to easily import via:
//!
//! ```rust
//! use verdant::prelude::*;
//! ```
//!
//! *Note: If you are building a library on top of Verdant, it is recommended to
//! import types explicitly from their respective modules to avoid future namespace
//! collisions.*

// the only time i have ever organized my imports
pub use crate::{Renderer, RendererResult, rgb, rgba, rgb255, rgba255};
pub use crate::canvas::Canvas;
pub use crate::event::{Key, KeyCode, MouseButton, ScrollDelta, WindowEvent};
pub use crate::image::Image;
pub use crate::render_surface::RenderSurface;
pub use crate::shapes::{Drawable, Ellipse, Line, Rect, ScalingMode, Style};
pub use crate::transform::Transform2d;
pub use crate::types::Color;
pub use crate::view::ViewMode;
pub use crate::window::{WindowId, WindowProperties};

#[cfg(feature = "text")]
pub use crate::text::{Font, HorizontalAlign, RichText, Span, Text, TextStyle, VerticalAlign};

#[cfg(not(feature = "glam"))]
pub use crate::vec::{Vec2, Vec3, Vec4};

#[cfg(feature = "glam")]
pub use glam::{Vec2, Vec3, Vec4};
