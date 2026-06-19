use std::{collections::HashSet, sync::{Arc, RwLockWriteGuard, atomic::{AtomicUsize, Ordering}}};

use wgpu::{CurrentSurfaceTexture, Extent3d, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration, SurfaceTexture};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, monitor::Fullscreen, window::WindowLevel};

use crate::{AdvancedWindowProperties, GpuContext, Renderer, RendererResult, canvas::{Canvas, CanvasDraw}, image::Image, render_surface::RenderSurface, shapes::ScalingMode, text::{Font, HorizontalAlign, Span, TextLayout, VerticalAlign}, transform::Transform2d, types::Color, vec::Vec2, view::ViewMode};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) usize);

impl WindowId {
    pub(crate) fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

// TODO: window should expose more mouse and input functions

// TODO: so i expose IME events, but currently there's no way to actually enable IME

// TODO: width and height are *physical*, not logical right now
//       DPI scaling will mess this up later

// TODO: you cannot disable vsync
#[derive(Debug, Clone)]
pub struct WindowProperties {
    /// The title of the window.
    pub title: String,

    /// The width of the window in pixels.
    pub width: u32,

    /// The height of the window in pixels.
    pub height: u32,

    /// Whether the window can be resized by the user.
    pub resizable: bool,

    /// Whether the window background is transparent (on platforms that support it).
    pub transparent: bool,

    /// Whether the window should launch in borderless fullscreen mode.
    pub fullscreen: bool,

    /// Whether the window should launch maximized.
    pub maximized: bool,

    /// Whether the window should always render on top of other windows.
    pub always_on_top: bool,
}

impl WindowProperties {
    /// Create a new [`WindowProperties`] with a `title`, `width`, and `height`.
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            ..Default::default()
        }
    }

    /// Set the title of the window.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the width of the window.
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Set the height of the window.
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Set the size of the window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set whether the window background is transparent (on platforms that support it).
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Set whether the window should launch in borderless fullscreen mode.
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Set whether the window should launch maximized.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Set whether the window should always render on top of other windows.
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }

    pub fn build(self, renderer: &mut Renderer) -> WindowId {
        renderer.create_window_ext(self)
    }
}

impl From<WindowProperties> for AdvancedWindowProperties {
    fn from(props: WindowProperties) -> Self {
        let mut attributes = AdvancedWindowProperties::default()
            .with_title(props.title)
            .with_surface_size(PhysicalSize::new(props.width, props.height))
            .with_resizable(props.resizable)
            .with_transparent(props.transparent)
            .with_maximized(props.maximized);

        if props.fullscreen {
            attributes = attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        if props.always_on_top {
            attributes = attributes.with_window_level(WindowLevel::AlwaysOnTop);
        }

        attributes
    }
}

impl Default for WindowProperties {
    fn default() -> Self {
        Self {
            title: "verdant window".into(),
            width: 800,
            height: 600,
            resizable: false,
            transparent: false,
            fullscreen: false,
            maximized: false,
            always_on_top: false,
        }
    }
}

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
        let Some(frame) = (0..3).find_map(|_| self.get_frame()) else {
            return Ok(());
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

    pub(crate) fn draw<'a>(&'a self) -> WindowDraw<'a> {
        WindowDraw {
            canvas: self.canvas.write(),
            window: self,
        }
    }
}

pub struct WindowDraw<'a> {
    canvas: RwLockWriteGuard<'a, CanvasDraw>,
    window: &'a Window,
}

impl<'a> WindowDraw<'a> {
    /// Returns the current width of the window in pixels.
    pub fn get_width(&self) -> f32 {
        self.canvas.view.window_size().x
    }

    /// Returns the current height of the window in pixels.
    pub fn get_height(&self) -> f32 {
        self.canvas.view.window_size().y
    }

    /// Returns the current size of the window as `(width, height)` in pixels.
    pub fn get_size(&self) -> Vec2 {
        self.canvas.view.window_size()
    }

    /// Returns the mouse X position, adjusted for the current view transform and letterboxing.
    pub fn get_mouse_x(&self) -> f32 {
        let letterbox = self.canvas.view.letterbox();
        (self.window.context.mouse_x as f32 - letterbox.2) / letterbox.0 - self.canvas.view.origin().x
    }

    /// Returns the mouse Y position, adjusted for the current view transform and letterboxing.
    pub fn get_mouse_y(&self) -> f32 {
        let letterbox = self.canvas.view.letterbox();
        (self.window.context.mouse_y as f32 - letterbox.3) / letterbox.1 - self.canvas.view.origin().y
    }

    /// Returns the mouse position as a `Vec2`, adjusted for the current view transform and letterboxing.
    pub fn get_mouse_pos(&self) -> Vec2 {
        Vec2::new(self.get_mouse_x(), self.get_mouse_y())
    }

    /// Returns the raw mouse X position in physical screen pixels, with no view transform applied.
    pub fn get_raw_mouse_x(&self) -> f32 {
        self.window.context.mouse_x as f32
    }

    /// Returns the raw mouse Y position in physical screen pixels, with no view transform applied.
    pub fn get_raw_mouse_y(&self) -> f32 {
        self.window.context.mouse_y as f32
    }

    /// Returns the raw mouse position as a `Vec2` in physical screen pixels, with no view transform applied.
    pub fn get_raw_mouse_pos(&self) -> Vec2 {
        Vec2::new(self.get_raw_mouse_x(), self.get_raw_mouse_y())
    }

    /// Returns whether the window is currently focused.
    pub fn is_focused(&self) -> bool {
        self.window.context.focused
    }

    /// Get the title of this window
    pub fn get_title(&mut self) -> String {
        self.window.inner_window.title()
    }

    /// Set the title of this window
    pub fn set_title(&mut self, title: impl ToString) {
        self.window.inner_window.set_title(&title.to_string());
    }

    /// Get the inner canvas of this window
    pub fn get_canvas(&self) -> Canvas {
        self.window.canvas.clone()
    }
}

impl<'a> RenderSurface for WindowDraw<'a> {
    fn background(&mut self, color: Color) {
        self.canvas.background(color);
    }

    fn fill(&mut self, color: Color) {
        self.canvas.fill(color);
    }

    fn no_fill(&mut self) {
        self.canvas.no_fill();
    }

    fn outline_color(&mut self, color: Color) {
        self.canvas.outline_color(color);
    }

    fn outline_width(&mut self, width: f32) {
        self.canvas.outline_width(width);
    }

    fn outline(&mut self, color: Color, width: f32) {
        self.canvas.outline(color, width);
    }

    fn outline_style(&mut self, color: Color, width: f32, scaling: ScalingMode) {
        self.canvas.outline_style(color, width, scaling);
    }

    fn no_outline(&mut self) {
        self.canvas.no_outline();
    }

    fn outline_scaling(&mut self, scaling: ScalingMode) {
        self.canvas.outline_scaling(scaling);
    }

    fn corner_radius(&mut self, radius: f32) {
        self.canvas.corner_radius(radius);
    }

    fn corner_scaling(&mut self, scaling: ScalingMode) {
        self.canvas.corner_scaling(scaling);
    }

    fn corner_style(&mut self, radius: f32, scaling: ScalingMode) {
        self.canvas.corner_style(radius, scaling);
    }

    fn scaling_modes(&mut self, outline_scaling: ScalingMode, corner_scaling: ScalingMode) {
        self.canvas.scaling_modes(outline_scaling, corner_scaling);
    }

    fn clear_style(&mut self) {
        self.canvas.clear_style();
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.canvas.rect(x, y, w, h);
    }

    fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        self.canvas.ellipse(x, y, rx, ry);
    }

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.canvas.line(x1, y1, x2, y2);
    }

    fn image(&mut self, image: impl AsRef<Image>, x: f32, y: f32, w: f32, h: f32) {
        self.canvas.image(image, x, y, w, h);
    }

    fn composite(&mut self, canvas: impl AsRef<Canvas>, x: f32, y: f32, w: f32, h: f32) {
        self.canvas.composite(canvas, x, y, w, h);
    }

    fn horizontal_text_align(&mut self, align: HorizontalAlign) {
        self.canvas.horizontal_text_align(align);
    }

    fn vertical_text_align(&mut self, align: VerticalAlign) {
        self.canvas.vertical_text_align(align);
    }

    fn text_align(&mut self, horizontal: HorizontalAlign, vertical: VerticalAlign) {
        self.canvas.text_align(horizontal, vertical);
    }

    fn line_align(&mut self, align: HorizontalAlign) {
        self.canvas.line_align(align);
    }

    fn font_size(&mut self, size_px: f32) {
        self.canvas.font_size(size_px);
    }

    fn text(&mut self, font: impl AsRef<Font>, x: f32, y: f32, text: impl ToString) {
        self.canvas.text(font, x, y, text);
    }

    fn rich_text(&mut self, x: f32, y: f32, spans: &[Span]) {
        self.canvas.rich_text(x, y, spans);
    }

    fn text_layout(&mut self, font: impl AsRef<Font>, text: impl ToString) -> TextLayout {
        self.canvas.text_layout(font, text)
    }

    fn text_size(&mut self, font: impl AsRef<Font>, text: impl ToString) -> Vec2 {
        self.canvas.text_size(font, text)
    }

    fn text_width(&mut self, font: impl AsRef<Font>, text: impl ToString) -> f32 {
        self.canvas.text_width(font, text)
    }

    fn text_height(&mut self, font: impl AsRef<Font>, text: impl ToString) -> f32 {
        self.canvas.text_height(font, text)
    }

    fn rich_text_layout(&mut self, spans: &[Span]) -> TextLayout {
        self.canvas.rich_text_layout(spans)
    }

    fn rich_text_size(&mut self, spans: &[Span]) -> Vec2 {
        self.canvas.rich_text_size(spans)
    }

    fn rich_text_width(&mut self, spans: &[Span]) -> f32 {
        self.canvas.rich_text_width(spans)
    }

    fn rich_text_height(&mut self, spans: &[Span]) -> f32 {
        self.canvas.rich_text_height(spans)
    }

    fn set_view(&mut self, width: f32, height: f32, view_mode: ViewMode) {
        self.canvas.set_view(width, height, view_mode);
    }

    fn clear_view(&mut self) {
        self.canvas.clear_view();
    }

    fn set_origin(&mut self, x: f32, y: f32) {
        self.canvas.set_origin(x, y);
    }

    fn clear_origin(&mut self) {
        self.canvas.clear_origin();
    }

    fn with_style(&mut self, commands: impl FnOnce(&mut Self)) {
        let (style, text_style, view) = {
            (self.canvas.style, self.canvas.text_style, self.canvas.view)
        };

        commands(self);

        self.canvas.style = style;
        self.canvas.text_style = text_style;
        self.canvas.view.set(view);
        self.canvas.sync_view_transform();
    }

    fn with_transform(&mut self, transform: impl AsRef<Transform2d>, commands: impl FnOnce(&mut Self)) {
        let old_local = {
            let old_local = self.canvas.context.local_transform;
            let new_local = old_local * *transform.as_ref();

            let transform = self.canvas.view.transform();
            self.canvas.context.local_transform = new_local;
            self.canvas.context.update_transform(transform * new_local);

            old_local
        };
        commands(self);

        self.canvas.context.local_transform = old_local;

        let transform = self.canvas.view.transform();
        self.canvas.context.update_transform(transform * old_local);
    }

    fn flush(&mut self) -> RendererResult<()> {
        let mut encoder = self.window.gpu_context.device.create_command_encoder(&Default::default());
        let Some(frame) = self.window.get_frame() else { return Ok(()) };

        {
            self.canvas.flush_with_encoder(&mut encoder, self.window.gpu_context.clone(), &mut HashSet::new(), self.window.config.format)?;
        }

        let Some(canvas_texture) = self.canvas.get_texture() else { return Ok(()) };

        encoder.copy_texture_to_texture(
            canvas_texture.as_image_copy(),
            frame.texture.as_image_copy(),
            Extent3d {
                width: self.window.config.width,
                height: self.window.config.height,
                depth_or_array_layers: 1,
            }
        );

        self.window.gpu_context.queue.submit([encoder.finish()]);
        frame.present();

        self.window.inner_window.request_redraw();

        Ok(())
    }
}
