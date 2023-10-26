use bytifex_utils::sync::types::ArcRwLock;
use vek::Vec2;

use super::RendererLayer;

#[derive(Clone)]
pub enum RendererPipelineStepImpl {
    Clear {
        depth: bool,
        color: bool,

        viewport_start_ndc: Vec2<f32>,
        viewport_end_ndc: Vec2<f32>,
    },
    Draw {
        renderer_layer: ArcRwLock<dyn RendererLayer>,

        viewport_start_ndc: Vec2<f32>,
        viewport_end_ndc: Vec2<f32>,
    },
}
