use bytemuck::{Pod, Zeroable};
use cgmath::Zero;
use wgpu::{Adapter, Backends, Buffer, BufferUsages, Device, DeviceDescriptor, Features, InstanceDescriptor, Queue, RequestAdapterOptions, Surface};
use wgpu::PowerPreference::HighPerformance;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub(crate) fn create_wgpu_buffer(device: &Device, label: Option<&str>, contents: &[u8], usage: BufferUsages) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {label, contents, usage})
}


// run tests for utility functions
#[cfg(test)]
mod tests {
    use pollster::FutureExt;
    use wgpu::BufferUsages;
    use crate::utils::create_wgpu_buffer;
    use wgpu::{Adapter, Backends, Device, DeviceDescriptor, Features, InstanceDescriptor, Queue, RequestAdapterOptions, Surface};
    use crate::state::State;

    #[test]
    fn test_create_wgpu_buffer() {
        let state: State = State::new(None).block_on();

        create_wgpu_buffer(&state.device, Some("Buffer 1"), bytemuck::cast_slice(&[0; 1024]), BufferUsages::VERTEX);
        create_wgpu_buffer(&state.device, Some("Buffer 2"), bytemuck::cast_slice(&[0; 1024 * 1024]), BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        create_wgpu_buffer(&state.device, Some("Buffer 3"), bytemuck::cast_slice(&[0; 1024 * 4096]), BufferUsages::INDEX | wgpu::BufferUsages::COPY_SRC);
    }
}