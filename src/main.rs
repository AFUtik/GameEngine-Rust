mod component_system;
mod render_system;
mod renderer;
mod model;
mod engine;
mod render_service;
mod gpu_resources;
mod camera;

use engine::EngineContext;
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, Window, WindowAttributes},
};

impl ApplicationHandler for EngineContext {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop
            .create_window(WindowAttributes::default().with_title("wgpu window"))
            .unwrap());

        self.renderer = Some(pollster::block_on(renderer::Renderer::new(window.clone())));
        self.window   = Some(window);

        self.init_render_systems();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode, PhysicalKey};

                let pressed = event.state == ElementState::Pressed;

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.w_pressed = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.a_pressed = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.s_pressed = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.d_pressed = pressed,
                    PhysicalKey::Code(KeyCode::ShiftLeft) => self.shift_pressed = pressed,
                    PhysicalKey::Code(KeyCode::Space) => self.space_pressed = pressed,
                    PhysicalKey::Code(KeyCode::Tab) =>      self.lock_cursor(),
                    PhysicalKey::Code(KeyCode::CapsLock) => self.unlock_cursor(),
                    _ => {}
                }
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = &mut self.renderer {
                    state.resize(new_size.width, new_size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta = glam::Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = EngineContext::new();

    event_loop.run_app(&mut app).unwrap();
}