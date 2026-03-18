use glam::{Mat4, Vec3, Vec2};
use image::RgbaImage;
use std::rc::Rc;

use crate::model::{self, Mesh, Vertex};
use std:: {collections::HashMap, marker::PhantomData};
use wgpu::{Texture, util::DeviceExt};

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
    texture_bind_group: wgpu::BindGroup,
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
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer:  Option<wgpu::Buffer>,
    vertex_count: u32,
    index_count:  u32
}

impl MeshGPU {
    pub fn new(device: &wgpu::Device, mesh: &model::Mesh) -> Self {
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

/* 
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
*/
pub struct ResourceController {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl ResourceController {
    fn create_mesh_gpu(&self, mesh: &Mesh) -> MeshGPU {
        MeshGPU::new(&self.device, &mesh)
    }

    fn create_texture_gpu(&self, image: &RgbaImage) -> TextureGPU {
        TextureGPU::new(&self.device, &self.queue, &image)
    }

    fn create_material_gpu(&self, layout: &wgpu::BindGroupLayout, texture: TextureGPU) -> MaterialGPU {
        MaterialGPU::new(&self.device, &layout, texture)
    }
}

pub trait RenderSystem {
    fn render<'a>(&self, render_pass: &'a mut wgpu::RenderPass);
}

pub struct BasicRenderSystem {
    pipeline: wgpu::RenderPipeline,
    res_controller: Rc<ResourceController>,

    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,

    model_buffer: wgpu::Buffer,
    model_bind_group_layout: wgpu::BindGroupLayout,
    model_bind_group: wgpu::BindGroup,

    mesh: MeshGPU,
    material: MaterialGPU
}

impl BasicRenderSystem {
    pub fn new(res_controller: &Rc<ResourceController>, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = res_controller.device.create_shader_module(
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

        let texture_bind_group_layout = res_controller.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let fovy = 45.0f32.to_radians();
        let aspect = 1920.0 / 1080.0;    
        let near = 0.1;                  
        let far = 100.0;                 

        let proj = Mat4::perspective_rh(fovy, aspect, near, far);

        let eye = Vec3::new(0.0, 2.0, 5.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::Y;

        let view = Mat4::look_at_rh(eye, target, up);
        let projview = proj * view;
        
        // Camera Uniform //
        let camera_buffer = res_controller.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[projview]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let camera_bind_group_layout = res_controller.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera layout"),
        });

        let camera_bind_group = res_controller.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera bind group"),
        });

        // Model Uniform //
        let model_buffer = 
            res_controller.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&[glam::Mat4::IDENTITY]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let model_bind_group_layout = 
            res_controller.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("model layout"),
        });

        let model_bind_group = res_controller.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("model bind group"),
        });

        let pipeline_layout =
            res_controller.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &model_bind_group_layout, &texture_bind_group_layout],
                immediate_size: 0,
            });

        let pipeline =
            res_controller.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

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

        let mesh = Mesh {
            vertices: vec![
                Vertex { position: [-0.5, -0.5, 0.0], uv: [0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] },
                Vertex { position: [ 0.5, -0.5, 0.0], uv: [1.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] },
                Vertex { position: [ 0.5,  0.5, 0.0], uv: [1.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
                Vertex { position: [-0.5,  0.5, 0.0], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
            ],
            indices: vec![0, 1, 2, 2, 3, 0],
        };

        let img = image::open("resources/images/green.png").unwrap().fliph();
        let rgba = img.to_rgba8(); 
        let tex_gpu = res_controller.create_texture_gpu(&rgba);

        Self {
            pipeline,
            res_controller: res_controller.clone(),

            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,

            model_buffer,
            model_bind_group_layout,
            model_bind_group,

            mesh: res_controller.create_mesh_gpu(&mesh),
            material: res_controller.create_material_gpu(&texture_bind_group_layout, tex_gpu)
        }
    }
}

impl RenderSystem for BasicRenderSystem {
    fn render(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_bind_group(1, &self.model_bind_group, &[]);

        if let Some(vb) = &self.mesh.vertex_buffer {
            pass.set_vertex_buffer(0, vb.slice(..));
        }
        if let Some(ib) = &self.mesh.index_buffer {
            pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        }
        pass.set_bind_group(2, &self.material.texture_bind_group, &[]);
        pass.draw_indexed(0..self.mesh.index_count as u32, 0, 0..1);
    }
}