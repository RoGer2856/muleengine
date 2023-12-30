use std::sync::Arc;

use vek::{Mat4, Vec2};

use super::RendererLayerHandler;

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

        compute_projection_matrix: Arc<dyn Fn(usize, usize) -> Mat4<f32> + Send + Sync>,
    },
}
