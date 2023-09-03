use wgpu::SurfaceError;
use winit::{event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use winit::window::Window;
use crate::state::State;

mod state;

pub async fn run() {
    env_logger::init();

    // Create new event loop, and link window events to it
    let event_loop = EventLoop::new();
    let window: Window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state: State = State::new(window).await;

    // Start main event loop
    event_loop.run(move |event, _, control_flow|
        match event {
            Event::WindowEvent {
                ref event, window_id
            } if window_id == state.window().id() => match event {
                WindowEvent::Resized(size) => { state.resize(*size) }
                WindowEvent::Moved(_) => {}
                WindowEvent::CloseRequested => { *control_flow = ControlFlow::ExitWithCode(0) }
                WindowEvent::Destroyed => {}
                WindowEvent::DroppedFile(_) => {}
                WindowEvent::HoveredFile(_) => {}
                WindowEvent::HoveredFileCancelled => {}
                WindowEvent::ReceivedCharacter(_) => {}
                WindowEvent::Focused(_) => {}
                WindowEvent::KeyboardInput { .. } => {}
                WindowEvent::ModifiersChanged(_) => {}
                WindowEvent::Ime(_) => {}
                WindowEvent::CursorMoved { .. } => {}
                WindowEvent::CursorEntered { .. } => {}
                WindowEvent::CursorLeft { .. } => {}
                WindowEvent::MouseWheel { .. } => {}
                WindowEvent::MouseInput { .. } => {}
                WindowEvent::TouchpadMagnify { .. } => {}
                WindowEvent::SmartMagnify { .. } => {}
                WindowEvent::TouchpadRotate { .. } => {}
                WindowEvent::TouchpadPressure { .. } => {}
                WindowEvent::AxisMotion { .. } => {}
                WindowEvent::Touch(_) => {}
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => { state.resize(**new_inner_size) }
                WindowEvent::ThemeChanged(_) => {}
                WindowEvent::Occluded(_) => {}
            }

            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {},
                    Err(SurfaceError::Lost) => state.resize(state.size),
                    Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::ExitWithCode(-1),
                    Err(e) => eprintln!("{:?}", e)
                }
            }

            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        });
}