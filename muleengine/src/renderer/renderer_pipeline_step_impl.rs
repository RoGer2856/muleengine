use std::sync::Arc;

use bytifex_utils::sync::types::ArcRwLock;
use vek::{Mat4, Vec2};

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

        compute_projection_matrix: Arc<dyn Fn(usize, usize) -> Mat4<f32> + Send + Sync>,
    },
}
