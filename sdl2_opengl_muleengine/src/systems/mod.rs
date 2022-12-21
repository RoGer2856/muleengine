use vek::Transform;

pub mod renderer;
pub mod renderer_group_object;
pub mod renderer_layer_object;
pub mod renderer_pipeline_step_object;

struct RendererTransformObject {
    transform: Transform<f32, f32, f32>,
}

struct RendererCameraObject {
    transform: Transform<f32, f32, f32>,
}
