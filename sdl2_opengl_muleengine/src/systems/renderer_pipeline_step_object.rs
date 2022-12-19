use muleengine::prelude::ArcRwLock;
use vek::Vec2;

use super::renderer_layer_object::RendererLayerObject;

#[derive(Clone)]
pub enum RendererPipelineStepObject {
    Clear {
        depth: bool,
        color: bool,

        viewport_start_ndc: Vec2<f32>,
        viewport_end_ndc: Vec2<f32>,
    },
    Draw {
        renderer_layer: ArcRwLock<RendererLayerObject>,

        viewport_start_ndc: Vec2<f32>,
        viewport_end_ndc: Vec2<f32>,
    },
}
