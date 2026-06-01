use crate::{RendererResult, image::Image, text::{Font, HorizontalAlign, Span, VerticalAlign}, transform::Transform2d, types::Color, view::ViewMode};

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
    /// Disables the outline for subsequent shapes.
    fn no_outline(&mut self);
    /// Resets the current style back to the default.
    /// ([`Color::WHITE`] fill, no outline)
    fn clear_style(&mut self);

    // shapes
    /// Draws a rectangle at `(x, y)` with the given width and height,
    /// using the current fill and outline style.
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32);
    /// Draws a rounded rectangle at `(x, y)` with the given width, height, and corner radius,
    /// using the current fill and outline style.
    fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, corner_radius: f32);
    /// Draws an ellipse centered at `(x, y)` with horizontal radius `rx` and vertical radius `ry`,
    /// using the current fill and outline style.
    fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32);
    /// Draws a line from `(x1, y1)` to `(x2, y2)` using the current outline color and width.
    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32);
    /// Draws an image at `(x, y)` with the given width and height using the current fill color.
    fn image(&mut self, image: impl AsRef<Image>, x: f32, y: f32, w: f32, h: f32);

    // text
    /// Sets the horizontal text alignment for subsequent text calls.
    /// Effects rich text.
    fn horizontal_text_align(&mut self, align: HorizontalAlign);
    /// Sets the vertical text alignment for subsequent text calls.
    /// Effects rich text.
    fn vertical_text_align(&mut self, align: VerticalAlign);
    /// Sets the text alignment for subsequent text calls.
    /// Effects rich text.
    fn text_align(&mut self, horizontal: HorizontalAlign, vertical: VerticalAlign);
    /// Sets the alignment per-line for subsequent text calls.
    /// Effects rich text.
    fn line_align(&mut self, align: HorizontalAlign);
    /// Sets the text size (in pixels) for subsequent text calls.
    /// Does not effect rich text.
    fn text_size(&mut self, size_px: f32);
    /// Draws text at `(x, y)` with the given font using the current fill color and text size.
    fn text(&mut self, font: impl AsRef<Font>, x: f32, y: f32, text: impl ToString);
    /// Draws rich text at `(x, y)` with each span's font and style.
    fn rich_text(&mut self, x: f32, y: f32, spans: &[Span]);

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
