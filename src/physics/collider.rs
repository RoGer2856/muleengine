use std::sync::Arc;

use muleengine::{aabb::AxisAlignedBoundingBox, heightmap::HeightMap};
use rapier3d::prelude::{
    nalgebra::{self, *},
    Collider, ColliderBuilder as RapierColliderBuilder, ColliderShape as RapierColliderShape,
    Vector,
};
use vek::Vec3;

pub enum ColliderShape {
    Capsule {
        radius: f32,
        height: f32,
    },
    Cone {
        radius: f32,
        height: f32,
    },
    Cylinder {
        radius: f32,
        height: f32,
    },
    Box {
        x: f32,
        y: f32,
        z: f32,
    },
    Sphere {
        radius: f32,
    },
    Heightmap {
        heightmap: Arc<HeightMap>,
        scale: Vec3<f32>,
    },
}

impl ColliderShape {
    pub fn compute_aabb(&self) -> AxisAlignedBoundingBox {
        match self {
            ColliderShape::Capsule { radius, height } => {
                let half_height = height / 2.0;
                let mut aabb = AxisAlignedBoundingBox::new(Vec3::new(
                    -radius,
                    -(half_height + radius),
                    -radius,
                ));
                aabb.add_vertex(Vec3::new(*radius, half_height + radius, *radius));
                aabb
            }
            ColliderShape::Cone { radius, height } => {
                let half_height = height / 2.0;
                let mut aabb =
                    AxisAlignedBoundingBox::new(Vec3::new(-radius, -half_height, -radius));
                aabb.add_vertex(Vec3::new(*radius, half_height, *radius));
                aabb
            }
            ColliderShape::Cylinder { radius, height } => {
                let half_height = height / 2.0;
                let mut aabb =
                    AxisAlignedBoundingBox::new(Vec3::new(-radius, -half_height, -radius));
                aabb.add_vertex(Vec3::new(*radius, half_height, *radius));
                aabb
            }
            ColliderShape::Box { x, y, z } => {
                let hx = x / 2.0;
                let hy = y / 2.0;
                let hz = z / 2.0;
                let mut aabb = AxisAlignedBoundingBox::new(Vec3::new(-hx, -hy, -hz));
                aabb.add_vertex(Vec3::new(hx, hy, hz));
                aabb
            }
            ColliderShape::Sphere { radius } => {
                let mut aabb = AxisAlignedBoundingBox::new(Vec3::new(-radius, -radius, -radius));
                aabb.add_vertex(Vec3::new(*radius, *radius, *radius));
                aabb
            }
            ColliderShape::Heightmap {
                heightmap: _,
                scale,
            } => {
                let mut aabb = AxisAlignedBoundingBox::new(Vec3::new(0.0, 0.0, 0.0));
                aabb.add_vertex(Vec3::new(scale.x, scale.y, scale.z));
                aabb
            }
        }
    }

    pub fn as_rapier_collider_shape(&self) -> RapierColliderShape {
        match self {
            ColliderShape::Capsule { radius, height } => {
                RapierColliderShape::capsule_y(*height / 2.0 - radius, *radius)
            }
            ColliderShape::Cone { radius, height } => {
                RapierColliderShape::cone(*height / 2.0, *radius)
            }
            ColliderShape::Cylinder { radius, height } => {
                RapierColliderShape::cylinder(*height / 2.0, *radius)
            }
            ColliderShape::Box { x, y, z } => {
                RapierColliderShape::cuboid(x / 2.0, y / 2.0, z / 2.0)
            }
            ColliderShape::Sphere { radius } => RapierColliderShape::ball(*radius),
            ColliderShape::Heightmap { heightmap, scale } => {
                let scale = Vector::new(scale.x, scale.y, scale.z);
                let heights = DMatrix::from_fn(
                    heightmap.get_row_count(),
                    heightmap.get_column_count(),
                    |x, y| heightmap.get_height_map()[y][x],
                );
                RapierColliderShape::heightfield(heights, scale)
            }
        }
    }
}

pub struct ColliderBuilder {
    position: Vec3<f32>,
    shape: ColliderShape,
    is_sensor: bool,
}

impl ColliderBuilder {
    pub(super) fn new(shape: ColliderShape) -> Self {
        Self {
            position: Vec3::broadcast(0.0),
            shape,
            is_sensor: false,
        }
    }

    pub fn position(mut self, position: Vec3<f32>) -> Self {
        self.position = position;
        self
    }

    pub fn is_sensor(mut self, is_sensor: bool) -> Self {
        self.is_sensor = is_sensor;
        self
    }

    pub fn build(self) -> Collider {
        let mut position_offset = Vec3::zero();

        let rapier_shape = self.shape.as_rapier_collider_shape();
        if let ColliderShape::Heightmap {
            heightmap: _,
            scale,
        } = &self.shape
        {
            position_offset.y = -scale.y / 2.0;
        };

        RapierColliderBuilder::new(rapier_shape)
            .translation(vector![
                self.position.x + position_offset.x,
                self.position.y + position_offset.y,
                self.position.z + position_offset.z
            ])
            .sensor(self.is_sensor)
            .build()
    }
}
