use crate::render_service::{RenderContext, RenderingService};
use crate::engine::EngineContext;

pub trait RenderComponent {
    fn init(&mut self, render_service: &mut RenderingService) {}
    fn render(&mut self, render_service: &mut RenderContext) {}
}