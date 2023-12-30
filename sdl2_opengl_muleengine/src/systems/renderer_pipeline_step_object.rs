use std::sync::Arc;

use muleengine::bytifex_utils::sync::types::RcRwLock;
use vek::{Mat4, Vec2};

use super::renderer_layer_object::RendererLayerObject;

pub(crate) enum RendererPipelineStepObject {
    Clear {
        depth: bool,
        color: bool,

        viewport_start_ndc: Vec2<f32>,
        viewport_end_ndc: Vec2<f32>,
    },
    Draw {
        renderer_layer: RcRwLock<RendererLayerObject>,

        viewport_start_ndc: Vec2<f32>,
        viewport_end_ndc: Vec2<f32>,

        projection_matrix: Mat4<f32>,
        compute_projection_matrix: Arc<dyn Fn(usize, usize) -> Mat4<f32> + Send>,
    },
}
