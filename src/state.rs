use wgpu::{Adapter, Backends, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Face, Features, FragmentState, FrontFace, Instance, InstanceDescriptor, MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule, ShaderModuleDescriptor, ShaderSource, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceError, SurfaceTexture, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, VertexState};
use wgpu::LoadOp::Clear;
use wgpu::PowerPreference::HighPerformance;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    window: Window,
    render_pipeline: RenderPipeline
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

        // define shader module
        let shader: ShaderModule = device.create_shader_module(ShaderModuleDescriptor {label: Some("Shader Module"), source: ShaderSource::Wgsl(include_str!("shader.wgsl").into())});
        let render_pipeline_layout: PipelineLayout = device.create_pipeline_layout(&PipelineLayoutDescriptor {label: Some("Render Pipeline Layout"), bind_group_layouts: &[], push_constant_ranges: &[]});
        // Grab a plate of spaghetti
        let render_pipeline: RenderPipeline = device.create_render_pipeline(&RenderPipelineDescriptor {label: Some("Render Pipeline"), layout: Some(&render_pipeline_layout), vertex: VertexState { module: &shader, entry_point: "vs_main", buffers: &[]}, fragment: Some(FragmentState {module: &shader, entry_point: "fs_main", targets: &[Some(ColorTargetState {format: config.format, blend: Some(BlendState::ALPHA_BLENDING), write_mask: ColorWrites::ALL})]}), primitive: PrimitiveState {topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: Some(Face::Back), polygon_mode: PolygonMode::Fill, unclipped_depth: false, conservative: false}, depth_stencil: None, multisample: MultisampleState {count: 1, mask: !0, alpha_to_coverage_enabled: false}, multiview: None});

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline
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
    pub fn input(&mut self, event: &WindowEvent) {
        todo!()
    }

    // Update (called every frame)
    pub fn update(&mut self) {

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
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

