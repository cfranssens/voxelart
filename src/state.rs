use bytemuck::cast_slice;
use cgmath::Vector3;
use image::{DynamicImage, GenericImageView, load_from_memory, RgbaImage};
use wgpu::{Adapter, AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Face, Features, FilterMode, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, IndexFormat, Instance, InstanceDescriptor, MultisampleState, Operations, Origin3d, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceError, SurfaceTexture, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexState};
use wgpu::LoadOp::Clear;
use wgpu::PowerPreference::HighPerformance;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;
use crate::{INDICES, Vertex, VERTICES, texture};
use crate::camera::{Camera, CameraController, CameraUniform};

pub struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    window: Window,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    diffuse_bind_group: BindGroup,
    diffuse_texture: texture::Texture,

    camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size: PhysicalSize<u32> = window.inner_size(); // retrieve size information from window object

        // Create a new instance and surface
        let instance: Instance = Instance::new(InstanceDescriptor {backends: Backends::all(), dx12_shader_compiler: Default::default() });
        let surface: Surface = unsafe { instance.create_surface(&window)}.unwrap();

        let adapter: Adapter = instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            }
        ).await.unwrap();

        let (device, queue): (Device, Queue) = adapter.request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: wgpu::Limits::default(),
                label: None
            },
            None
        ).await.unwrap();

        let caps: SurfaceCapabilities = surface.get_capabilities(&adapter);
        let format: TextureFormat = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
        let config: SurfaceConfiguration = SurfaceConfiguration {usage: TextureUsages::RENDER_ATTACHMENT, format, width: size.width, height: size.height, present_mode: caps.present_modes[0], alpha_mode: caps.alpha_modes[0], view_formats: vec![]};

        surface.configure(&device, &config);

        let diffuse_bytes: &[u8] = include_bytes!("../textures/img.png");
        let diffuse_texture: texture::Texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "rick").unwrap();

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
        let diffuse_bind_group: BindGroup = device.create_bind_group(
            &BindGroupDescriptor {
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

        let mut camera: Camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fov: 45.0, near: 0.1, far: 100.0,
            uniform: CameraUniform::new(),
            controller: CameraController::default()
        };

        let camera_buffer: Buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Camera buffer"),
                contents: cast_slice(&[camera.uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
            }
        );

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
        let render_pipeline: RenderPipeline = device.create_render_pipeline(&RenderPipelineDescriptor {label: Some("Render Pipeline"), layout: Some(&render_pipeline_layout), vertex: VertexState { module: &shader, entry_point: "vs_main", buffers: &[Vertex::desc()]}, fragment: Some(FragmentState {module: &shader, entry_point: "fs_main", targets: &[Some(ColorTargetState {format: config.format, blend: Some(BlendState::ALPHA_BLENDING), write_mask: ColorWrites::ALL})]}), primitive: PrimitiveState {topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: Some(Face::Back), polygon_mode: PolygonMode::Fill, unclipped_depth: false, conservative: false}, depth_stencil: None, multisample: MultisampleState {count: 1, mask: !0, alpha_to_coverage_enabled: false}, multiview: None});

        let vertex_buffer: Buffer = device.create_buffer_init(&BufferInitDescriptor {label: Some("Vertex Buffer"), contents: bytemuck::cast_slice(&VERTICES), usage: BufferUsages::VERTEX});
        let index_buffer: Buffer = device.create_buffer_init(&BufferInitDescriptor { label: Some("Index buffer"), contents: bytemuck::cast_slice(&INDICES), usage: BufferUsages::INDEX});

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
            num_indices: INDICES.len() as u32,
            diffuse_bind_group,
            diffuse_texture,

            camera,
            camera_buffer,
            camera_bind_group
        }
    }
    // handle to the 'window' member of struct
    pub fn window(&self) -> &Window {
        &self.window
    }
    // Called when winit window is resized
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    // handling input
    pub fn input(&mut self, control_flow: &mut ControlFlow, event: &Event<()>) {
        let mut window_event: &WindowEvent = &WindowEvent::CloseRequested;
        let mut device_event: &DeviceEvent = &DeviceEvent::Added;

        match event {
            Event::WindowEvent {ref event, window_id} if window_id == &self.window.id() => {
                window_event = event;
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => {
                                self.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &mut so w have to dereference it twice
                                self.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    },

                Event::RedrawRequested(window_id) if window_id == &self.window.id() => {
                    self.update();
                    match self.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            self.resize(self.size)
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // We're ignoring timeouts
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    self.window.request_redraw();
                }

                Event::DeviceEvent {event, ..} => device_event = event,
                _ => {}
        }
        self.camera.process_events(window_event, device_event);
    }

    // Update (called every frame)
    pub fn update(&mut self) {
        self.camera.update_view_proj();
        self.queue.write_buffer(&self.camera_buffer, 0, cast_slice(&[self.camera.uniform]));
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output: SurfaceTexture = self.surface.get_current_texture()?;
        let view: TextureView = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder: CommandEncoder = self.device.create_command_encoder(&CommandEncoderDescriptor {label: Some("Render Encoder")});

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
            depth_stencil_attachment: None,
        });


        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

