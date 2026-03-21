use crate::render_service::{self, RenderContext, RenderServiceRc, HandlesContainer};
use crate::engine::EngineContext;

pub trait RenderComponent {
    fn init(&mut self, render_service: &RenderServiceRc) {}

    fn render(&mut self, context: &mut RenderContext) {}
}