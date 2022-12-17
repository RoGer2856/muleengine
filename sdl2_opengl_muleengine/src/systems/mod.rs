use vek::Transform;

pub mod renderer;
pub mod renderer_group_object;

struct RendererTransformObject {
    transform: Transform<f32, f32, f32>,
}
