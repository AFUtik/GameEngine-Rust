use glam::{Mat4, Vec3, Vec2};
use image::RgbaImage;

use std::rc::Rc;
use std::cell::RefCell;

use crate::model::{self, Mesh, Vertex, Transform};
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

// Render Scene //

pub struct Handle<T> {
    pub handle: u32,
    _phantom: PhantomData<T>,
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub enum RenderLayer {
    Opaque, 
    Transparent,
    Solid
}

pub struct DrawMesh {
    mesh: MeshGPU,
    ref_count: u32,
}

pub struct DrawMaterial {
    material: MaterialGPU,
    ref_count: u32,
}

pub struct RenderObject {
    transform: glam::Mat4,
    mesh: Handle<MeshGPU>,
    material: Handle<MaterialGPU>,
    layer: RenderLayer,
    visible: bool
}

pub struct DrawCommand {
    object_id: Handle<RenderObject>,
}

pub struct DrawCommandScissor {
    object_id: Handle<RenderObject>,
    scissor_rect: [u32; 4]
}

pub struct RenderScene {
    controller: Rc<ResourceController>,
    texture_bind_group_layout: Rc<wgpu::BindGroupLayout>,
    
    meshes:    Vec<DrawMesh>,
    materials: Vec<DrawMaterial>,
    renderables: Vec<RenderObject>, 
}

impl RenderScene {
    fn new(controller: &Rc<ResourceController>, texture_bind_group_layout: &Rc<wgpu::BindGroupLayout>) -> Self {
        Self {
            controller: controller.clone(),
            texture_bind_group_layout: texture_bind_group_layout.clone(),
            meshes:      Vec::new(),
            materials:   Vec::new(),
            renderables: Vec::new()
        }
    }

    //fn create_render_object(&mut self, mesh: &model::Mesh, image: &RgbaImage) -> RenderObject {
    //    let mesh_gpu = MeshGPU::new(&self.device, &mesh);

    //    let texture_gpu  = TextureGPU::new(&self.device, &self.queue, &image);
    //    let material_gpu = MaterialGPU::new(&self.device, &self.texture_bind_group_layout, texture_gpu);

    //    let obj_handle      = Handle::<RenderObject> {handle: self.renderables.len() as u32, _phantom: PhantomData};
    //    let mesh_handle     = Handle::<MeshGPU> {handle: self.meshes.len() as u32, _phantom: PhantomData};
    //    let material_handle = Handle::<MaterialGPU> {handle: self.materials.len() as u32, _phantom: PhantomData};

    //    self.renderables.push(obj_handle);
    //    self.meshes.push(mesh_gpu);
    //    self.materials.push(material_gpu);

    //    RenderObject { mesh: mesh_handle, material: material_handle}
    //}

    pub fn register_render_object(
        &mut self,
        mesh_handle: Handle<MeshGPU>,
        material_handle: Handle<MaterialGPU>, 
        render_layer: RenderLayer) -> Handle<RenderObject> 
    {
        let render_handle = Handle::<RenderObject> {handle: self.renderables.len() as u32, _phantom: PhantomData};
        self.renderables.push(RenderObject {transform: glam::Mat4::IDENTITY, mesh: mesh_handle, material: material_handle, layer: render_layer, visible: true});
        return render_handle;
    }

    pub fn transform_render_object(&mut self, transform: &Transform, object_id: Handle<RenderObject>) {
        let object = &mut self.renderables[object_id.handle as usize];
        object.transform =  glam::Mat4::from_translation(transform.pos.as_vec3())
                          * glam::Mat4::from_quat(transform.rot.as_quat())
                          * glam::Mat4::from_scale(transform.scl.as_vec3())
    }

    pub fn create_mesh(&mut self, mesh: &Mesh) -> Handle<MeshGPU> {
        let mesh_gpu = self.controller.create_mesh_gpu(mesh);
        let mesh_handle     = Handle::<MeshGPU> {handle: self.meshes.len() as u32, _phantom: PhantomData};

        self.meshes.push(DrawMesh { mesh: mesh_gpu, ref_count: 1 });

        mesh_handle
    }

    pub fn create_material(&mut self, albedo: &RgbaImage) -> Handle<MaterialGPU> {
        let tex_gpu  = self.controller.create_texture_gpu(albedo);
        let mate_gpu = self.controller.create_material_gpu(&self.texture_bind_group_layout, tex_gpu);

        let mate_handle     = Handle::<MaterialGPU> {handle: self.materials.len() as u32, _phantom: PhantomData};

        self.materials.push(DrawMaterial { material: mate_gpu, ref_count: 1 });

        mate_handle
    }

    pub fn ref_cnt_mesh_add(&mut self, handle: Handle<MeshGPU>) {
        self.meshes[handle.handle as usize].ref_count += 1;
    }

    pub fn ref_cnt_material_add(&mut self, handle: Handle<MaterialGPU>) {
        self.materials[handle.handle as usize].ref_count += 1;
    }

    pub fn delete_render_object(&mut self, handle: Handle<RenderObject>) {
        let mesh_handle     = self.renderables[handle.handle as usize].mesh;
        let material_handle = self.renderables[handle.handle as usize].material;

        self.delete_mesh(mesh_handle);
        self.delete_material(material_handle);
    }

    pub fn delete_mesh(&mut self, handle: Handle<MeshGPU>) {
        let mesh = &mut self.meshes[handle.handle as usize];
        mesh.ref_count-=1;

        if mesh.ref_count == 0 {
            self.meshes.remove(handle.handle as usize);
        }
    }

    pub fn delete_material(&mut self, handle: Handle<MaterialGPU>) {
        let mat = &mut self.materials[handle.handle as usize];
        mat.ref_count-=1;

        if mat.ref_count == 0 {
            self.materials.remove(handle.handle as usize);
        }
    }
}

pub struct MeshObject {
    transform: Rc<RefCell<Transform>>,
    render_id: Handle<RenderObject>,
}

impl MeshObject {
    pub fn new() -> Self {
        Self {
            transform: Rc::new(RefCell::new(Transform::new())),
            render_id: Handle::<RenderObject> { handle: u32::MAX, _phantom: PhantomData }
        }
    }
}

pub trait ObjectRenderer {
    fn init(&mut self, scene: &Rc<RefCell<RenderScene>>);

    fn make_draw_commands(&self, commands: &mut Vec<DrawCommand>);

    fn make_draw_commands_opaque(&self, commands: &mut Vec<DrawCommand>) {}

    fn make_draw_commands_transparent(&self, commands: &mut Vec<DrawCommand>) {}

    fn make_draw_commands_scissor(&self, commands: &mut Vec<DrawCommandScissor>) {}
}

pub struct BaseObjectRenderer {
    render_scene: Option<Rc<RefCell<RenderScene>>>,
    obj: MeshObject,
}

impl BaseObjectRenderer {
    pub fn new() -> Self {
        Self {
            render_scene: None,
            obj: MeshObject::new()
        }
    }
}

impl ObjectRenderer for BaseObjectRenderer {
    fn init(&mut self, scene: &Rc<RefCell<RenderScene>>) {
        self.render_scene = Some(scene.clone());
        
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
    
        let mut scene_mut = scene.borrow_mut();
        let mesh_h = scene_mut.create_mesh(&mesh);
        let mat_h  = scene_mut.create_material(&rgba);
        self.obj.render_id = scene_mut.register_render_object(mesh_h, mat_h, RenderLayer::Solid);
    }

    fn make_draw_commands(&self, commands: &mut Vec<DrawCommand>) {
        if let Some(scene) = &self.render_scene {

            let transform = self.obj.transform.borrow();

            scene.borrow_mut().transform_render_object(&transform, self.obj.render_id);
            commands.push(DrawCommand { object_id: self.obj.render_id });
        }
    }
}

pub trait RenderSystem {
    fn render(&mut self, render_pass: &mut wgpu::RenderPass);
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

    texture_bind_group_layout: Rc<wgpu::BindGroupLayout>,

    render_scene: Rc<RefCell<RenderScene>>,
    draw_commands: Vec<DrawCommand>,
    draw_commands_scissor: Vec<DrawCommandScissor>,
    object_renderers: Vec<Box<dyn ObjectRenderer>>
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

        let texture_bind_group_layout = Rc::new(res_controller.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        }));

        let fovy = 45.0f32.to_radians();
        let aspect = 1920.0 / 1080.0;    
        let near = 0.1;                  
        let far = 100.0;  

        let proj = Mat4::perspective_rh(fovy, aspect, near, far);
        
        let eye = Vec3::new(0.0, 2.0, 4.0);
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
        let model_matrix: [f32; 16] = Mat4::IDENTITY.to_cols_array();
        let model_buffer = 
            res_controller.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&model_matrix),
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
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_buffer.as_entire_binding(),
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

        Self {
            pipeline,
            res_controller: res_controller.clone(),

            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,

            model_buffer,
            model_bind_group_layout,
            model_bind_group,

            texture_bind_group_layout: texture_bind_group_layout.clone(),

            render_scene: Rc::new(RefCell::new(RenderScene::new(&res_controller, &texture_bind_group_layout))),
            draw_commands: Vec::new(),
            draw_commands_scissor: Vec::new(),
            object_renderers: Vec::new()
        }
    }

    pub fn register_object_renderer(&mut self, mut renderer: Box<dyn ObjectRenderer>) {
        renderer.init(&self.render_scene);
        self.object_renderers.push(renderer);
    }
}

impl RenderSystem for BasicRenderSystem {
    fn render(&mut self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_bind_group(1, &self.model_bind_group,  &[]);
        
        for renderer in self.object_renderers.iter() {
            renderer.make_draw_commands(&mut self.draw_commands);
        }

        let scene = self.render_scene.borrow();
        for draw in self.draw_commands.iter() {
            let obj = &scene.renderables[draw.object_id.handle as usize];
            let mesh = &scene.meshes[obj.mesh.handle as usize].mesh;
            let material = &scene.materials[obj.material.handle as usize].material;

            self.res_controller.queue.write_buffer(&self.model_buffer, 0, bytemuck::cast_slice(&obj.transform.to_cols_array()));
            if let Some(vb) = &mesh.vertex_buffer {
                pass.set_vertex_buffer(0, vb.slice(..));
            }
            if let Some(ib) = &mesh.index_buffer {
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
            }
            pass.set_bind_group(2, &material.texture_bind_group, &[]);
            pass.draw_indexed(0..mesh.index_count as u32, 0, 0..1);
        }
        
        self.draw_commands.clear();
    }
}