use bytemuck::{Pod, Zeroable};
use cgmath::{Deg, Matrix4, perspective, Point3, SquareMatrix, Vector3};
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent};
use crate::OPENGL_TO_WGPU_MATRIX;

#[derive(Default)]
pub struct CameraController {
    speed: f32,
    m_x: f32,
    m_y: f32,
    is_left_click: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4]
}
impl CameraUniform {
    pub fn new() -> CameraUniform {
        Self {view_proj: Matrix4::identity().into()}
    }
}
pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,

    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,

    pub uniform: CameraUniform,
    pub controller: CameraController,
}

impl Camera {
    pub fn process_events(&mut self, window_event: &WindowEvent, device_event: &DeviceEvent) {
        match window_event {
            WindowEvent::MouseInput {button: MouseButton::Left, state, ..} => {
                self.controller.is_left_click = *state == ElementState::Pressed;
            },
            _ => {},
        }

        match device_event {
            DeviceEvent::MouseMotion {delta} => {
                self.controller.m_x += (delta.0 as f32);
                self.controller.m_y += (delta.1 as f32);

                let pitch = self.controller.m_x.to_radians();
                let yaw = self.controller.m_y.to_radians();

                self.eye.x = (pitch.cos() * 3.0);
                self.eye.z = pitch.sin() * 3.0;
            }
            _ => {}
        }
    }
    pub fn update_view_proj(&mut self) {
        let view: Matrix4<f32> = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj: Matrix4<f32> = perspective(Deg(self.fov), self.aspect, self.near, self.far);
        self.uniform.view_proj = (OPENGL_TO_WGPU_MATRIX * (proj * view)).into();
    }
}