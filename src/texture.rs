use std::default::Default;
use image::{DynamicImage, GenericImageView, RgbaImage};
use anyhow::*;
use wgpu::{Sampler, TextureView, Device, Queue, Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, ImageCopyTexture, Origin3d, TextureAspect, ImageDataLayout, TextureViewDescriptor, SamplerDescriptor, AddressMode, FilterMode, SurfaceConfiguration, CompareFunction};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler
}

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
    pub fn create_depth_texture(device: &Device, config: &SurfaceConfiguration, label: &str) -> Self {
        let size: Extent3d = Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1
        };

        let desc: TextureDescriptor = TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        };

        let texture: wgpu::Texture = device.create_texture(&desc);
        let view: TextureView = texture.create_view(&TextureViewDescriptor::default());
        let sampler: Sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {texture, view, sampler}
    }

    pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8], label: &str) -> Result<Self> {
        let img: DynamicImage = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_image(device: &Device, queue: &Queue, img: &DynamicImage, label: Option<&str>) -> Result<Self> {
        let img_rgba: RgbaImage = img.to_rgba8();

        let dimensions: (u32, u32) = img_rgba.dimensions();
        let texture_size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
        };

        // explicitly state this as wgpu::Texture
        let texture: wgpu::Texture = device.create_texture(&TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("texture"),
            view_formats: &[]
        });

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All
            },
            &img_rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1)
            },
            texture_size
        );

        let view: TextureView = texture.create_view(&TextureViewDescriptor::default());
        let sampler: Sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {texture, view, sampler})
    }
}