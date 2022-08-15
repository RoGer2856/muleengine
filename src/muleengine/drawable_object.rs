use vek::Mat4;

pub trait DrawableObject {
    fn render(
        &mut self,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
        object_matrix: &Mat4<f32>,
    );
}
