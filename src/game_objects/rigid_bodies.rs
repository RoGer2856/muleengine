use std::sync::Arc;

use entity_component::EntityId;
use muleengine::mesh_creator;
use vek::{Transform, Vec3};

use crate::{
    essential_services::EssentialServices,
    physics::{collider::ColliderShape, rigid_body::RigidBodyType},
};

use super::tools::game_object_builder::GameObjectBuilder;

pub async fn create_box(
    essentials: &Arc<EssentialServices>,
    position: Vec3<f32>,
    dimensions: Vec3<f32>,
    rigid_body_type: RigidBodyType,
) -> EntityId {
    let entity_builder = GameObjectBuilder::new(essentials)
        .renderer_group_handler(
            essentials
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .shader("Assets/shaders/lit_normal")
        .await
        .transform(Transform::<f32, f32, f32> {
            position,
            scale: dimensions,
            ..Transform::<f32, f32, f32>::default()
        })
        .await
        .mesh(Arc::new(mesh_creator::rectangle3d::create(1.0, 1.0, 1.0)))
        .await
        .simple_rigid_body(
            position,
            ColliderShape::Box {
                x: dimensions.x,
                y: dimensions.y,
                z: dimensions.z,
            },
            rigid_body_type,
        )
        .build()
        .await;

    entity_builder.build()
}

pub async fn create_sphere(
    essentials: &Arc<EssentialServices>,
    position: Vec3<f32>,
    radius: f32,
    rigid_body_type: RigidBodyType,
) -> EntityId {
    let entity_builder = GameObjectBuilder::new(essentials)
        .renderer_group_handler(
            essentials
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .shader("Assets/shaders/lit_normal")
        .await
        .transform(Transform::<f32, f32, f32> {
            position,
            scale: Vec3::broadcast(radius * 2.0),
            ..Transform::<f32, f32, f32>::default()
        })
        .await
        .mesh(Arc::new(mesh_creator::sphere::create(0.5, 16)))
        .await
        .simple_rigid_body(position, ColliderShape::Sphere { radius }, rigid_body_type)
        .build()
        .await;

    entity_builder.build()
}
