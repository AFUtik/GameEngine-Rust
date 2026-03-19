use glam::{Mat4, Vec3, Vec2};
use image::RgbaImage;
use std:: {collections::HashMap, marker::PhantomData};
use wgpu::{Texture, util::DeviceExt};

// std
use std::rc::Rc;

// crate
use crate::model::{Mesh, Vertex, Transform};
use crate::component_system::RenderComponent;
use crate::render_service::{RenderingService, RenderState};
use crate::gpu_resources::ResourceController;

pub trait RenderSystem {
    fn render(&mut self, render_pass: &mut wgpu::RenderPass);

    fn get_render_service_mut(&mut self) -> &mut RenderingService;
    fn get_render_state(&mut self) -> &mut RenderState;
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

    rendering_service: RenderingService,
    render_state: RenderState,

    render_components: Vec<Box<dyn RenderComponent>>
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
                    buffers: &[Vertex::desc()],
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

            rendering_service: RenderingService::new(&res_controller, &texture_bind_group_layout),
            render_state: RenderState {draw_commands: Vec::new()},
            render_components: Vec::new()
        }
    }

    pub fn add_render_component(&mut self, component: Box<dyn RenderComponent>) {
        self.render_components.push(component);
    }

    pub fn init_components(&mut self) {
        for render_component in self.render_components.iter_mut() {
            render_component.init_render(&mut self.rendering_service);
        }
    }
}

impl RenderSystem for BasicRenderSystem {
    fn render(&mut self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_bind_group(1, &self.model_bind_group,  &[]);

        for component in self.render_components.iter_mut() {
            component.render(&mut self.rendering_service, &mut self.render_state);
        }
    
        for draw in self.render_state.draw_commands.iter() {
            let obj = &self.rendering_service.renderables[draw.object_id.handle as usize];
            let mesh = &self.rendering_service.meshes[obj.mesh.handle as usize].mesh;
            let material = &self.rendering_service.materials[obj.material.handle as usize].material;

            self.res_controller.queue.write_buffer(&self.model_buffer, 0, bytemuck::cast_slice(&draw.transform_mat.to_cols_array()));
            if let Some(vb) = &mesh.vertex_buffer {
                pass.set_vertex_buffer(0, vb.slice(..));
            }
            if let Some(ib) = &mesh.index_buffer {
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
            }
            pass.set_bind_group(2, &material.texture_bind_group, &[]);
            pass.draw_indexed(0..mesh.index_count as u32, 0, 0..1);
        }
        self.render_state.draw_commands.clear();
    }

    fn get_render_service_mut(&mut self) -> &mut RenderingService {
        &mut self.rendering_service
    }

    fn get_render_state(&mut self) -> &mut RenderState {
        &mut self.render_state
    }
}