use rapier3d::prelude::Collider;
use vek::Vec3;

#[derive(Clone)]
pub struct RigidBody {
    pub(super) colliders: Vec<Collider>,
    pub(super) position: Vec3<f32>,
    pub(super) rigid_body_type: RigidBodyType,
}

#[derive(Clone)]
pub enum RigidBodyType {
    Dynamic,
    Static,
    KinematicPositionBased,
    KinematicVelocityBased,
}

pub struct RigidBodyBuilder {
    rigid_body: RigidBody,
}

impl RigidBodyBuilder {
    pub(super) fn new(collider: Collider, rigid_body_type: RigidBodyType) -> Self {
        Self {
            rigid_body: RigidBody {
                colliders: vec![collider],
                position: Vec3::broadcast(0.0),
                rigid_body_type,
            },
        }
    }

    pub fn position(mut self, position: Vec3<f32>) -> Self {
        self.rigid_body.position = position;
        self
    }

    pub fn with_collider(mut self, collider: Collider) -> Self {
        self.rigid_body.colliders.push(collider);
        self
    }

    pub fn build(self) -> RigidBody {
        self.rigid_body
    }
}
