use std::{collections::HashMap, mem::take, ops::Range, sync::Arc};

use bytemuck::cast_slice;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Extent3d, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, util::{BufferInitDescriptor, DeviceExt}};

use crate::{GpuContext, RendererResult, Vertex, image::Image, ortho, shape_vertices::{ellipse_vertices, line_vertices, rect_vertices, textured_vertices}, shapes::Style, text::{self, Font, HorizontalAlign, Span, VerticalAlign}, transform::{GpuTransform2d, Transform2d}, types::Color, vec::Vec2, view::{View, ViewMode}};

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

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextStyle {
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

#[derive(Debug, Clone)]
struct DrawBatch {
    pub texture: Option<Image>,
    pub range: Range<u32>,
}

#[derive(Debug, Default)]
pub(crate) struct CanvasContext {
    pub(crate) local_transform: Transform2d,

    vertices: Vec<Vertex>,
    transform: Transform2d,
    batches: Vec<DrawBatch>,
    current_texture: Option<Image>,

    clear_color: Option<Color>,
}

impl CanvasContext {
    pub(crate) fn update_batch(&mut self) {
        let start = if let Some(group) = self.batches.last() {
            group.range.end
        } else {
            0
        };

        if self.vertices.len() as u32 != start {
            self.batches.push(DrawBatch {
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
        self.transform = transform;
    }
}

fn create_texture(width: u32, height: u32, format: TextureFormat, gpu_context: &GpuContext) -> (Texture, TextureView) {
    let texture = gpu_context.device.create_texture(&TextureDescriptor {
        label: Some("canvas texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());

    (texture, view)
}

#[derive(Debug)]
pub struct Canvas {
    pub(crate) context: CanvasContext,

    pub(crate) style: Style,
    pub(crate) text_style: TextStyle,
    pub(crate) view: View,

    pub(crate) texture: Texture,
    texture_view: TextureView,

    projection_buffer: Buffer,
    projection_group: BindGroup,

    vertex_buffer: Buffer,
    vertex_buffer_size: u64,

    gpu_context: Arc<GpuContext>,
}

impl Canvas {
    pub(crate) fn new(
        width: u32,
        height: u32,
        format: TextureFormat,
        gpu_context: Arc<GpuContext>,
    ) -> Self {
        let (texture, texture_view) = create_texture(width, height, format, &gpu_context);

        let projection = ortho(width as f32, height as f32);
        let projection_buffer = gpu_context.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("projection"),
            contents: cast_slice(&[GpuTransform2d::from(projection)]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let projection_group = gpu_context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("projection bind group"),
            layout: &gpu_context.projection_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
        });

        let vertex_buffer = gpu_context.device.create_buffer(&BufferDescriptor {
            label: Some("vertex buffer"),
            size: 0,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            texture,
            texture_view,

            projection_buffer,
            projection_group,

            vertex_buffer,
            vertex_buffer_size: 0,

            context: CanvasContext::default(),
            gpu_context,

            style: Style::default(),
            text_style: TextStyle::default(),
            view: View::default(),
        }
    }

    fn push_vertices<const N: usize>(&mut self, mut vertices: [Vertex; N]) {
        for v in &mut vertices {
            v.position = self.context.transform.transform_point(v.position);
        }
        self.context.vertices.extend(vertices);
    }

    pub(crate) fn flush_with_encoder(&mut self, encoder: &mut CommandEncoder) -> RendererResult<()> {
        let load = if let Some(color) = take(&mut self.context.clear_color) {
            LoadOp::Clear(wgpu::Color::from(color))
        } else {
            LoadOp::Load
        };

        let vertices = take(&mut self.context.vertices);

        let required = (vertices.len() * size_of::<Vertex>()) as u64;
        while required > self.vertex_buffer_size {
            if self.vertex_buffer_size == 0 {
                self.vertex_buffer_size = required;
            } else {
                self.vertex_buffer_size *= 2;
            }
            self.vertex_buffer = self.gpu_context.device.create_buffer(&BufferDescriptor {
                label: Some("vertex buffer"),
                size: self.vertex_buffer_size,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
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
                texture: self.context.current_texture.clone(),
                range: last_group_end..vertices.len() as u32,
            });
        }

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations { load, store: StoreOp::Store },
                })],
                ..Default::default()
            });

            pass.set_pipeline(&self.gpu_context.pipeline);

            if self.vertex_buffer_size > 0 {
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                for batch in batches.iter_mut() {
                    pass.set_bind_group(0, Some(&self.projection_group), &[]);

                    if let Some(texture) = batch.texture.as_mut() {
                        let data = texture.submit_to_gpu(&self.gpu_context)?;
                        pass.set_bind_group(1, Some(&data.bind_group), &[]);
                    } else {
                        pass.set_bind_group(1, Some(&self.gpu_context.dummy_bind_group), &[]);
                    }

                    pass.draw(batch.range.clone(), 0..1);
                }
            }
        }

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let (texture, texture_view) = create_texture(
            width,
            height,
            self.texture.format(),
            &self.gpu_context
        );
        self.texture = texture;
        self.texture_view = texture_view;

        self.view.set_window_size(Vec2::new(width as f32, height as f32), &mut self.context);

        let projection: GpuTransform2d = ortho(width as f32, height as f32).into();
        self.gpu_context.queue.write_buffer(&self.projection_buffer, 0, cast_slice(&[projection]));
    }
}

impl RenderSurface for Canvas {
    fn background(&mut self, color: Color) {
        self.context.vertices.clear();
        self.context.clear_color = Some(color);
    }

    fn fill(&mut self, color: Color) {
        self.style.fill_color = color;
    }

    fn no_fill(&mut self) {
        self.style.fill_color = Color::TRANSPARENT;
    }

    fn outline_color(&mut self, color: Color) {
        self.style.outline_color = color;
    }

    fn outline_width(&mut self, width: f32) {
        self.style.outline_width = width;
    }

    fn outline(&mut self, color: Color, width: f32) {
        self.style.outline_color = color;
        self.style.outline_width = width;
    }

    fn no_outline(&mut self) {
        self.style.outline_color = Color::TRANSPARENT;
        self.style.outline_width = 0.;
    }

    fn clear_style(&mut self) {
        self.style = Style::default();
    }

        fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
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

    fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, corner_radius: f32) {
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

    fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
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

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.push_vertices(line_vertices(
            Vec2::new(x1, y1),
            Vec2::new(x2, y2),
            self.style.outline_color,
            self.style.outline_width
        ))
    }

    fn image(&mut self, image: impl AsRef<Image>, x: f32, y: f32, w: f32, h: f32) {
        self.context.update_texture(Some(image.as_ref().clone()));

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

    fn horizontal_text_align(&mut self, align: HorizontalAlign) {
        self.text_style.horizontal_align = align;
    }

    fn vertical_text_align(&mut self, align: VerticalAlign) {
        self.text_style.vertical_align = align;
    }

    fn text_align(&mut self, horizontal: HorizontalAlign, vertical: VerticalAlign) {
        self.text_style.horizontal_align = horizontal;
        self.text_style.vertical_align = vertical;
    }

    fn line_align(&mut self, align: HorizontalAlign) {
        self.text_style.line_align = align;
    }

    fn text_size(&mut self, size_px: f32) {
        self.text_style.size = size_px;
    }

    fn text(&mut self, font: impl AsRef<Font>, x: f32, y: f32, text: impl ToString) {
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

    fn rich_text(&mut self, x: f32, y: f32, spans: &[Span]) {
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

    fn set_view(&mut self, width: f32, height: f32, view_mode: ViewMode) {
        self.view.set_size(Some(Vec2::new(width, height)), &mut self.context);
        self.view.set_mode(view_mode, &mut self.context);
    }

    fn clear_view(&mut self) {
        self.view.set_size(None, &mut self.context);
        self.view.set_mode(ViewMode::Unscaled, &mut self.context);
    }

    fn set_origin(&mut self, x: f32, y: f32) {
        self.view.set_origin(Vec2 { x, y }, &mut self.context);
    }

    fn clear_origin(&mut self) {
        self.view.set_origin(Vec2::ZERO, &mut self.context);
    }

    fn with_style(&mut self, commands: impl FnOnce(&mut Self)) {
        let style = self.style;
        let text_style = self.text_style;
        let view = self.view;

        commands(self);

        self.style = style;
        self.text_style = text_style;
        self.view.set(view, &mut self.context);
    }

    fn with_transform(&mut self, transform: impl AsRef<Transform2d>, commands: impl FnOnce(&mut Self)) {
        let old_local = self.context.local_transform;
        let new_local = old_local * *transform.as_ref();

        self.context.local_transform = new_local;
        self.context.update_transform(self.view.transform() * new_local);
        commands(self);

        self.context.local_transform = old_local;
        self.context.update_transform(self.view.transform() * old_local);
    }

    fn flush(&mut self) -> RendererResult<()> {
        let mut encoder = self.gpu_context.device.create_command_encoder(&Default::default());

        self.flush_with_encoder(&mut encoder)?;

        self.gpu_context.queue.submit([encoder.finish()]);

        Ok(())
    }
}
