use vek::{Mat4, Transform};

pub(crate) struct GLCamera {
    pub transform: Transform<f32, f32, f32>,
}

impl GLCamera {
    pub fn compute_view_matrix(&self) -> Mat4<f32> {
        let mut transform_matrix = Into::<Mat4<f32>>::into(self.transform);

        transform_matrix.invert_affine_transform_no_scale();

        transform_matrix
    }
}
