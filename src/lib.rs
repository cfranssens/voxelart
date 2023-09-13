use std::mem::size_of;
use bytemuck::{Pod, Zeroable};
use cgmath::Matrix4;
use wgpu::{BufferAddress, SurfaceError, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};
use winit::{event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::Window;
use crate::state::State;

mod state;
mod texture;
mod camera;
pub mod utils;
mod voxel;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 3],
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

impl Vertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3
                },
            ]
        }
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub async fn run() {
    env_logger::init();

    // Create new event loop, and link window events to it
    let event_loop: EventLoop<()> = EventLoop::new();
    let window: Window = WindowBuilder::new().with_inner_size(PhysicalSize::new(1920, 1080)).with_position(PhysicalPosition::new(1600, 0)).build(&event_loop).unwrap();

    let mut state: State = State::new(Some(window)).await;

    // Start main event loop
    event_loop.run(move |event, _, control_flow| {
        state.input(control_flow, &event);
        match &state.window {
            Some(window) => {
                match event {
                    Event::WindowEvent {ref event, window_id} if window_id == window.id() => match event {
                        WindowEvent::CloseRequested => {*control_flow = ControlFlow::ExitWithCode(0)}
                        WindowEvent::Resized(physical_size) => {state.resize(*physical_size)}
                        WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {state.resize(**new_inner_size)}
                        _ => {}
                    }

                    Event::RedrawRequested(window_id)
                    if window_id == window.id() => {
                        state.update();
                        match state.render() {Ok(_) => {}, Err(SurfaceError::Lost) => state.resize(state.size), Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::ExitWithCode(-1), Err(e) => eprintln!("{:?}", e) }
                    }

                    Event::MainEventsCleared => {
                        window.request_redraw();
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    });
}