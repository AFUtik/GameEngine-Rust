use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, Window, WindowAttributes},
};


use crate::component_system::RenderComponent;
use crate::render_service::{self, Handle, HandlesContainer, RenderObject, RenderService, RenderServiceRc};
use crate::render_system::{self, BasicRenderSystem, RenderSystem};
use crate::renderer::Renderer;
use crate::model::{self, Mesh, Transform, Vertex};
use crate::camera::Camera;

pub struct EngineContext {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer>,

    pub render_systems: Vec<Box<dyn RenderSystem>>,
    pub camera: Camera,

    camera_speed: f32,
    camera_sensivity: f32,

    pub mouse_delta: glam::Vec2,
    pub w_pressed: bool,
    pub a_pressed: bool,
    pub s_pressed: bool,
    pub d_pressed: bool,
    pub space_pressed: bool,
    pub shift_pressed: bool,
    //pub render_components: Vec<Box<dyn RenderComponent>>
}

impl EngineContext {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            render_systems: Vec::new(),
            camera: Camera::new(),

            camera_speed: 10.0,
            camera_sensivity: 0.008,

            mouse_delta: glam::Vec2::ZERO,
            w_pressed: false,
            a_pressed: false,
            s_pressed: false,
            d_pressed: false,
            space_pressed: false,
            shift_pressed: false,
        }
    }
}

struct SimpleRenderComponent {
    transform: model::Transform,

    render_service: Option<RenderServiceRc>,
    render_handles: Option<HandlesContainer>,
    object: Handle<render_service::RenderObject>
}

impl SimpleRenderComponent {
    fn new() -> Self {
        Self {
            transform: Transform::new(),
            render_service: None,
            render_handles: None,
            object: Handle::new(),
        }
    }
}

impl RenderComponent for SimpleRenderComponent {
    fn init(&mut self, render_service: &render_service::RenderServiceRc) {
        let mut render_handles = HandlesContainer::new(render_service);

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
        
        let mut render_service_mut = render_service.borrow_mut();
        let mesh_h  = render_service_mut.create_mesh(&mesh, &mut render_handles, None);
        let mat_h   = render_service_mut.create_material(&rgba, &mut render_handles, None);
        self.object = render_service_mut.register_render_object(mesh_h, mat_h, &mut render_handles, None);

        // Render service injection
        self.render_service = Some(render_service.clone());
        self.render_handles = Some(render_handles);
    }

    fn render(&mut self, ctx: &mut render_service::RenderContext) {
        ctx.service.render_object(self.object, &self.transform, ctx.state);
    }
}

impl EngineContext {
    pub fn init_render_systems(&mut self) {
        if let Some(renderer) = &self.renderer {
            let mut basic_system = Box::new(BasicRenderSystem::new(&renderer.controller, &renderer.config));

            self.camera.translate(&glam::Vec3 { x: 0.0, y: 0.0, z: 2.0 });
            self.camera.update();
            
            basic_system.add_render_component(Box::new(SimpleRenderComponent::new()));
            basic_system.init_components();

            self.render_systems.push(basic_system);
        }
    }

    pub fn render(&mut self) {
        const DT: f32 = 1.0/165.0;
        self.update_camera(DT);

        if let Some(renderer) = &mut self.renderer {
            renderer.render(&mut self.render_systems, &self.camera);
        }
    }

    pub fn update_camera(&mut self, dt: f32) {
        let mut move_dir = glam::Vec3::ZERO;

        let forward = self.camera.forward();
        let right   = self.camera.right();
        let up      = self.camera.up();

        if self.w_pressed { move_dir += forward; }
        if self.s_pressed { move_dir -= forward; }
        if self.a_pressed { move_dir -= right; }
        if self.d_pressed { move_dir += right; }
        if self.shift_pressed { move_dir -= up; }
        if self.space_pressed { move_dir += up; }

        if move_dir.length_squared() > 0.0 {
            self.camera.translate(&(move_dir.normalize() * self.camera_speed * dt));
        }

        if self.mouse_delta != glam::Vec2::ZERO {
            self.camera.rotate_yaw_pitch(
                -self.mouse_delta.x * self.camera_sensivity,
                -self.mouse_delta.y * self.camera_sensivity,
            );
            self.mouse_delta = glam::Vec2::ZERO;
        }

        self.camera.update();
    }

    pub fn lock_cursor(&mut self) {
        if let Some(window_arc) = &self.window {
            window_arc.set_cursor_visible(false);

            if window_arc.set_cursor_grab(CursorGrabMode::Locked).is_err() {
                window_arc.set_cursor_grab(CursorGrabMode::Confined).ok();
            }
        }
    }

    pub fn unlock_cursor(&mut self) {
        if let Some(window_arc) = &self.window {
            window_arc.set_cursor_visible(true);
            window_arc.set_cursor_grab(CursorGrabMode::None).ok();
        }
    }
}