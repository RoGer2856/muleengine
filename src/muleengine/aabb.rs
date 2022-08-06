use vek::{Transform, Vec3};

#[repr(C)]
pub struct AxisAlignedBoundingBox {
    min_vertex: Vec3<f32>,
    max_vertex: Vec3<f32>,
}

impl AxisAlignedBoundingBox {
    pub fn new(vertex: Vec3<f32>) -> Self {
        Self {
            min_vertex: vertex,
            max_vertex: vertex,
        }
    }

    pub fn get_min_vertex(&self) -> &Vec3<f32> {
        &self.min_vertex
    }

    pub fn get_max_vertex(&self) -> &Vec3<f32> {
        &self.max_vertex
    }

    pub fn add_vertex(&mut self, vertex: Vec3<f32>) {
        self.min_vertex = Vec3::partial_min(self.min_vertex, vertex);
        self.max_vertex = Vec3::partial_max(self.max_vertex, vertex);
    }

    pub fn collide(&self, other: &AxisAlignedBoundingBox) -> bool {
        let min_coordinates = Vec3::<f32>::partial_min(self.min_vertex, other.min_vertex);
        let max_coordinates = Vec3::<f32>::partial_max(self.max_vertex, other.max_vertex);

        let x_length =
            self.max_vertex.x - self.min_vertex.x + other.max_vertex.x - other.min_vertex.x;
        if max_coordinates.x - min_coordinates.x <= x_length {
            let y_length =
                self.max_vertex.y - self.min_vertex.y + other.max_vertex.y - other.min_vertex.y;
            if max_coordinates.y - min_coordinates.y <= y_length {
                let z_length =
                    self.max_vertex.z - self.min_vertex.z + other.max_vertex.z - other.min_vertex.z;
                if max_coordinates.z - min_coordinates.z <= z_length {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn apply_transform(&mut self, transform: &Transform<f32, f32, f32>) {
        let a = transform.orientation
            * Vec3::new(self.min_vertex.x, self.min_vertex.y, self.min_vertex.z);
        let b = transform.orientation
            * Vec3::new(self.min_vertex.x, self.min_vertex.y, self.max_vertex.z);
        let c = transform.orientation
            * Vec3::new(self.min_vertex.x, self.max_vertex.y, self.min_vertex.z);
        let d = transform.orientation
            * Vec3::new(self.min_vertex.x, self.max_vertex.y, self.max_vertex.z);
        let e = transform.orientation
            * Vec3::new(self.max_vertex.x, self.min_vertex.y, self.min_vertex.z);
        let f = transform.orientation
            * Vec3::new(self.max_vertex.x, self.min_vertex.y, self.max_vertex.z);
        let g = transform.orientation
            * Vec3::new(self.max_vertex.x, self.max_vertex.y, self.min_vertex.z);
        let h = transform.orientation
            * Vec3::new(self.max_vertex.x, self.max_vertex.y, self.max_vertex.z);

        self.min_vertex = Vec3::partial_min(
            a,
            Vec3::partial_min(
                b,
                Vec3::partial_min(
                    c,
                    Vec3::partial_min(
                        d,
                        Vec3::partial_min(e, Vec3::partial_min(f, Vec3::partial_min(g, h))),
                    ),
                ),
            ),
        );

        self.max_vertex = Vec3::partial_max(
            a,
            Vec3::partial_max(
                b,
                Vec3::partial_max(
                    c,
                    Vec3::partial_max(
                        d,
                        Vec3::partial_max(e, Vec3::partial_max(f, Vec3::partial_max(g, h))),
                    ),
                ),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use vek::Vec3;

    use super::AxisAlignedBoundingBox;

    #[test]
    fn no_collision() {
        let mut aabb0 = AxisAlignedBoundingBox::new(Vec3::new(0.0, 0.0, 0.0));
        aabb0.add_vertex(Vec3::new(1.0, 1.0, 1.0));

        let mut aabb1 = AxisAlignedBoundingBox::new(Vec3::new(1.1, 1.1, 1.1));
        aabb1.add_vertex(Vec3::new(2.0, 2.0, 2.0));

        let mut aabb2 = AxisAlignedBoundingBox::new(Vec3::new(-0.1, -0.1, -0.1));
        aabb2.add_vertex(Vec3::new(-2.0, -2.0, -2.0));

        let mut aabb3 = AxisAlignedBoundingBox::new(Vec3::new(0.0, 1.1, 1.1));
        aabb1.add_vertex(Vec3::new(1.0, 2.0, 2.0));

        let mut aabb4 = AxisAlignedBoundingBox::new(Vec3::new(1.1, 0.0, 1.1));
        aabb4.add_vertex(Vec3::new(2.0, 1.0, 2.0));

        let mut aabb5 = AxisAlignedBoundingBox::new(Vec3::new(1.1, 1.1, 0.0));
        aabb1.add_vertex(Vec3::new(2.0, 2.0, 1.0));

        assert!(!aabb0.collide(&aabb1));
        assert!(!aabb0.collide(&aabb2));
        assert!(!aabb0.collide(&aabb3));
        assert!(!aabb0.collide(&aabb4));
        assert!(!aabb0.collide(&aabb5));
    }

    #[test]
    fn collision() {
        let mut aabb0 = AxisAlignedBoundingBox::new(Vec3::new(0.0, 0.0, 0.0));
        aabb0.add_vertex(Vec3::new(1.0, 1.0, 1.0));

        let mut aabb1 = AxisAlignedBoundingBox::new(Vec3::new(-1.0, -1.0, -1.0));
        aabb1.add_vertex(Vec3::new(0.5, 0.5, 0.5));

        aabb0.collide(&aabb1);
    }
}
