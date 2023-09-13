use bytemuck::{Pod, Zeroable};
use cgmath::{Deg, Matrix4, perspective, Point3, SquareMatrix, Vector3};
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent};
use crate::OPENGL_TO_WGPU_MATRIX;

#[derive(Default)]
pub struct CameraController {
    sensitivity: f32, // configure camera sensitivity

    m_x: f32, // mouse x
    m_y: f32, // mouse y

    is_left_click: bool, // if left mouse button is clicked
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4]
} // wrapper for camera matrices
impl CameraUniform {
    pub fn new() -> CameraUniform {
        Self {view_proj: Matrix4::identity().into()}
    }
}


// Camera object
pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,

    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,

    pub uniform: CameraUniform,
    pub controller: Option<CameraController>,
}

impl Camera {
    // Temporary code to test mouse compatibility
    pub fn process_events(&mut self, window_event: &WindowEvent, device_event: &DeviceEvent) {
        match &mut self.controller {
            Some(controller) => {
                match window_event {
                    WindowEvent::MouseInput {
                        button: MouseButton::Left, state, ..
                    } => {
                        controller.is_left_click = *state == ElementState::Pressed;
                    },
                    _ => {},
                }

                match device_event {
                    DeviceEvent::MouseMotion { delta } => {
                        if controller.is_left_click {
                            controller.m_x += delta.0 as f32;
                            controller.m_y -= delta.1 as f32;

                            let pitch = controller.m_y.to_radians();
                            let yaw = controller.m_x.to_radians();

                            self.eye.x = (pitch.sin() * 15.0) * yaw.cos();
                            self.eye.y = pitch.cos() * 15.0;
                            self.eye.z = (pitch.sin() * 15.0) * yaw.sin();
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    pub fn update_view_proj(&mut self) {
        let view: Matrix4<f32> = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj: Matrix4<f32> = perspective(Deg(self.fov), self.aspect, self.near, self.far);
        self.uniform.view_proj = (OPENGL_TO_WGPU_MATRIX * (proj * view)).into();

        match &mut self.controller {
            Some(controller) => {
                controller.m_x += 0.0 as f32;
                controller.m_y -= 0.005 as f32;

                let pitch = controller.m_y;
                let yaw = controller.m_x;

                //self.eye.x = (pitch.sin() * 3.0) * yaw.cos();
                self.eye.y = pitch.cos() * 150.0;
                //self.eye.z = 300.0; //(pitch.sin() * 3.0) * yaw.sin();
            }

            _ => {}
        }
    } // update view matrix inside camera
}