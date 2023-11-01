use std::sync::Arc;

use entity_component::EntityId;
use muleengine::{bytifex_utils::result_option_inspect::ResultInspector, mesh_creator};
use vek::{Transform, Vec3};

use super::spawner::Spawner;

pub async fn create_box(
    spawner: &Arc<Spawner>,
    position: Vec3<f32>,
    dimensions: Vec3<f32>,
) -> EntityId {
    let transform = Transform::<f32, f32, f32> {
        position,
        scale: dimensions,
        ..Transform::<f32, f32, f32>::default()
    };

    let mesh = Arc::new(mesh_creator::rectangle3d::create(1.0, 1.0, 1.0));

    let shader_handler = spawner
        .renderer_client
        .create_shader("Assets/shaders/lit_wo_normal".to_string())
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let material_handler = spawner
        .renderer_client
        .create_material(mesh.get_material().clone())
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let mesh_handler = spawner
        .renderer_client
        .create_mesh(mesh)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let transform_handler = spawner
        .renderer_client
        .create_transform(transform)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let renderer_object_handler = spawner
        .renderer_client
        .create_renderer_object_from_mesh(
            mesh_handler,
            shader_handler,
            material_handler,
            transform_handler.clone(),
        )
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();

    spawner
        .renderer_client
        .add_renderer_object_to_group(
            renderer_object_handler.clone(),
            spawner
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();

    let rigid_body_descriptor = spawner
        .physics_engine
        .write()
        .create_box_rigid_body(position, dimensions);

    spawner
        .entity_container
        .lock()
        .entity_builder()
        .with_component(renderer_object_handler)
        .with_component(transform_handler)
        .with_component(rigid_body_descriptor)
        .build()
}

pub async fn create_sphere(spawner: &Arc<Spawner>, position: Vec3<f32>, radius: f32) -> EntityId {
    let transform = Transform::<f32, f32, f32> {
        position,
        scale: Vec3::broadcast(radius * 2.0),
        ..Transform::<f32, f32, f32>::default()
    };

    let mesh = Arc::new(mesh_creator::sphere::create(0.5, 16));

    let shader_handler = spawner
        .renderer_client
        .create_shader("Assets/shaders/lit_wo_normal".to_string())
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let material_handler = spawner
        .renderer_client
        .create_material(mesh.get_material().clone())
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let mesh_handler = spawner
        .renderer_client
        .create_mesh(mesh)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let transform_handler = spawner
        .renderer_client
        .create_transform(transform)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let renderer_object_handler = spawner
        .renderer_client
        .create_renderer_object_from_mesh(
            mesh_handler,
            shader_handler,
            material_handler,
            transform_handler.clone(),
        )
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();

    spawner
        .renderer_client
        .add_renderer_object_to_group(
            renderer_object_handler.clone(),
            spawner
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();

    let rigid_body_descriptor = spawner
        .physics_engine
        .write()
        .create_sphere_rigid_body(position, radius);

    spawner
        .entity_container
        .lock()
        .entity_builder()
        .with_component(renderer_object_handler)
        .with_component(transform_handler)
        .with_component(rigid_body_descriptor)
        .build()
}
