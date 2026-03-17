use std::sync::Arc;

use image::GenericImageView;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

use wgpu::{Texture, util::DeviceExt};

mod model;

mod render_system;
mod renderer;


use crate::{render_system::BasicRenderSystem, renderer::Renderer};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop
            .create_window(WindowAttributes::default().with_title("wgpu window"))
            .unwrap());

        self.renderer = Some(pollster::block_on(Renderer::new(window.clone())));
        self.window   = Some(window);

        if let Some(renderer) = &mut self.renderer {
            let rsystem = Box::new(BasicRenderSystem::new(&renderer.device, &renderer.queue, &renderer.config));
            renderer.create_render_system(rsystem);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = &mut self.renderer {
                    state.resize(new_size.width, new_size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.renderer {
                    state.render();
                }
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

    let mut app = App {
        window: None,
        renderer: None,
    };

    event_loop.run_app(&mut app).unwrap();
}