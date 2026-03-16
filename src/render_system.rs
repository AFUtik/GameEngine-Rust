use image::RgbaImage;

use crate::model;
use std:: {collections::HashMap, marker::PhantomData};
use wgpu::{Texture, util::DeviceExt};

struct TextureGPU {
   image: wgpu::Texture,
   view: wgpu::TextureView,
   sampler: wgpu::Sampler,  
}

impl TextureGPU {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, image_data: &RgbaImage) -> Self {
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

struct MaterialGPU {
    albedo: TextureGPU,
    texture_bind_group: wgpu::BindGroup,
}

impl MaterialGPU {
    fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, albedo: TextureGPU) -> Self {
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
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer:  Option<wgpu::Buffer>,
}

impl MeshGPU {
    fn new(device: &wgpu::Device, mesh: &model::Mesh) -> Self {
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
            index_buffer
        }
    }
}

struct Handle<T> {
    handle: u32,
    _phantom: std::marker::PhantomData<T>,
}
struct RenderObject {
    mesh: Handle<MeshGPU>,
    material: Handle<MaterialGPU>,
}

pub struct RenderScene<'a> {
    device: &'a wgpu::Device,
    queue:  &'a wgpu::Queue,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    
    meshes:    Vec<MeshGPU>,
    materials: Vec<MaterialGPU>,

    renderables: Vec<Handle::<RenderObject>>

    //meshConvert: HashMap<*const model::Mesh, MeshGPU>
}

impl<'a> RenderScene<'a> {
    fn new(device: &'a wgpu::Device, queue:  &'a wgpu::Queue, texture_bind_group_layout: wgpu::BindGroupLayout) -> Self {
        Self {
            device,
            queue,
            texture_bind_group_layout,
            meshes:      Vec::new(),
            materials:   Vec::new(),
            renderables: Vec::new()
        }
    }

    fn create_render_object(&mut self, mesh: &model::Mesh, image: &RgbaImage) -> RenderObject {
        let mesh_gpu = MeshGPU::new(&self.device, &mesh);

        let texture_gpu  = TextureGPU::new(&self.device, &self.queue, &image);
        let material_gpu = MaterialGPU::new(&self.device, &self.texture_bind_group_layout, texture_gpu);

        let obj_handle      = Handle::<RenderObject> {handle: self.renderables.len() as u32, _phantom: PhantomData};
        let mesh_handle     = Handle::<MeshGPU> {handle: self.meshes.len() as u32, _phantom: PhantomData};
        let material_handle = Handle::<MaterialGPU> {handle: self.materials.len() as u32, _phantom: PhantomData};

        self.renderables.push(obj_handle);
        self.meshes.push(mesh_gpu);
        self.materials.push(material_gpu);

        RenderObject { mesh: mesh_handle, material: material_handle}
    }
}

pub struct RenderSystem<'a> {
    device: &'a wgpu::Device,
    queue:  &'a wgpu::Queue,

    pipeline: wgpu::RenderPipeline,
    scene: RenderScene<'a>
}

impl<'a> RenderSystem<'a> {
    pub fn new(device: &'a wgpu::Device, queue:  &'a wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../resources/shaders/shader.wgsl").into()
                ),
            }
        );

        let color_target = Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                immediate_size: 0,
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

                label: Some("pipeline"),

                layout: Some(&pipeline_layout),

                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[model::Vertex::desc()],
                    compilation_options: Default::default(),
                },

                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[color_target],
                    compilation_options: Default::default(),
                }),

                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        Self {
            device,
            queue,
            pipeline,
            scene: RenderScene::new(device, queue, texture_bind_group_layout)
        }
    }

    pub fn pass(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
    }
}