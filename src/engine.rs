use std::sync::Arc;
use winit::window::Window;


use crate::component_system::RenderComponent;
use crate::render_service::{self, Handle, RenderObject};
use crate::render_system::{self, BasicRenderSystem, RenderSystem};
use crate::renderer::Renderer;
use crate::model::{self, Mesh, Transform, Vertex};
use crate::camera::Camera;

pub struct EngineContext {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer>,

    pub render_systems: Vec<Box<dyn RenderSystem>>,
    //pub render_components: Vec<Box<dyn RenderComponent>>
}

impl EngineContext {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            render_systems: Vec::new()
        }
    }
}

struct SimpleRenderComponent {
    transform: model::Transform,
    object: Handle<render_service::RenderObject>
}

impl SimpleRenderComponent {
    fn new() -> Self {
        Self {
            transform: Transform::new(),
            object: Handle::<render_service::RenderObject>::new()
        }
    }
}

impl RenderComponent for SimpleRenderComponent {
    fn init(&mut self, render_service: &mut render_service::RenderingService) {
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
    
        let mesh_h  = render_service.create_mesh(&mesh);
        let mat_h   = render_service.create_material(&rgba);
        self.object = render_service.register_render_object(mesh_h, mat_h, render_service::RenderLayer::Solid);
    }

    fn render(&mut self, ctx: &mut render_service::RenderContext) {
        ctx.service.render_object(self.object, &self.transform, ctx.state);
    }
}

impl EngineContext {
    pub fn init_render_systems(&mut self) {
        if let Some(renderer) = &self.renderer {
            let mut basic_system = Box::new(BasicRenderSystem::new(&renderer.controller, &renderer.config));
            
            basic_system.camera.translate(&glam::Vec3 { x: 0.0, y: 0.0, z: 2.0 });
            basic_system.camera.update();
            
            basic_system.add_render_component(Box::new(SimpleRenderComponent::new()));
            basic_system.init_components();

            self.render_systems.push(basic_system);
        }
    }

    pub fn render(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            renderer.render(&mut self.render_systems);
        }
    }
}