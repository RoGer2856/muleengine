use muleengine::prelude::RcRwLock;
use vek::Vec2;

use super::renderer_layer_object::RendererLayerObject;

#[derive(Clone)]
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
    },
}
