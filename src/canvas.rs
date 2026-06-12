use std::{collections::{HashMap, HashSet, hash_map::Entry}, mem::take, ops::Range, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, atomic::{AtomicU64, Ordering}}};

use bytemuck::cast_slice;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Extent3d, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, util::{BufferInitDescriptor, DeviceExt}, wgt::TextureDataOrder};

use crate::{GpuContext, RendererResult, Vertex, image::Image, ortho, shape_vertices::{canvas_vertices, ellipse_vertices, line_vertices, rect_vertices, textured_vertices}, shapes::{ScalingMode, Style}, text::{self, Font, HorizontalAlign, Span, VerticalAlign}, transform::{GpuTransform2d, Transform2d}, types::Color, vec::Vec2, view::{View, ViewMode}};

static CANVAS_ID: AtomicU64 = AtomicU64::new(0);

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
    /// Disables the outline for subsequent shapes.
    fn no_outline(&mut self);
    /// Sets the outline scaling mode for subsequent shapes.
    // TODO: i feel like this is poorly documented and explained
    //       it's a useful feature, but i need to improve it
    fn outline_scaling(&mut self, mode: ScalingMode);
    /// Sets the corner radius for subsequent shapes.
    fn corner_radius(&mut self, radius: f32);
    /// Sets the corner radius scaling mode for subsequent shapes.
    fn corner_scaling(&mut self, mode: ScalingMode);
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
    /// Sets the text size (in pixels) for subsequent text calls.
    /// Does not affect rich text.
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

fn resolve_scale(value: f32, mode: ScalingMode, model_scale: f32, view_scale: f32) -> f32 {
    match mode {
        ScalingMode::Constant => value,
        ScalingMode::Transform => value * model_scale,
        ScalingMode::View => value * view_scale,
        ScalingMode::Geometric => value * model_scale * view_scale,
    }
}

#[derive(Debug, Clone)]
struct DrawBatch {
    pub texture: Option<Image>,
    pub canvas: Option<Canvas>,
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

        let len = self.vertices.len() as u32;
        if let Some(last_batch) = self.batches.last_mut() && last_batch.texture == self.current_texture {
            last_batch.range.end = len;
            return;
        }

        if self.vertices.len() as u32 != start {
            self.batches.push(DrawBatch {
                texture: self.current_texture.clone(),
                canvas: None,
                range: start..len,
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

fn create_texture(width: u32, height: u32, format: TextureFormat, gpu_context: &GpuContext, init_black: bool) -> (Texture, TextureView) {
    let desc = TextureDescriptor {
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
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture = if init_black {
        let mut data = vec![0u8; (width * height * 4) as usize];

        for chunk in data.chunks_exact_mut(4) {
            chunk[3] = 255;
        }

        gpu_context.device.create_texture_with_data(
            &gpu_context.queue,
            &desc,
            TextureDataOrder::LayerMajor,
            &data,
        )
    } else {
        gpu_context.device.create_texture(&desc)
    };

    let view = texture.create_view(&Default::default());

    (texture, view)
}

#[derive(Debug)]
struct CanvasRenderContext {
    texture: Texture,
    texture_view: TextureView,
    texture_bind_group: BindGroup,

    projection_buffer: Buffer,
    projection_group: BindGroup,

    vertex_buffer: Buffer,
    vertex_buffer_size: u64,

    scratch_texture: Option<(Texture, BindGroup)>,

    gpu_context: Arc<GpuContext>,
}

impl CanvasRenderContext {
    fn scratch_texture(&mut self, width: u32, height: u32, format: TextureFormat) -> (Texture, BindGroup) {
        if let Some(scratch) = &self.scratch_texture {
            scratch.clone()
        } else {
            let (texture, view) = create_texture(width, height, format, &self.gpu_context, false);

            let bind_group = self.gpu_context.device.create_bind_group(&BindGroupDescriptor {
                layout: &self.gpu_context.texture_group_layout,
                entries: &[
                    BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&view) },
                    BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&self.gpu_context.sampler)},
                ],
                label: Some("canvas bind group"),
            });

            let scratch = (texture, bind_group);
            self.scratch_texture = Some(scratch.clone());
            scratch
        }
    }
}

#[derive(Debug)]
pub struct CanvasState {
    pub id: u64,
    pub(crate) context: CanvasContext,

    pub(crate) style: Style,
    pub(crate) text_style: TextStyle,
    pub(crate) view: View,

    width: u32,
    height: u32,

    init_black: bool,
    render_context: Option<CanvasRenderContext>,
}

impl CanvasState {
    pub(crate) fn new(
        id: u64,
        width: u32,
        height: u32,
        init_black: bool,
    ) -> Self {
        let mut view = View::default();
        view.set_window_size(Vec2::new(width as f32, height as f32));

        let mut context = CanvasContext::default();
        context.update_transform(view.transform() * context.local_transform);

        Self {
            id,

            context,

            style: Style::default(),
            text_style: TextStyle::default(),
            view,

            width,
            height,

            init_black,
            render_context: None,
        }
    }

    pub(crate) fn init_render_context(&mut self, gpu_context: Arc<GpuContext>, format: TextureFormat) {
        if self.render_context.is_some() { return; }

        let (texture, texture_view) = create_texture(self.width, self.height, format, &gpu_context, self.init_black);

        let texture_bind_group = gpu_context.device.create_bind_group(&BindGroupDescriptor {
            layout: &gpu_context.texture_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&texture_view) },
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&gpu_context.sampler)},
            ],
            label: Some("canvas bind group"),
        });

        let projection = ortho(self.width as f32, self.height as f32);
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

        self.render_context = Some(CanvasRenderContext {
            texture,
            texture_view,
            texture_bind_group,

            projection_buffer,
            projection_group,

            vertex_buffer,
            vertex_buffer_size: 0,

            scratch_texture: None,

            gpu_context,
        });
    }

    fn push_vertices<const N: usize>(&mut self, mut vertices: [Vertex; N]) {
        for v in &mut vertices {
            v.position = self.context.transform.transform_point(v.position);
        }
        self.context.vertices.extend(vertices);
    }

    // this function name sucks because i wrote it tired and i'm bad at naming things
    fn get_scales(&self) -> (f32, f32, Vec2) {
        let model_scale = self.context.local_transform.get_safe_scale();
        let view_scale = self.view.transform().get_safe_scale();

        (
            (model_scale.x + model_scale.y) / 2.,
            (view_scale.x + view_scale.y) / 2.,
            self.context.transform.get_safe_scale(),
        )
    }

    pub(crate) fn flush_with_encoder(
        &mut self,
        encoder: &mut CommandEncoder,
        gpu_context: Arc<GpuContext>,
        flushing: &mut HashSet<u64>,
        format: TextureFormat,
    ) -> RendererResult<()> {
        if !flushing.insert(self.id) { return Ok(()) }

        self.init_render_context(gpu_context.clone(), format);

        let vertices = take(&mut self.context.vertices);
        let mut batches = take(&mut self.context.batches);

        let last_group_end = if let Some(group) = batches.last() {
            group.range.end
        } else {
            0
        };

        if last_group_end < vertices.len() as u32 {
            batches.push(DrawBatch {
                texture: self.context.current_texture.clone(),
                canvas: None,
                range: last_group_end..vertices.len() as u32,
            });
        }

        let mut child_views = HashMap::new();
        for batch in &batches {
            if let Some(child_canvas) = &batch.canvas {
                if child_canvas.id == self.id {
                    if let Entry::Vacant(e) = child_views.entry(self.id) && let Some(rc) = self.render_context.as_mut() {
                        let (texture, bind_group) = rc.scratch_texture(self.width, self.height, format);

                        encoder.copy_texture_to_texture(
                            rc.texture.as_image_copy(),
                            texture.as_image_copy(),
                            Extent3d {
                                width: self.width,
                                height: self.height,
                                depth_or_array_layers: 1
                            }
                        );
                        e.insert(bind_group);
                    }
                } else if let Entry::Vacant(e) = child_views.entry(child_canvas.id) {
                    let mut child_state = child_canvas.write();
                    child_state.flush_with_encoder(encoder, gpu_context.clone(), flushing, format)?;

                    // TODO: these `let Some(render_context)` blocks are stupid
                    //       we know for a fact they exist by this point, so the code should really be
                    //       organized in such a way that we don't have to do this check ever
                    if let Some(rc) = child_state.render_context.as_ref() {
                        e.insert(rc.texture_bind_group.clone());
                    }
                }
            }
        }

        let Some(render_context) = self.render_context.as_mut() else {
            flushing.remove(&self.id);
            return Ok(());
        };

        let load = if let Some(color) = take(&mut self.context.clear_color) {
            LoadOp::Clear(wgpu::Color::from(color.to_srgb()))
        } else {
            LoadOp::Load
        };

        let required = (vertices.len() * size_of::<Vertex>()) as u64;
        while required > render_context.vertex_buffer_size {
            if render_context.vertex_buffer_size == 0 {
                render_context.vertex_buffer_size = required;
            } else {
                render_context.vertex_buffer_size *= 2;
            }
            render_context.vertex_buffer = render_context.gpu_context.device.create_buffer(&BufferDescriptor {
                label: Some("vertex buffer"),
                size: render_context.vertex_buffer_size,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        render_context.gpu_context.queue.write_buffer(&render_context.vertex_buffer, 0, cast_slice(&vertices));

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &render_context.texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations { load, store: StoreOp::Store },
                })],
                ..Default::default()
            });

            pass.set_pipeline(&render_context.gpu_context.pipeline);

            if render_context.vertex_buffer_size > 0 {
                pass.set_vertex_buffer(0, render_context.vertex_buffer.slice(..));

                for batch in batches.iter_mut() {
                    pass.set_bind_group(0, Some(&render_context.projection_group), &[]);

                    if let Some(texture) = batch.texture.as_mut() {
                        let data = texture.submit_to_gpu(&render_context.gpu_context)?;
                        pass.set_bind_group(1, Some(&data.bind_group), &[]);
                    } else if let Some(child_canvas) = &batch.canvas {
                        if let Some(bind_group) = child_views.get(&child_canvas.id) {
                            pass.set_bind_group(1, Some(bind_group), &[]);
                        } else {
                            pass.set_bind_group(1, Some(&render_context.gpu_context.dummy_bind_group), &[]);
                        }
                    } else {
                        pass.set_bind_group(1, Some(&render_context.gpu_context.dummy_bind_group), &[]);
                    }

                    pass.draw(batch.range.clone(), 0..1);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn get_texture(&self) -> Option<Texture> {
        self.render_context.as_ref().map(|c| c.texture.clone())
    }

    pub(crate) fn sync_view_transform(&mut self) {
        self.context.update_transform(self.view.transform() * self.context.local_transform);
    }

    /// Resize this canvas using the given width and height.
    ///
    /// Note: this function clears the canvas whenever called.
    pub fn resize(&mut self, width: u32, height: u32) {
        let Some(render_context) = self.render_context.as_mut() else { return };

        let (texture, texture_view) = create_texture(
            width,
            height,
            render_context.texture.format(),
            &render_context.gpu_context,
            self.init_black,
        );
        render_context.texture = texture;
        render_context.texture_view = texture_view;

        self.view.set_window_size(Vec2::new(width as f32, height as f32));
        self.sync_view_transform();

        let Some(render_context) = self.render_context.as_mut() else { return };
        let projection: GpuTransform2d = ortho(width as f32, height as f32).into();
        render_context.gpu_context.queue.write_buffer(&render_context.projection_buffer, 0, cast_slice(&[projection]));
    }
}

impl RenderSurface for CanvasState {
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

    fn outline_scaling(&mut self, mode: ScalingMode) {
        self.style.outline_scaling = mode;
    }

    fn corner_radius(&mut self, radius: f32) {
        self.style.corner_radius = radius;
    }

    fn corner_scaling(&mut self, mode: ScalingMode) {
        self.style.corner_scaling = mode;
    }

    fn clear_style(&mut self) {
        self.style = Style::default();
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let (model_scale, view_scale, total_scale) = self.get_scales();

        let outline_width = resolve_scale(
            self.style.outline_width,
            self.style.outline_scaling,
            model_scale,
            view_scale
        );

        let corner_radius = resolve_scale(
            self.style.corner_radius,
            self.style.corner_scaling,
            model_scale,
            view_scale
        );

        self.push_vertices(rect_vertices(
            x,
            y,
            w,
            h,
            self.style.fill_color,
            self.style.outline_color,
            outline_width,
            corner_radius,
            total_scale,
        ))
    }

    fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        let (model_scale, view_scale, total_scale) = self.get_scales();

        let outline_width = resolve_scale(
            self.style.outline_width,
            self.style.outline_scaling,
            model_scale,
            view_scale
        );

        self.push_vertices(ellipse_vertices(
            x,
            y,
            rx,
            ry,
            self.style.fill_color,
            self.style.outline_color,
            outline_width,
            total_scale,
        ))
    }

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let (model_scale, view_scale, _) = self.get_scales();

        let outline_width = resolve_scale(
            self.style.outline_width,
            self.style.outline_scaling,
            model_scale,
            view_scale
        );

        self.push_vertices(line_vertices(
            Vec2::new(x1, y1),
            Vec2::new(x2, y2),
            self.style.outline_color,
            outline_width,
            self.context.transform,
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

    fn composite(&mut self, canvas: impl AsRef<Canvas>, x: f32, y: f32, w: f32, h: f32) {
        self.context.update_batch();
        let len = self.context.vertices.len() as u32;
        self.push_vertices(canvas_vertices(
            x,
            y,
            w,
            h,
            Vec2::ZERO,
            Vec2::ONE,
            self.style.fill_color
        ));
        self.context.batches.push(DrawBatch {
            texture: None,
            canvas: Some(canvas.as_ref().clone()),
            range: len..len+6,
        });
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
        self.view.set_size(Some(Vec2::new(width, height)));
        self.view.set_mode(view_mode);
        self.sync_view_transform();
    }

    fn clear_view(&mut self) {
        self.view.set_size(None);
        self.view.set_mode(ViewMode::Unscaled);
        self.sync_view_transform();
    }

    fn set_origin(&mut self, x: f32, y: f32) {
        self.view.set_origin(Vec2 { x, y });
        self.sync_view_transform();
    }

    fn clear_origin(&mut self) {
        self.view.set_origin(Vec2::ZERO);
        self.sync_view_transform();
    }

    fn with_style(&mut self, commands: impl FnOnce(&mut Self)) {
        let style = self.style;
        let text_style = self.text_style;
        let view = self.view;

        commands(self);

        self.style = style;
        self.text_style = text_style;
        self.view.set(view);
        self.sync_view_transform();
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
        let (mut encoder, gpu_context, format) = {
            let Some(render_context) = self.render_context.as_ref() else { return Ok(()) };
            (
                render_context.gpu_context.device.create_command_encoder(&Default::default()),
                render_context.gpu_context.clone(),
                render_context.texture.format(),
            )
        };
        self.flush_with_encoder(&mut encoder, gpu_context, &mut HashSet::new(), format)?;

        let Some(render_context) = self.render_context.as_ref() else { return Ok(()) };
        render_context.gpu_context.queue.submit([encoder.finish()]);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Canvas {
    id: u64,
    inner: Arc<RwLock<CanvasState>>,
}

impl AsRef<Canvas> for Canvas {
    fn as_ref(&self) -> &Canvas {
        self
    }
}

impl Canvas {
    pub(crate) fn new(width: u32, height: u32, init_black: bool) -> Self {
        let id = CANVAS_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            inner: Arc::new(RwLock::new(CanvasState::new(id, width, height, init_black))),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, CanvasState> {
        match self.inner.read() {
            Ok(guard) => guard,
            Err(err) => err.into_inner(),
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, CanvasState> {
        match self.inner.write() {
            Ok(guard) => guard,
            Err(err) => err.into_inner(),
        }
    }

    pub fn draw(&self, commands: impl FnOnce(&mut CanvasState)) {
        commands(&mut self.write())
    }
}
