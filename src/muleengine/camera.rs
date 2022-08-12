use vek::{Mat4, Quaternion, Transform, Vec3};

pub struct Camera {
    pub transform: Transform<f32, f32, f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            transform: Transform {
                position: Vec3::new(0.0, 0.0, 0.0),
                orientation: Quaternion::identity(),
                scale: Vec3::broadcast(1.0),
            },
        }
    }

    pub fn move_by(&mut self, delta: Vec3<f32>) {
        self.transform.position += delta;
    }

    pub fn set_orientation(&mut self, orientation: Quaternion<f32>) {
        self.transform.orientation = orientation;
    }

    pub fn compute_view_matrix(&self) -> Mat4<f32> {
        let mut transform_matrix = Into::<Mat4<f32>>::into(self.transform);

        transform_matrix.invert_affine_transform_no_scale();

        transform_matrix
    }
}
