use vek::{Mat4, Quaternion, Transform, Vec3};

#[derive(Clone)]
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

    pub fn axis_x(&self) -> Vec3<f32> {
        self.transform.orientation * Vec3::unit_x()
    }

    pub fn axis_y(&self) -> Vec3<f32> {
        self.transform.orientation * Vec3::unit_y()
    }

    pub fn axis_z(&self) -> Vec3<f32> {
        self.transform.orientation * Vec3::unit_z()
    }

    pub fn set_orientation(&mut self, orientation: Quaternion<f32>) {
        self.transform.orientation = orientation;
    }

    pub fn rotate_around_unit_x(&mut self, angle_radians: f32) {
        self.transform
            .orientation
            .rotate_3d(angle_radians, Vec3::unit_x());
    }

    pub fn rotate_around_unit_y(&mut self, angle_radians: f32) {
        self.transform
            .orientation
            .rotate_3d(angle_radians, Vec3::unit_y());
    }

    pub fn rotate_around_unit_z(&mut self, angle_radians: f32) {
        self.transform
            .orientation
            .rotate_3d(angle_radians, Vec3::unit_z());
    }

    pub fn pitch(&mut self, angle_radians: f32) {
        self.transform
            .orientation
            .rotate_3d(angle_radians, self.transform.orientation * Vec3::unit_x());
    }

    pub fn yaw(&mut self, angle_radians: f32) {
        self.transform
            .orientation
            .rotate_3d(angle_radians, self.transform.orientation * Vec3::unit_y());
    }

    pub fn roll(&mut self, angle_radians: f32) {
        self.transform
            .orientation
            .rotate_3d(angle_radians, self.transform.orientation * Vec3::unit_z());
    }

    pub fn compute_view_matrix(&self) -> Mat4<f32> {
        let mut transform_matrix = Into::<Mat4<f32>>::into(self.transform);

        transform_matrix.invert_affine_transform_no_scale();

        transform_matrix
    }
}
