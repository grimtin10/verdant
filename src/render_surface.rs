use crate::{RendererResult, canvas::Canvas, image::Image, shapes::ScalingMode, text::{Font, HorizontalAlign, Span, TextLayout, VerticalAlign}, transform::Transform2d, types::Color, vec::Vec2, view::ViewMode};

/// A surface that can be drawn onto.
///
/// [`RenderSurface`] uses a stateful model; style properties like fill color, outline, and text
/// alignment are set once and apply to all subsequent draw calls until changed. Use [`with_style`]
/// to scope state changes so they don't leak into the surrounding context.
///
/// The coordinate origin is at the top-left by default and can be shifted with [`set_origin`].
/// Use [`set_view`] to define a logical coordinate space independent of the physical surface size.
pub trait RenderSurface {
    /// Clears the window to the given color at the start of each frame.
    /// Any vertices queued before this call are discarded, since they would be covered by the clear.
    fn background(&mut self, color: Color);

    // styling
    /// Sets the fill color for subsequent shapes.
    fn fill(&mut self, color: Color);
    /// Disables fill for subsequent shapes.
    fn no_fill(&mut self);
    /// Sets the outline color for subsequent shapes.
    fn outline_color(&mut self, color: Color);
    /// Sets the outline width for subsequent shapes.
    fn outline_width(&mut self, width: f32);
    /// Sets the outline color and width for subsequent shapes.
    fn outline(&mut self, color: Color, width: f32);
    /// Sets the outline scaling mode for subsequent shapes.
    fn outline_scaling(&mut self, scaling: ScalingMode);
    /// Sets the outline color, width, and scaling for subsequent shapes.
    fn outline_style(&mut self, color: Color, width: f32, scaling: ScalingMode);
    /// Disables the outline for subsequent shapes.
    fn no_outline(&mut self);
    /// Sets the corner radius for subsequent shapes.
    fn corner_radius(&mut self, radius: f32);
    /// Sets the corner scaling mode for subsequent shapes.
    fn corner_scaling(&mut self, scaling: ScalingMode);
    /// Sets the corner radius and scaling mode for subsequent shapes.
    fn corner_style(&mut self, radius: f32, scaling: ScalingMode);
    /// Sets the outline and corner scaling modes for subsequent shapes.
    fn scaling_modes(&mut self, outline_scaling: ScalingMode, corner_scaling: ScalingMode);
    /// Resets the current style back to the default.
    /// ([`Color::WHITE`] fill, no outline)
    fn clear_style(&mut self);

    // shapes
    /// Draws a rectangle at `(x, y)` with the given width and height,
    /// using the current fill and outline style.
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32);
    /// Draws an ellipse centered at `(x, y)` with horizontal radius `rx` and vertical radius `ry`,
    /// using the current fill and outline style.
    fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32);
    /// Draws a line from `(x1, y1)` to `(x2, y2)` using the current outline color and width.
    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32);
    /// Draws an image at `(x, y)` with the given width and height, tinting with the current fill color.
    fn image(&mut self, image: impl AsRef<Image>, x: f32, y: f32, w: f32, h: f32);
    /// Draws a canvas at `(x, y)` with the given width and height, tinting with the current fill color.
    fn composite(&mut self, canvas: impl AsRef<Canvas>, x: f32, y: f32, w: f32, h: f32);

    // text
    /// Sets the horizontal text alignment for subsequent text calls.
    /// Affects rich text.
    fn horizontal_text_align(&mut self, align: HorizontalAlign);
    /// Sets the vertical text alignment for subsequent text calls.
    /// Affects rich text.
    fn vertical_text_align(&mut self, align: VerticalAlign);
    /// Sets the text alignment for subsequent text calls.
    /// Affects rich text.
    fn text_align(&mut self, horizontal: HorizontalAlign, vertical: VerticalAlign);
    /// Sets the alignment per-line for subsequent text calls.
    /// Affects rich text.
    fn line_align(&mut self, align: HorizontalAlign);
    /// Sets the font size (in pixels) for subsequent text calls.
    /// Does not affect rich text.
    fn font_size(&mut self, size_px: f32);
    /// Draws text at `(x, y)` with the given font using the current fill color and text size.
    fn text(&mut self, font: impl AsRef<Font>, x: f32, y: f32, text: impl ToString);
    /// Draws rich text at `(x, y)` with each span's font and style.
    fn rich_text(&mut self, x: f32, y: f32, spans: &[Span]);

    /// Gets the layout for `text` with the given font and the current font size and text alignment.
    fn text_layout(&mut self, font: impl AsRef<Font>, text: impl ToString) -> TextLayout;
    /// Gets the size of `text` with the given font and the current font size.
    fn text_size(&mut self, font: impl AsRef<Font>, text: impl ToString) -> Vec2;
    /// Gets the width of `text` with the given font and the current font size.
    fn text_width(&mut self, font: impl AsRef<Font>, text: impl ToString) -> f32;
    /// Gets the height of `text` with the given font and the current font size.
    fn text_height(&mut self, font: impl AsRef<Font>, text: impl ToString) -> f32;

    /// Gets the layout for `spans` with the current text alignment.
    fn rich_text_layout(&mut self, spans: &[Span]) -> TextLayout;
    /// Gets the size of `spans`.
    fn rich_text_size(&mut self, spans: &[Span]) -> Vec2;
    /// Gets the width of `spans`.
    fn rich_text_width(&mut self, spans: &[Span]) -> f32;
    /// Gets the height of `spans`.
    fn rich_text_height(&mut self, spans: &[Span]) -> f32;

    // view
    /// Sets the logical view size and scaling mode.
    fn set_view(&mut self, width: f32, height: f32, view_mode: ViewMode);
    /// Clears the logical view size and resets the scaling mode to `Unscaled`.
    fn clear_view(&mut self);
    /// Offsets the view origin, shifting where `(0, 0)` appears on screen.
    fn set_origin(&mut self, x: f32, y: f32);
    /// Clears the origin, setting it back to the top left corner.
    fn clear_origin(&mut self);

    // isolation closures
    /// Temporarily isolates style and view state for the duration of `commands`.
    /// Any changes to style or view made inside will be reverted when it returns.
    fn with_style(&mut self, commands: impl FnOnce(&mut Self));
    /// Applies `transform` relative to the current transform for the duration of `commands`.
    fn with_transform(&mut self, transform: impl AsRef<Transform2d>, commands: impl FnOnce(&mut Self));

    /// Submits all queued draw calls to the GPU and presents the frame.
    fn flush(&mut self) -> RendererResult<()>;
}
