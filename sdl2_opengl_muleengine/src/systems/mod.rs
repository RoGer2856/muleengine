use vek::{Mat4, Transform};

pub mod renderer;
pub mod renderer_group_object;
pub mod renderer_layer_object;
pub mod renderer_pipeline_step_object;

pub(crate) struct RendererTransformObject {
    transform: Transform<f32, f32, f32>,
}

pub(crate) struct RendererCameraObject {
    transform: Transform<f32, f32, f32>,
}

impl RendererCameraObject {
    pub fn compute_view_matrix(&self) -> Mat4<f32> {
        let mut transform_matrix = Into::<Mat4<f32>>::into(self.transform);

        transform_matrix.invert_affine_transform_no_scale();

        transform_matrix
    }
}
