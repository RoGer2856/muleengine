use vek::Vec2;

use super::RendererLayerHandler;

#[derive(Debug, Clone)]
pub enum RendererPipelineStep {
    Clear {
        depth: bool,
        color: bool,

        viewport_start_ndc: Vec2<f32>,
        viewport_end_ndc: Vec2<f32>,
    },
    Draw {
        renderer_layer_handler: RendererLayerHandler,

        viewport_start_ndc: Vec2<f32>,
        viewport_end_ndc: Vec2<f32>,
    },
}
