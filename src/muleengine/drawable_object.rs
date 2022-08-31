use vek::{Mat4, Vec3};

pub trait DrawableObject {
    fn render(
        &self,
        eye_position: &Vec3<f32>,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
        object_matrix: &Mat4<f32>,
    );
}
