use bytemuck::{Pod, Zeroable};
use cgmath::{Vector3, Vector4};
use crate::Vertex;

// THIS FILE CONTAINS THE BASE POINTS FOR EVERY VOXEL
pub(crate) const VERTEX_INDICES: &[u16] =
    &[ // front
        0, 1, 2,
        2, 3, 0,

        // back
        4, 5, 6,
        6, 7, 4,

        // bottom
        5, 2, 1,
        6, 5, 1,

        // top
        0, 3, 4,
        0, 4, 7,
    ];

pub(crate) const VV : &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.5]}, // 0
    Vertex { position: [-0.5, -0.5, 0.5]},
    Vertex { position: [0.5, -0.5, 0.5]},
    Vertex { position: [0.5, 0.5, 0.5]}, // 3
    Vertex { position: [0.5, 0.5, -0.5]}, // 4
    Vertex { position: [0.5, -0.5, -0.5]},
    Vertex { position: [-0.5, -0.5, -0.5]},
    Vertex { position: [-0.5, 0.5, -0.5]} // 7
];

pub struct Instance {
    position: Vector3<f32>,
    color: Vector4<f32>,

    pub raw: InstanceRaw
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    pub(crate) color: [f32; 4],
    pub(crate) position: [f32; 3],
}

impl Instance {
    pub fn new(position: Vector3<f32>, color: Vector4<f32>) -> Self {
        Self {
            position,
            color,
            raw: InstanceRaw {
                color: color.into(),
                position: position.into()
            }
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1, // color
                    format: wgpu::VertexFormat::Float32x4,
                },

                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ],
        }
    }
}