use std::rc::Rc;
use std::sync::Arc;

use image::RgbaImage;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

use crate::{model::Mesh, render_system::{ResourceController, MaterialGPU, MeshGPU, RenderSystem, TextureGPU}};

pub struct Renderer {
    pub controller: Rc<ResourceController>,

    surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,

    // Render system that have different pipeline //
    systems: Vec<Box<dyn RenderSystem>>
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: Default::default(),
                experimental_features: Default::default(),
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            controller: Rc::new(ResourceController {device, queue}),
            surface,
            config,
            systems: Vec::new()
        }
    }

    pub fn create_render_system(&mut self, system: Box<dyn RenderSystem>) {
        self.systems.push(system);
    }

    pub fn render(&mut self) {
        let frame = self.surface
            .get_current_texture()
            .unwrap();

        let view = frame.texture
            .create_view(&Default::default());

        let mut encoder =
            self.controller.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor::default()
            );

        {
            let mut pass =
                encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {

                        label: None,

                        color_attachments: &[Some(
                            wgpu::RenderPassColorAttachment {

                                view: &view,
                                resolve_target: None,
                                depth_slice: None,

                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(
                                        wgpu::Color {
                                            r: 0.4, 
                                            g: 0.4, 
                                            b: 0.4, 
                                            a: 1.0
                                        }
                                    ),
                                    store: wgpu::StoreOp::Store,
                                },
                            }
                        )],

                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    }
                );   
            for system in self.systems.iter() {system.render(&mut pass);}
        }

        self.controller.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.controller.device, &self.config);
    }
}
