use muleengine::renderer::RendererTransform;
use vek::Transform;

pub mod renderer;

struct RendererTransformImpl {
    transform: Transform<f32, f32, f32>,
}

impl RendererTransform for RendererTransformImpl {}
