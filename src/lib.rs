// TODO: currently, verdant doesn't allow for the full amount of low-level control that it should
//       it makes a lot of assumptions and choices for the user
//       i need to expose various functions that allow the user to choose things
//       (do you want transparency? do you want window decorations? do you want linear filtering on images?)

// TODO: the renderer expects a certain setup that might not be the real setup
//       (premultiplied, non-srgb)
//       and i don't think it'll look right if you're not on that setup

// TODO: while the library aims to be performant, i'm yet to actually do any optimization passes or
//       profiling, so i know there's a long way to go
//       though i am currently happy with its performance as it *is* in a pre-1.0 state
//       honestly, it does pretty well for what it is

// TODO: another documentation pass...

#![deny(clippy::unwrap_used)]

pub use wgpu::TextureFormat;
pub use winit::{event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent}, keyboard::{Key, KeyCode, NamedKey, PhysicalKey}};

use bytemuck::{Pod, Zeroable};

use pollster::block_on;
use wgpu::{Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState, BufferBindingType, ColorTargetState, ColorWrites, Device, DeviceDescriptor, Extent3d, FilterMode, FragmentState, FrontFace, Instance, MultisampleState, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerDescriptor, ShaderStages, Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexBufferLayout, VertexState, VertexStepMode, include_wgsl, util::{DeviceExt}, vertex_attr_array, wgt::TextureDataOrder};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event_loop::{ActiveEventLoop, EventLoop}, window::WindowAttributes};

use std::{collections::{HashMap, VecDeque}, sync::Arc};

use crate::{canvas::{Canvas, RenderSurface}, errors::Error, transform::Transform2d, types::{Color, WindowId, WindowProperties}, vec::Vec2, window::Window};

pub mod canvas;
pub mod errors;
pub mod image;
pub mod shapes;
pub mod transform;
pub mod types;
pub mod window;
pub mod view;
pub mod vec;

mod shape_vertices;

#[cfg(feature = "text")]
pub mod text;

pub type RendererResult<T> = Result<T, Error>;

const KIND_RECT:     u32 = 0;
const KIND_ELLIPSE:  u32 = 1;
const KIND_LINE:     u32 = 2;
const KIND_TEXTURED: u32 = 3;
#[allow(unused)]
const KIND_SDF_TEXT: u32 = 4;
const KIND_CANVAS:   u32 = 5;

/// Constructs a `Color` from RGB components in the range `0.0..=1.0`, with full opacity.
#[inline(always)]
pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}

/// Constructs a `Color` from RGBA components in the range `0.0..=1.0`.
#[inline(always)]
pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color { r, g, b, a }
}

/// Constructs a `Color` from RGB components in the range `0.0..=255.0`, with full opacity.
#[inline(always)]
pub const fn rgb255(r: f32, g: f32, b: f32) -> Color {
    Color { r: r / 255., g: g / 255., b: b / 255., a: 1.0 }
}

/// Constructs a `Color` from RGBA components in the range `0.0..=255.0`.
#[inline(always)]
pub const fn rgba255(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color { r: r / 255., g: g / 255., b: b / 255., a: a / 255. }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position:      Vec2,
    uv:            Vec2,
    radii:         Vec2,
    fill_color:    Color,
    outline_color: Color,
    outline_width: f32,
    corner_radius: f32,
    kind:          u32,
}

#[derive(Debug)]
struct GpuContext {
    adapter: Adapter,
    device: Device,
    queue: Queue,

    projection_group_layout: BindGroupLayout,
    texture_group_layout: BindGroupLayout,

    sampler: Sampler,
    dummy_bind_group: BindGroup,

    pipeline: RenderPipeline,
}

impl GpuContext {
    fn create_dummy_texture(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        sampler: &Sampler,
    ) -> BindGroup {
        let dummy_texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("dummy 1x1 texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            TextureDataOrder::MipMajor,
            &[255, 255, 255, 255],
        );

        let view = dummy_texture.create_view(&TextureViewDescriptor::default());

        device.create_bind_group(&BindGroupDescriptor {
            label: Some("dummy texture bind group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}

struct RendererContext {
    instance: Arc<Instance>,
    context: Option<Arc<GpuContext>>,

    windows: HashMap<WindowId, Window>,

    virtual_to_real: HashMap<WindowId, winit::window::WindowId>,
    real_to_virtual: HashMap<winit::window::WindowId, WindowId>,

    window_queue: VecDeque<(WindowId, WindowProperties)>,

    events: Vec<(WindowId, WindowEvent)>,

    is_wayland: bool,
}

impl RendererContext {
    fn new(is_wayland: bool) -> Self {
        Self {
            instance: Arc::new(Instance::default()),
            context: None,

            windows: HashMap::new(),

            virtual_to_real: HashMap::new(),
            real_to_virtual: HashMap::new(),

            window_queue: VecDeque::new(),

            events: Vec::new(),

            is_wayland,
        }
    }

    async fn get_or_init_context(&mut self, surface: &Surface<'_>) -> RendererResult<Arc<GpuContext>> {
        if let Some(context) = self.context.clone() {
            Ok(context)
        } else {
            let adapter = self.instance.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            }).await?;

            let surface_capabilities = surface.get_capabilities(&adapter);
            let format = surface_capabilities.formats.iter().copied()
                .find(|f| !f.is_srgb())
                .unwrap_or(surface_capabilities.formats[0]);

            let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;

            let projection_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("projection layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

            let texture_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("texture layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

            let sampler = device.create_sampler(&SamplerDescriptor {
                label: Some("texture sampler"),
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Nearest,
                ..Default::default()
            });

            let shader = device.create_shader_module(include_wgsl!("shaders/2d.wgsl"));

            let dummy_bind_group = GpuContext::create_dummy_texture(
                &device,
                &queue,
                &texture_group_layout,
                &sampler
            );

            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    Some(&projection_group_layout),
                    Some(&texture_group_layout)
                ],
                immediate_size: 0,
            });

            let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),

                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        // TODO: i think it would be really cool if there was some way to build
                        //       this from the shader
                        //       i know it's possible
                        attributes: &vertex_attr_array![
                            0 => Float32x2, // position
                            1 => Float32x2, // uv
                            2 => Float32x2, // radii
                            3 => Float32x4, // fill_color
                            4 => Float32x4, // outline_color
                            5 => Float32,   // outline_width
                            6 => Float32,   // corner_radius
                            7 => Uint32,    // kind
                        ]
                    }],
                },

                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState {
                        format,
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::One,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::One,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::ALL,
                    })],
                }),

                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },

                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

            let context = Arc::new(GpuContext {
                adapter,
                device,
                queue,

                projection_group_layout,
                texture_group_layout,

                sampler,
                dummy_bind_group,

                pipeline,
            });
            self.context = Some(context.clone());

            Ok(context)
        }
    }

    fn process_queued_windows(&mut self, event_loop: &dyn ActiveEventLoop) -> RendererResult<()> {
        while let Some((id, props)) = self.window_queue.pop_front() {
            let mut attributes = WindowAttributes::default()
                .with_title(props.title)
                .with_resizable(props.resizable)
                .with_surface_size(PhysicalSize::new(props.width, props.height))
                .with_transparent(props.transparent);

            #[cfg(linux_platform)]
            {
                use winit::platform::wayland::WindowAttributesWayland;
                use winit::platform::x11::WindowAttributesX11;
                if self.is_wayland {
                    let platform_attributes = WindowAttributesWayland::default()
                        .with_name("verdant", "verdant");
                    attributes = attributes.with_platform_attributes(Box::new(platform_attributes));
                } else {
                    let platform_attributes = WindowAttributesX11::default()
                        .with_name("verdant", "verdant");
                    attributes = attributes.with_platform_attributes(Box::new(platform_attributes));
                }
            }

            #[cfg(windows_platform)]
            {
                use winit::platform::windows::WindowAttributesWindows;
                let platform_attributes = WindowAttributesWindows::default()
                    .with_class_name("Verdant");
                attributes = attributes.with_platform_attributes(Box::new(platform_attributes));
            }

            let inner_window = Arc::new(event_loop.create_window(attributes)?);
            let surface = self.instance.create_surface(inner_window.clone())?;

            let context = block_on(self.get_or_init_context(&surface))?;
            let window = Self::configure_window(inner_window, surface, context, props.width, props.height)?;

            let real_id = window.inner_window.id();
            self.virtual_to_real.insert(id, real_id);
            self.real_to_virtual.insert(real_id, id);
            self.windows.insert(id, window);
        }

        Ok(())
    }

    pub(crate) fn configure_window(
        inner_window: Arc<Box<dyn winit::window::Window>>,
        surface: Surface<'static>,
        context: Arc<GpuContext>,
        width: u32,
        height: u32,
    ) -> RendererResult<Window> {
        let surface_capabilities = surface.get_capabilities(&context.adapter);
        let format = surface_capabilities.formats.iter().copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);
        let alpha_mode = [
            wgpu::CompositeAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::Inherit,
        ]
            .iter().copied()
            .find(|m| surface_capabilities.alpha_modes.contains(m))
            .unwrap_or(wgpu::CompositeAlphaMode::Auto);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
            format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&context.device, &config);

        inner_window.request_redraw();

        let window = Window::new(
            inner_window,
            surface,
            config,
            context.clone()
        );

        window.present_blank_frame()?;

        Ok(window)
    }
}

pub struct Renderer {
    event_loop: EventLoop,
    context: RendererContext,
}

fn ortho(width: f32, height: f32) -> Transform2d {
    *Transform2d::scaling(2. / width, -2. / height).translate(-1., 1.)
}

impl Renderer {
    /// Creates a new `Renderer` instance with no windows.
    pub fn new() -> RendererResult<Self> {
        let event_loop = EventLoop::new()?;
        let is_wayland = {
            #[cfg(linux_platform)]
            {
                use winit::platform::wayland::EventLoopExtWayland;

                event_loop.is_wayland()
            }
            #[cfg(not(linux_platform))]
            false
        };
        Ok(Self {
            event_loop,
            context: RendererContext::new(is_wayland),
        })
    }

    /// Creates a new window with the given title, dimensions, and resizability.
    /// Returns a [`WindowId`] that can be used to interact with the window.
    /// Initializes the GPU context if this is the first window created.
    pub fn create_window(
        &mut self,
        title: impl ToString,
        width: u32,
        height: u32,
    ) -> WindowId {
        self.create_window_ext(WindowProperties {
            width,
            height,
            title: title.to_string(),
            ..Default::default()
        })
    }

    /// Creates a new window with the given properties.
    /// Returns a [`WindowId`] that can be used to interact with the window.
    /// Initializes the GPU context if this is the first window created.
    pub fn create_window_ext(&mut self, props: WindowProperties) -> WindowId {
        let id = WindowId::new();

        self.context.window_queue.push_back((id, props));

        id
    }

    /// Create a new canvas with the given width and height.
    pub fn create_canvas(&mut self, width: u32, height: u32) -> RendererResult<Canvas> {
        Ok(Canvas::new(width, height))
    }

    /// Pumps the event loop and returns all window events since the last call.
    /// Resize, cursor movement, and focus events are also forwarded to their
    /// respective windows internally.
    ///
    /// Only available on Windows, macOS, Linux, and Android.
    #[cfg(any(windows_platform, macos_platform, linux_platform, android_platform))]
    pub fn poll(&mut self) -> Vec<(WindowId, WindowEvent)> {
        use std::{mem::take, time::Duration};
        use winit::event_loop::pump_events::EventLoopExtPumpEvents;

        self.event_loop.pump_app_events(Some(Duration::ZERO), &mut self.context);

        take(&mut self.context.events)
    }

    /// Closes the window with the given [`WindowId`], removing it from the renderer.
    /// Returns `true` if a window with that ID existed.
    pub fn close_window(&mut self, id: WindowId) -> bool {
        self.context.windows.remove(&id).is_some()
    }

    /// Returns a mutable reference to the window with the given [`WindowId`], if it exists.
    pub fn get_window(&mut self, id: WindowId) -> Option<&mut Window> {
        self.context.windows.get_mut(&id)
    }

    /// Submits all queued draw calls for each open window to the GPU and presents their frames.
    pub fn flush(&mut self) -> RendererResult<()> {
        for window in self.context.windows.values_mut() {
            window.flush()?;
        }

        Ok(())
    }

    /// Returns `true` if any windows are still open.
    pub fn is_running(&self) -> bool {
        !self.context.windows.is_empty() || !self.context.window_queue.is_empty()
    }

    // TODO: this needs to allow the user to pass in a custom trait
    //       and if i'm being honest i just want to get this working
    // pub fn run(self) -> RendererResult<()> {
    //     Ok(self.event_loop.run_app(self.context)?)
    // }
}

// TODO: error handling/forwarding
impl ApplicationHandler for RendererContext {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = self.process_queued_windows(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let _ = self.process_queued_windows(event_loop);

        if let Some(id) = self.real_to_virtual.get(&window_id) {
            if let Some(window) = self.windows.get_mut(id) {
                if let WindowEvent::SurfaceResized(size) = event {
                    window.on_resize(size);
                }
                if let WindowEvent::PointerMoved { position, .. } = event {
                    window.on_mouse_move(position);
                }
                if let WindowEvent::Focused(focus) = event {
                    window.on_focus_update(focus);
                }
            }

            self.events.push((*id, event));
        }
    }
}
