use std::{collections::HashMap, mem::take, ops::Range, sync::Arc};

use bytemuck::cast_slice;
use wgpu::{BindGroup, Buffer, BufferDescriptor, BufferUsages, CurrentSurfaceTexture, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration, SurfaceTexture};
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::{GpuContext, RendererResult, Vertex, image::Image, ortho, shape_vertices::{ellipse_vertices, line_vertices, rect_vertices, textured_vertices}, shapes::Style, text::{self, Font, HorizontalAlign, Span, VerticalAlign}, transform::{GpuTransform2d, Transform2d}, types::Color, vec::Vec2, view::{View, ViewMode}};

// TODO: the ability to toggle if you want stroke scaling or not with views/transforms

#[derive(Debug, Clone)]
pub(crate) struct DrawBatch {
    pub transform: Transform2d,
    pub texture: Option<Image>,
    pub range: Range<u32>,
}

#[derive(Debug, Default)]
pub(crate) struct WindowContext {
    pub mouse_x: f64,
    pub mouse_y: f64,

    pub focused: bool,

    pub vertices: Vec<Vertex>,
    pub transform: Transform2d,
    pub local_transform: Transform2d,
    pub batches: Vec<DrawBatch>,
    pub current_texture: Option<Image>,

    pub clear_color: Option<Color>,
}

impl WindowContext {
    pub(crate) fn update_batch(&mut self) {
        let start = if let Some(group) = self.batches.last() {
            group.range.end
        } else {
            0
        };

        if self.vertices.len() as u32 != start {
            self.batches.push(DrawBatch {
                transform: self.transform,
                texture: self.current_texture.clone(),
                range: start..self.vertices.len() as u32,
            });
        }
    }

    pub(crate) fn update_texture(&mut self, texture: Option<Image>) {
        if self.current_texture != texture {
            self.update_batch();
            self.current_texture = texture;
        }
    }

    pub(crate) fn update_transform(&mut self, transform: Transform2d) {
        self.update_batch();
        self.transform = transform;
    }
}

#[derive(Debug, Clone, Copy)]
struct TextStyle {
    size: f32,
    horizontal_align: HorizontalAlign,
    vertical_align: VerticalAlign,
    line_align: HorizontalAlign,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.,
            horizontal_align: HorizontalAlign::default(),
            vertical_align: VerticalAlign::default(),
            line_align: HorizontalAlign::default(),
        }
    }
}

pub struct Window {
    pub(crate) inner_window: Arc<Box<dyn winit::window::Window>>,

    surface: Surface<'static>,
    config: SurfaceConfiguration,

    projection_buffer: Buffer,
    projection_group: BindGroup,

    transform_buffer: Buffer,
    transform_group: BindGroup,
    transform_buffer_capacity: u64,

    vertex_buffer: Buffer,
    vertex_buffer_size: u64,

    gpu_context: Arc<GpuContext>,
    context: WindowContext,
    style: Style,
    text_style: TextStyle,

    view: View,
}

impl Window {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        inner_window: Arc<Box<dyn winit::window::Window>>,

        surface: Surface<'static>,
        config: SurfaceConfiguration,

        projection_buffer: Buffer,
        projection_group: BindGroup,

        transform_buffer: Buffer,
        transform_group: BindGroup,

        vertex_buffer: Buffer,

        gpu_context: Arc<GpuContext>,
    ) -> Self {
        Self {
            inner_window,

            surface,
            config,

            projection_buffer,
            projection_group,

            transform_buffer,
            transform_group,
            transform_buffer_capacity: 0,

            vertex_buffer,
            vertex_buffer_size: 0,

            gpu_context,
            context: WindowContext::default(),
            style: Style::default(),
            text_style: TextStyle::default(),
            view: View::default(),
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

        self.view.set_window_size(Vec2::new(size.width as f32, size.height as f32), &mut self.context);

        let projection: GpuTransform2d = ortho(size.width as f32, size.height as f32).into();
        self.gpu_context.queue.write_buffer(&self.projection_buffer, 0, cast_slice(&[projection]));
    }

    pub(crate) fn on_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.context.mouse_x = position.x;
        self.context.mouse_y = position.y;
    }

    pub(crate) fn on_focus_update(&mut self, focused: bool) {
        self.context.focused = focused;
    }

    // troll function
    pub(crate) fn push_vertices<const N: usize>(&mut self, vertices: [Vertex; N]) {
        self.context.vertices.extend(vertices);
    }

    /// Returns the current width of the window in pixels.
    pub fn get_width(&self) -> f32 {
        self.view.window_size().x
    }

    /// Returns the current height of the window in pixels.
    pub fn get_height(&self) -> f32 {
        self.view.window_size().y
    }

    /// Returns the current size of the window as `(width, height)` in pixels.
    pub fn get_size(&self) -> Vec2 {
        self.view.window_size()
    }

    /// Returns the mouse X position, adjusted for the current view transform and letterboxing.
    pub fn get_mouse_x(&self) -> f32 {
        let letterbox = self.view.letterbox();
        (self.context.mouse_x as f32 - letterbox.2) / letterbox.0 - self.view.origin().x
    }

    /// Returns the mouse Y position, adjusted for the current view transform and letterboxing.
    pub fn get_mouse_y(&self) -> f32 {
        let letterbox = self.view.letterbox();
        (self.context.mouse_y as f32 - letterbox.3) / letterbox.1 - self.view.origin().y
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

    /// Clears the window to the given color at the start of each frame.
    /// Any vertices queued before this call are discarded, since they would be covered by the clear.
    // TODO: this is *meant* to work by only clearing when you call it,
    //       and keeping the current frame if you don't, processing style
    //       however, due to how wgpu works, right now if you don't clear every frame, it freaks out
    //       so i have to implement Canvas before it'll work right
    pub fn background(&mut self, color: Color) {
        // dont render things that are just getting cleared
        self.context.vertices.clear();
        self.context.clear_color = Some(color);
    }

    /// Sets the fill color for subsequent shapes.
    pub fn fill(&mut self, color: Color) {
        self.style.fill_color = color;
    }

    /// Disables fill for subsequent shapes.
    pub fn no_fill(&mut self) {
        self.style.fill_color = Color::TRANSPARENT;
    }

    /// Sets the outline color for subsequent shapes.
    pub fn outline_color(&mut self, color: Color) {
        self.style.outline_color = color;
    }

    /// Sets the outline width for subsequent shapes.
    pub fn outline_width(&mut self, width: f32) {
        self.style.outline_width = width;
    }

    /// Sets the outline color and width for subsequent shapes.
    pub fn outline(&mut self, color: Color, width: f32) {
        self.style.outline_color = color;
        self.style.outline_width = width;
    }

    /// Disables the outline for subsequent shapes.
    pub fn no_outline(&mut self) {
        self.style.outline_color = Color::TRANSPARENT;
        self.style.outline_width = 0.;
    }

    /// Draws a rectangle at `(x, y)` with the given width and height,
    /// using the current fill and outline style.
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.push_vertices(rect_vertices(
            x,
            y,
            w,
            h,
            self.style.fill_color,
            self.style.outline_color,
            self.style.outline_width,
            0.,
        ))
    }

    /// Draws a rounded rectangle at `(x, y)` with the given width, height, and corner radius,
    /// using the current fill and outline style.
    pub fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, corner_radius: f32) {
        self.push_vertices(rect_vertices(
            x,
            y,
            w,
            h,
            self.style.fill_color,
            self.style.outline_color,
            self.style.outline_width,
            corner_radius,
        ))
    }

    /// Draws an ellipse centered at `(x, y)` with horizontal radius `rx` and vertical radius `ry`,
    /// using the current fill and outline style.
    pub fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        self.push_vertices(ellipse_vertices(
            x,
            y,
            rx,
            ry,
            self.style.fill_color,
            self.style.outline_color,
            self.style.outline_width
        ))
    }

    /// Draws a line from `(x1, y1)` to `(x2, y2)` using the current outline color and width.
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.push_vertices(line_vertices(
            Vec2::new(x1, y1),
            Vec2::new(x2, y2),
            self.style.outline_color,
            self.style.outline_width
        ))
    }

    /// Draws an image at `(x, y)` with the given width and height using the current fill color.
    pub fn image(&mut self, image: &Image, x: f32, y: f32, w: f32, h: f32) {
        self.context.update_texture(Some(image.clone()));

        self.push_vertices(textured_vertices(
            x,
            y,
            w,
            h,
            Vec2::ZERO,
            Vec2::ONE,
            self.style.fill_color
        ));

        self.context.update_texture(None);
    }

    /// Sets the horizontal text alignment for subsequent text calls.
    /// Effects rich text.
    pub fn horizontal_text_align(&mut self, horizontal: HorizontalAlign) {
        self.text_style.horizontal_align = horizontal;
    }

    /// Sets the vertical text alignment for subsequent text calls.
    /// Effects rich text.
    pub fn vertical_text_align(&mut self, vertical: VerticalAlign) {
        self.text_style.vertical_align = vertical;
    }

    /// Sets the text alignment for subsequent text calls.
    /// Effects rich text.
    pub fn text_align(&mut self, horizontal: HorizontalAlign, vertical: VerticalAlign) {
        self.text_style.horizontal_align = horizontal;
        self.text_style.vertical_align = vertical;
    }

    /// Sets the alignment per-line for subsequent text calls.
    /// Effects rich text.
    pub fn line_align(&mut self, align: HorizontalAlign) {
        self.text_style.line_align = align;
    }

    /// Draws rich text at `(x, y)` with each span's font and style.
    pub fn rich_text(&mut self, x: f32, y: f32, spans: &[Span]) {
        // because the hash of a `Font` is just the `Arc` pointer, this is fine
        #[allow(clippy::mutable_key_type)]
        let mut fonts = HashMap::new();
        let mut line_heights = Vec::new();

        let mut total_width = 0.;
        let mut line_width = 0.;
        let mut line_widths = Vec::new();
        for span in spans {
            let key = (span.style, span.font.clone());
            let glyphs: &mut HashMap<_, _> = fonts.entry(key).or_default();

            let mut cx = 0.;
            let mut width: f32 = 0.;
            let mut line_height: f32 = 0.;

            let mut retries = 0;
            'outer: loop {
                // TODO: probably put a warning here that the text was too big to fit in the atlas
                //       i want to make a proper error handling/signaling system first,
                //       that's why i'm not doing it now
                if retries > 1 {
                    break;
                }

                for char in span.text.chars() {
                    let Ok(Some(glyph)) = span.font.get_or_load_glyph(char, span.style.size) else {
                        glyphs.clear();
                        retries += 1;
                        continue 'outer;
                    };

                    cx += glyph.advance;
                    line_width += glyph.advance;
                    if char == '\n' {
                        line_widths.push(line_width);
                        line_width = 0.;
                        cx = 0.;
                    } else {
                        line_height = line_height.max(glyph.height);
                    }

                    width = width.max(cx);

                    glyphs.insert(char, glyph);
                }
                break;
            }

            total_width += width;
            line_heights.push(line_height);
        }
        line_widths.push(line_width);

        let total_x_offset = match self.text_style.horizontal_align {
            HorizontalAlign::Left => 0.,
            HorizontalAlign::Center => -total_width / 2.,
            HorizontalAlign::Right => -total_width,
        };

        let mut line = 0;
        let mut x_offset = match self.text_style.line_align {
            HorizontalAlign::Left => 0.,
            HorizontalAlign::Center => (total_width - line_widths[line]) / 2.,
            HorizontalAlign::Right => total_width - line_widths[line],
        };

        let mut cx = x;
        let mut cy = y;
        for (span, line_height) in spans.iter().zip(line_heights) {
            self.context.update_texture(Some(span.font.atlas().clone()));

            let Some(glyphs) = fonts.get(&(span.style, span.font.clone())) else { continue };

            let y_offset = match self.text_style.vertical_align {
                VerticalAlign::Bottom => 0.,
                VerticalAlign::Center => line_height / 2.,
                VerticalAlign::Top => line_height,
            };

            for char in span.text.chars() {
                if char == '\n' {
                    if let Some(glyph) = glyphs.get(&char) {
                        cx = x;
                        cy += glyph.height;
                    }

                    line += 1;
                    x_offset = match self.text_style.line_align {
                        HorizontalAlign::Left => 0.,
                        HorizontalAlign::Center => (total_width - line_widths[line]) / 2.,
                        HorizontalAlign::Right => total_width - line_widths[line],
                    };

                    continue;
                }

                if char.is_whitespace() {
                    if let Some(glyph) = glyphs.get(&char) {
                        cx += glyph.advance;
                    }
                    continue;
                }

                let Some(glyph) = glyphs.get(&char) else { continue };

                let char_x = cx + glyph.xmin + span.style.offset.x + total_x_offset + x_offset;
                let char_y = cy - glyph.height - glyph.ymin + span.style.offset.y + y_offset;
                let w = glyph.width;
                let h = glyph.height;

                self.push_vertices(textured_vertices(
                    char_x,
                    char_y,
                    w,
                    h,
                    glyph.uv_min,
                    glyph.uv_max,
                    span.style.color,
                ));

                cx += glyph.advance;
            }
        }

        self.context.update_texture(None);
    }

    /// Sets the text size (in pixels) for subsequent text calls.
    /// Does not effect rich text.
    pub fn text_size(&mut self, size_px: f32) {
        self.text_style.size = size_px;
    }

    /// Draws text at `(x, y)` with the given font using the current fill color and text size.
    pub fn text(&mut self, font: impl AsRef<Font>, x: f32, y: f32, text: impl ToString) {
        self.rich_text(x, y, &[Span {
            text: text.to_string(),
            font: font.as_ref().clone(),
            style: text::TextStyle {
                size: self.text_style.size,
                color: self.style.fill_color,
                ..Default::default()
            },
        }]);
    }

    /// Sets the logical view size and scaling mode.
    pub fn set_view(&mut self, width: f32, height: f32, view_mode: ViewMode) {
        self.view.set_size(Some(Vec2::new(width, height)), &mut self.context);
        self.view.set_mode(view_mode, &mut self.context);
    }

    /// Clears the logical view size and resets the scaling mode to `Stretch`.
    pub fn clear_view(&mut self) {
        self.view.set_size(None, &mut self.context);
        self.view.set_mode(ViewMode::Stretch, &mut self.context);
    }

    /// Offsets the view origin, shifting where `(0, 0)` appears on screen.
    pub fn set_origin(&mut self, x: f32, y: f32) {
        self.view.set_origin(Vec2 { x, y }, &mut self.context);
    }

    /// Temporarily isolates style and view state for the duration of `commands`.
    /// Any changes to style or view made inside will be reverted when it returns.
    pub fn with_style(&mut self, commands: impl FnOnce(&mut Self)) {
        let style = self.style;
        let text_style = self.text_style;
        let view = self.view;

        commands(self);

        self.style = style;
        self.text_style = text_style;
        self.view.set(view, &mut self.context);
    }

    /// Applies `transform` relative to the current transform for the duration of `commands`.
    // TODO: i spent a bit of time getting these transforms rendering on the GPU, but honestly,
    //       it's probably not worth it unless the group is large
    //       we should implement smart batching, if the number of vertices per transform is enough,
    //       do the matrix math on the GPU, otherwise, do it on the CPU to save on draw calls
    pub fn with_transform(&mut self, transform: impl AsRef<Transform2d>, commands: impl FnOnce(&mut Self)) {
        let old_local = self.context.local_transform;
        let new_local = old_local * *transform.as_ref();

        self.context.local_transform = new_local;
        self.context.update_transform(self.view.transform() * new_local);
        commands(self);

        self.context.local_transform = old_local;
        self.context.update_transform(self.view.transform() * old_local);
    }

    /// Get the title of this window
    pub fn get_title(&mut self) -> String {
        self.inner_window.title()
    }

    /// Set the title of this window
    pub fn set_title(&mut self, title: impl ToString) {
        self.inner_window.set_title(&title.to_string());
    }

    /// Submits all queued draw calls to the GPU and presents the frame.
    pub fn flush(&mut self) -> RendererResult<()> {
        let mut encoder = self.gpu_context.device.create_command_encoder(&Default::default());
        let Some(frame) = self.get_frame() else { return Ok(()) };

        let load = if let Some(color) = take(&mut self.context.clear_color) {
            LoadOp::Clear(wgpu::Color::from(color.premultiplied().to_srgb()))
        } else {
            LoadOp::Load
        };

        let view = frame.texture.create_view(&Default::default());

        let vertices = take(&mut self.context.vertices);

        // TODO: this should work with powers of two, or like how `Vec`s scale
        let required = (vertices.len() * size_of::<Vertex>()) as u64;

        if required > self.vertex_buffer_size {
            self.vertex_buffer = self.gpu_context.device.create_buffer(&BufferDescriptor {
                label: Some("vertex buffer"),
                size: required,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_size = required;
        }

        self.gpu_context.queue.write_buffer(&self.vertex_buffer, 0, cast_slice(&vertices));

        let mut batches = take(&mut self.context.batches);

        let last_group_end = if let Some(group) = batches.last() {
            group.range.end
        } else {
            0
        };

        if last_group_end < vertices.len() as u32 {
            batches.push(DrawBatch {
                transform: self.context.transform,
                texture: self.context.current_texture.clone(),
                range: last_group_end..vertices.len() as u32,
            });
        }

        let required_transforms = batches.len() as u64;

        if required_transforms > self.transform_buffer_capacity {
            (self.transform_buffer, self.transform_group) = self.gpu_context.create_transform_buffer(required_transforms);
            self.transform_buffer_capacity = required_transforms;
        }

        let transforms: Vec<GpuTransform2d> = batches
            .iter()
            .map(|group| group.transform.into())
            .collect();

        let transform_stride = self.gpu_context.get_transform_stride();

        for (i, transform) in transforms.iter().enumerate() {
            self.gpu_context.queue.write_buffer(
                &self.transform_buffer,
                i as u64 * transform_stride,
                cast_slice(&[*transform])
            );
        }

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations { load, store: StoreOp::Store }
                })],
                ..Default::default()
            });

            pass.set_pipeline(&self.gpu_context.pipeline);

            if self.vertex_buffer_size > 0 {
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                for (i, batch) in batches.iter_mut().enumerate() {
                    let offset = (i as u64 * transform_stride) as u32;
                    pass.set_bind_group(0, Some(&self.projection_group), &[]);
                    pass.set_bind_group(1, Some(&self.transform_group), &[offset]);

                    if let Some(texture) = batch.texture.as_mut() {
                        let data = texture.submit_to_gpu(&self.gpu_context)?;
                        pass.set_bind_group(2, Some(&data.bind_group), &[]);
                    } else {
                        pass.set_bind_group(2, Some(&self.gpu_context.dummy_bind_group), &[]);
                    }

                    pass.draw(batch.range.start..batch.range.end, 0..1);
                }
            }
        }

        self.gpu_context.queue.submit([encoder.finish()]);
        frame.present();

        self.inner_window.request_redraw();

        Ok(())
    }
}
