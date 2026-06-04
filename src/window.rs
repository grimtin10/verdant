use std::{collections::HashSet, sync::Arc};

use wgpu::{CurrentSurfaceTexture, Extent3d, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration, SurfaceTexture};
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::{GpuContext, RendererResult, canvas::{Canvas, RenderSurface}, image::Image, text::{Font, HorizontalAlign, Span, VerticalAlign}, transform::Transform2d, types::Color, vec::Vec2, view::ViewMode};

// TODO: the ability to toggle if you want stroke scaling or not with views/transforms

#[derive(Debug, Default)]
pub(crate) struct WindowContext {
    pub mouse_x: f64,
    pub mouse_y: f64,

    pub focused: bool,
}

pub struct Window {
    pub(crate) inner_window: Arc<Box<dyn winit::window::Window>>,

    canvas: Canvas,

    surface: Surface<'static>,
    config: SurfaceConfiguration,

    gpu_context: Arc<GpuContext>,
    context: WindowContext,
}

impl Window {
    pub(crate) fn new(
        inner_window: Arc<Box<dyn winit::window::Window>>,
        surface: Surface<'static>,
        config: SurfaceConfiguration,
        gpu_context: Arc<GpuContext>,
    ) -> Self {
        Self {
            inner_window,

            canvas: Canvas::new(config.width, config.height, true),

            surface,
            config,

            gpu_context,
            context: WindowContext::default(),
        }
    }

    pub(crate) fn get_frame(&self) -> Option<SurfaceTexture> {
        match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(tex)
            | CurrentSurfaceTexture::Suboptimal(tex) => Some(tex),

            CurrentSurfaceTexture::Outdated
            | CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.gpu_context.device, &self.config);
                None
            }

            _ => None
        }
    }

    pub(crate) fn present_blank_frame(&self) -> RendererResult<()> {
        let frame = loop {
            if let Some(frame) = self.get_frame() {
                break frame;
            }
        };

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.gpu_context.device.create_command_encoder(&Default::default());

        encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK.into()),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        self.gpu_context.queue.submit([encoder.finish()]);
        frame.present();

        Ok(())
    }

    pub(crate) fn on_resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 { return; }

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.gpu_context.device, &self.config);

        self.canvas.write().resize(size.width, size.height);
    }

    pub(crate) fn on_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.context.mouse_x = position.x;
        self.context.mouse_y = position.y;
    }

    pub(crate) fn on_focus_update(&mut self, focused: bool) {
        self.context.focused = focused;
    }

    /// Returns the current width of the window in pixels.
    pub fn get_width(&self) -> f32 {
        self.canvas.read().view.window_size().x
    }

    /// Returns the current height of the window in pixels.
    pub fn get_height(&self) -> f32 {
        self.canvas.read().view.window_size().y
    }

    /// Returns the current size of the window as `(width, height)` in pixels.
    pub fn get_size(&self) -> Vec2 {
        self.canvas.read().view.window_size()
    }

    /// Returns the mouse X position, adjusted for the current view transform and letterboxing.
    pub fn get_mouse_x(&self) -> f32 {
        let letterbox = self.canvas.read().view.letterbox();
        (self.context.mouse_x as f32 - letterbox.2) / letterbox.0 - self.canvas.read().view.origin().x
    }

    /// Returns the mouse Y position, adjusted for the current view transform and letterboxing.
    pub fn get_mouse_y(&self) -> f32 {
        let letterbox = self.canvas.read().view.letterbox();
        (self.context.mouse_y as f32 - letterbox.3) / letterbox.1 - self.canvas.read().view.origin().y
    }

    /// Returns the mouse position as a `Vec2`, adjusted for the current view transform and letterboxing.
    pub fn get_mouse_pos(&self) -> Vec2 {
        Vec2::new(self.get_mouse_x(), self.get_mouse_y())
    }

    /// Returns the raw mouse X position in physical screen pixels, with no view transform applied.
    pub fn get_raw_mouse_x(&self) -> f32 {
        self.context.mouse_x as f32
    }

    /// Returns the raw mouse Y position in physical screen pixels, with no view transform applied.
    pub fn get_raw_mouse_y(&self) -> f32 {
        self.context.mouse_y as f32
    }

    /// Returns the raw mouse position as a `Vec2` in physical screen pixels, with no view transform applied.
    pub fn get_raw_mouse_pos(&self) -> Vec2 {
        Vec2::new(self.get_raw_mouse_x(), self.get_raw_mouse_y())
    }

    /// Returns whether the window is currently focused.
    pub fn is_focused(&self) -> bool {
        self.context.focused
    }

    /// Get the title of this window
    pub fn get_title(&mut self) -> String {
        self.inner_window.title()
    }

    /// Set the title of this window
    pub fn set_title(&mut self, title: impl ToString) {
        self.inner_window.set_title(&title.to_string());
    }
}

impl RenderSurface for Window {
    fn background(&mut self, color: Color) {
        self.canvas.write().background(color);
    }

    fn fill(&mut self, color: Color) {
        self.canvas.write().fill(color);
    }

    fn no_fill(&mut self) {
        self.canvas.write().no_fill();
    }

    fn outline_color(&mut self, color: Color) {
        self.canvas.write().outline_color(color);
    }

    fn outline_width(&mut self, width: f32) {
        self.canvas.write().outline_width(width);
    }

    fn outline(&mut self, color: Color, width: f32) {
        self.canvas.write().outline(color, width);
    }

    fn no_outline(&mut self) {
        self.canvas.write().no_outline();
    }

    fn clear_style(&mut self) {
        self.canvas.write().clear_style();
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.canvas.write().rect(x, y, w, h);
    }

    fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, corner_radius: f32) {
        self.canvas.write().round_rect(x, y, w, h, corner_radius);
    }

    fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        self.canvas.write().ellipse(x, y, rx, ry);
    }

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.canvas.write().line(x1, y1, x2, y2);
    }

    fn image(&mut self, image: impl AsRef<Image>, x: f32, y: f32, w: f32, h: f32) {
        self.canvas.write().image(image, x, y, w, h);
    }

    fn composite(&mut self, canvas: impl AsRef<Canvas>, x: f32, y: f32, w: f32, h: f32) {
        self.canvas.write().composite(canvas, x, y, w, h);
    }

    fn horizontal_text_align(&mut self, align: HorizontalAlign) {
        self.canvas.write().horizontal_text_align(align);
    }

    fn vertical_text_align(&mut self, align: VerticalAlign) {
        self.canvas.write().vertical_text_align(align);
    }

    fn text_align(&mut self, horizontal: HorizontalAlign, vertical: VerticalAlign) {
        self.canvas.write().text_align(horizontal, vertical);
    }

    fn line_align(&mut self, align: HorizontalAlign) {
        self.canvas.write().line_align(align);
    }

    fn text_size(&mut self, size_px: f32) {
        self.canvas.write().text_size(size_px);
    }

    fn text(&mut self, font: impl AsRef<Font>, x: f32, y: f32, text: impl ToString) {
        self.canvas.write().text(font, x, y, text);
    }

    fn rich_text(&mut self, x: f32, y: f32, spans: &[Span]) {
        self.canvas.write().rich_text(x, y, spans);
    }

    fn set_view(&mut self, width: f32, height: f32, view_mode: ViewMode) {
        self.canvas.write().set_view(width, height, view_mode);
    }

    fn clear_view(&mut self) {
        self.canvas.write().clear_view();
    }

    fn set_origin(&mut self, x: f32, y: f32) {
        self.canvas.write().set_origin(x, y);
    }

    fn clear_origin(&mut self) {
        self.canvas.write().clear_origin();
    }

    fn with_style(&mut self, commands: impl FnOnce(&mut Self)) {
        let (style, text_style, view) = {
            let inner = self.canvas.read();
            (inner.style, inner.text_style, inner.view)
        };

        commands(self);

        let mut inner = self.canvas.write();
        inner.style = style;
        inner.text_style = text_style;
        inner.view.set(view);
        inner.sync_view_transform();
    }

    fn with_transform(&mut self, transform: impl AsRef<Transform2d>, commands: impl FnOnce(&mut Self)) {
        let old_local = {
            let mut inner = self.canvas.write();
            let old_local = inner.context.local_transform;
            let new_local = old_local * *transform.as_ref();

            let transform = inner.view.transform();
            inner.context.local_transform = new_local;
            inner.context.update_transform(transform * new_local);

            old_local
        };
        commands(self);

        let mut inner = self.canvas.write();
        inner.context.local_transform = old_local;

        let transform = inner.view.transform();
        inner.context.update_transform(transform * old_local);
    }

    fn flush(&mut self) -> RendererResult<()> {
        let mut encoder = self.gpu_context.device.create_command_encoder(&Default::default());
        let Some(frame) = self.get_frame() else { return Ok(()) };

        {
            let mut root_canvas = self.canvas.write();
            root_canvas.flush_with_encoder(&mut encoder, self.gpu_context.clone(), &mut HashSet::new(), self.config.format)?;
        }

        let Some(canvas_texture) = self.canvas.read().get_texture() else { return Ok(()) };

        encoder.copy_texture_to_texture(
            canvas_texture.as_image_copy(),
            frame.texture.as_image_copy(),
            Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            }
        );

        self.gpu_context.queue.submit([encoder.finish()]);
        frame.present();

        self.inner_window.request_redraw();

        Ok(())
    }
}
