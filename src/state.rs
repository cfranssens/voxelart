use bytemuck::cast_slice;
use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Vector3, Zero};
use image::{DynamicImage, GenericImageView, load_from_memory, RgbaImage};
use wgpu::{Adapter, AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder, CommandEncoderDescriptor, CompareFunction, CompositeAlphaMode, DepthBiasState, DepthStencilState, Device, DeviceDescriptor, Extent3d, Face, Features, FilterMode, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, IndexFormat, InstanceDescriptor, LoadOp, MultisampleState, Operations, Origin3d, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState, PrimitiveTopology, QuerySet, QuerySetDescriptor, QueryType, Queue, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, StencilState, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceError, SurfaceTexture, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexState};
use wgpu::LoadOp::Clear;
use wgpu::PowerPreference::HighPerformance;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;
use crate::{Vertex, texture};

use crate::camera::{Camera, CameraController, CameraUniform};
use crate::utils::create_wgpu_buffer;
use crate::voxel::{VERTEX_INDICES, VV, Instance};

pub struct State {
    surface: Option<Surface>,
    pub(crate) device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub(crate) window: Option<Window>,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    diffuse_bind_group: BindGroup,
    diffuse_texture: texture::Texture,
    depth_texture: texture::Texture,

    camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    instances: Vec<Instance>,
    instance_buffer: Buffer
}

impl State {
    pub async fn new(window: Option<Window>) -> Self {
        let size: PhysicalSize<u32> = match &window {Some(w) => w.inner_size(), _ => (0, 0).into()}; // retrieve size information from window object

        // Create a new instance and surface (if window is present)
        let instance: wgpu::Instance = wgpu::Instance::new(InstanceDescriptor {backends: Backends::all(), dx12_shader_compiler: Default::default() });
        let surface: Option<Surface> = match &window {Some(w) => Some(unsafe {instance.create_surface(&w)}.unwrap()), _ => None};

        // Connect to the almighty gpu
        let adapter: Adapter = instance.request_adapter(&RequestAdapterOptions {
                power_preference: HighPerformance,
                compatible_surface: Option::from(&surface),
                force_fallback_adapter: false
            }).await.unwrap();
        let (device, queue): (Device, Queue) = adapter.request_device(&DeviceDescriptor { features: Features::POLYGON_MODE_LINE, limits: wgpu::Limits::default(), label: None }, None).await.unwrap();

        // configure surface if there is a window
        let caps: SurfaceCapabilities = match &surface {Some(s) => s.get_capabilities(&adapter), _ => SurfaceCapabilities::default()};
        let format: TextureFormat = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(TextureFormat::Rgba8UnormSrgb);
        let config: SurfaceConfiguration = SurfaceConfiguration {usage: TextureUsages::RENDER_ATTACHMENT, format, width: size.width, height: size.height, present_mode: PresentMode::Fifo, alpha_mode: CompositeAlphaMode::Auto, view_formats: vec![]};
        match &surface {Some(s) => s.configure(&device, &config), _ => {}};

        // load texture
        let diffuse_bytes: &[u8] = include_bytes!("../textures/img.png");
        let diffuse_texture: texture::Texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "rick").unwrap();

        let depth_texture: texture::Texture = texture::Texture::create_depth_texture(&device, &config, "depth texture");

        let texture_bind_group_layout: BindGroupLayout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None
               }
            ],
            label: Some("Texture Bind Group Layout")
        });
        let diffuse_bind_group: BindGroup = device.create_bind_group(&BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&diffuse_texture.view)
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&diffuse_texture.sampler)
                    }
                ],
                label: Some("Diffuse Bind Group")
        });

        // camera presets
        let mut camera: Camera = Camera {
            eye: (50.0, 10.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fov: 110.0, near: 0.1, far: 10000.0,
            uniform: CameraUniform::new(),
            controller: Some(CameraController::default())
        };
        let camera_buffer: Buffer = create_wgpu_buffer(&device, Some("Camera Buffer"), cast_slice(&[camera.uniform]), BufferUsages::UNIFORM | BufferUsages::COPY_DST);

        let camera_bind_group_layout: BindGroupLayout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                }
            ],
            label: Some("Camera Bind Group Layout Descriptor")
        });
        let camera_bind_group: BindGroup = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                }
            ],
            label: Some("Camera Bind Group")
        });

        // define shader module
        let shader: ShaderModule = device.create_shader_module(ShaderModuleDescriptor {label: Some("Shader Module"), source: ShaderSource::Wgsl(include_str!("shader.wgsl").into())});
        let render_pipeline_layout: PipelineLayout = device.create_pipeline_layout(&PipelineLayoutDescriptor {label: Some("Render Pipeline Layout"), bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout], push_constant_ranges: &[]});
        // Grab a plate of spaghetti

        let render_pipeline: RenderPipeline = device.create_render_pipeline(&RenderPipelineDescriptor {label: Some("Render Pipeline"), layout: Some(&render_pipeline_layout), vertex: VertexState { module: &shader, entry_point: "vs_main", buffers: &[Vertex::desc(), Instance::desc()]}, fragment: Some(FragmentState {module: &shader, entry_point: "fs_main", targets: &[Some(ColorTargetState {format: config.format, blend: Some(BlendState::ALPHA_BLENDING), write_mask: ColorWrites::ALL})]}), primitive: PrimitiveState {topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: Some(Face::Back), polygon_mode: PolygonMode::Line, unclipped_depth: false, conservative: false}, multisample: MultisampleState {count: 1, mask: !0, alpha_to_coverage_enabled: false}, multiview: None, depth_stencil: Some(DepthStencilState {format: texture::Texture::DEPTH_FORMAT, depth_write_enabled: true, depth_compare: CompareFunction::Less, stencil: StencilState::default(), bias: DepthBiasState::default()})});
        let vertex_buffer: Buffer = create_wgpu_buffer(&device, Some("Vertex Buffer"), cast_slice(&VV), BufferUsages::VERTEX);
        let index_buffer: Buffer = create_wgpu_buffer(&device, Some("Index buffer"), cast_slice(&VERTEX_INDICES), BufferUsages::INDEX);

        // crappy test code
        let instances = (-100..0).flat_map(|z| {
            (0..100).flat_map(|y| {
                (0..100).map(move |x| {
                    let position: Vector3<f32> = Vector3::new(x as f32, y as f32, 100.0 + z as f32);
                    Instance::new(position, (x as f32 * 0.005, y as f32 * 0.005, (z + 200) as f32 * 0.005, 1.0).into())
                })
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(|d| {d.raw}).collect::<Vec<_>>();
        let instance_buffer: Buffer = create_wgpu_buffer(&device, Some("Instance buffer"), cast_slice(&instance_data), BufferUsages::VERTEX);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: VERTEX_INDICES.len() as u32,
            diffuse_bind_group,
            diffuse_texture,
            depth_texture,

            camera,
            camera_buffer,
            camera_bind_group,

            instances,
            instance_buffer
        }
    }

    // Called when winit window is resized
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            match &self.surface {Some(surface) => surface.configure(&self.device, &self.config), _ => {}};
        }
    }

    // handling input
    pub fn input(&mut self, control_flow: &mut ControlFlow, event: &Event<()>) {
    }

    // Update (called every frame)
    pub fn update(&mut self) {
        self.camera.update_view_proj();
        self.queue.write_buffer(&self.camera_buffer, 0, cast_slice(&[self.camera.uniform]));
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        match &self.surface {
            Some(surface) => {
                let output: SurfaceTexture = surface.get_current_texture()?;
                let view: TextureView = output.texture.create_view(&TextureViewDescriptor::default());
                let mut encoder: CommandEncoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Render Encoder") });

                let mut render_pass: RenderPass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: Clear(Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0
                            }),
                            store: true,
                        }
                    })],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: true
                        }),
                        stencil_ops: None
                    }),
                });


                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

                render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
                drop(render_pass);

                self.queue.submit(std::iter::once(encoder.finish()));
                output.present();

                Ok(())
            }

            _ => {Err(SurfaceError::Lost)}
        }
    }
}

