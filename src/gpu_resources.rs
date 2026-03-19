use wgpu::{Texture, util::DeviceExt};
use image::RgbaImage;

use crate::model::Mesh;

pub struct TextureGPU {
   image: wgpu::Texture,
   view: wgpu::TextureView,
   sampler: wgpu::Sampler,  
}

impl TextureGPU {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, image_data: &RgbaImage) -> Self {
        let dimensions = image_data.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let image = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("Diffuse Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &image,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = image.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            image,
            view,
            sampler
        }
    }
}

pub struct MaterialGPU {
    albedo: TextureGPU,
    pub texture_bind_group: wgpu::BindGroup,
}

impl MaterialGPU {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, albedo: TextureGPU) -> Self {
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&albedo.sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        Self {
            albedo,
            texture_bind_group
        }
    }
}

pub struct MeshGPU {
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer:  Option<wgpu::Buffer>,
    pub vertex_count: u32,
    pub index_count:  u32
}

impl MeshGPU {
    pub fn new(device: &wgpu::Device, mesh: &Mesh) -> Self {
        let vertex_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        ));

        let index_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        ));

        Self {
            vertex_buffer,
            index_buffer,
            vertex_count: mesh.vertices.len() as u32,
            index_count:  mesh.indices.len() as u32
        }
    }
}
pub struct ResourceController {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl ResourceController {
    pub fn create_mesh_gpu(&self, mesh: &Mesh) -> MeshGPU {
        MeshGPU::new(&self.device, &mesh)
    }

    pub fn create_texture_gpu(&self, image: &RgbaImage) -> TextureGPU {
        TextureGPU::new(&self.device, &self.queue, &image)
    }

    pub fn create_material_gpu(&self, layout: &wgpu::BindGroupLayout, texture: TextureGPU) -> MaterialGPU {
        MaterialGPU::new(&self.device, &layout, texture)
    }
}