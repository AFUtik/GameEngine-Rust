use crate::render_service::{RenderingService, RenderState};
use crate::engine::EngineContext;

struct GameContext {
    tick: u32
}

pub trait RenderComponent {
    fn init_render(&mut self, render_service: &mut RenderingService) {}
    fn render(&mut self, render_service: &mut RenderingService, render_state: &mut RenderState) {}
}

pub trait LogicComponent {
    fn update(context: &GameContext);
}
