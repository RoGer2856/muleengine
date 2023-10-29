use std::sync::Arc;

use entity_component::{EntityContainer, EntityId};
use muleengine::{
    bytifex_utils::result_option_inspect::ResultInspector, mesh_creator,
    renderer::renderer_system::renderer_decoupler, service_container::ServiceContainer,
};
use vek::{Transform, Vec3};

use crate::{
    physics::Rapier3dPhysicsEngineService, systems::renderer_configuration::RendererConfiguration,
};

pub async fn create_box(
    service_container: ServiceContainer,
    position: Vec3<f32>,
    dimensions: Vec3<f32>,
) -> EntityId {
    let physics_engine = service_container
        .get_service::<Rapier3dPhysicsEngineService>()
        .unwrap();

    let entity_container = service_container
        .get_service::<EntityContainer>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .as_ref()
        .clone();

    let renderer_client = service_container
        .get_service::<renderer_decoupler::Client>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .as_ref()
        .clone();

    let renderer_configuration = service_container
        .get_service::<RendererConfiguration>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap();

    let transform = Transform::<f32, f32, f32> {
        position,
        scale: dimensions,
        ..Transform::<f32, f32, f32>::default()
    };

    let mesh = Arc::new(mesh_creator::rectangle3d::create(1.0, 1.0, 1.0));

    let shader_handler = renderer_client
        .create_shader("Assets/shaders/lit_wo_normal".to_string())
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let material_handler = renderer_client
        .create_material(mesh.get_material().clone())
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let mesh_handler = renderer_client
        .create_mesh(mesh)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let transform_handler = renderer_client
        .create_transform(transform)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let renderer_object_handler = renderer_client
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

    renderer_client
        .add_renderer_object_to_group(
            renderer_object_handler.clone(),
            renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();

    let rigid_body_descriptor = physics_engine
        .write()
        .create_box_rigid_body(position, dimensions);

    entity_container
        .lock()
        .entity_builder()
        .with_component(renderer_object_handler)
        .with_component(transform_handler)
        .with_component(rigid_body_descriptor)
        .build()
}

pub async fn create_sphere(
    service_container: ServiceContainer,
    position: Vec3<f32>,
    radius: f32,
) -> EntityId {
    let physics_engine = service_container
        .get_service::<Rapier3dPhysicsEngineService>()
        .unwrap();

    let entity_container = service_container
        .get_service::<EntityContainer>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .as_ref()
        .clone();

    let renderer_client = service_container
        .get_service::<renderer_decoupler::Client>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .as_ref()
        .clone();

    let renderer_configuration = service_container
        .get_service::<RendererConfiguration>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap();

    let transform = Transform::<f32, f32, f32> {
        position,
        scale: Vec3::broadcast(radius * 2.0),
        ..Transform::<f32, f32, f32>::default()
    };

    let mesh = Arc::new(mesh_creator::sphere::create(0.5, 16));

    let shader_handler = renderer_client
        .create_shader("Assets/shaders/lit_wo_normal".to_string())
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let material_handler = renderer_client
        .create_material(mesh.get_material().clone())
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let mesh_handler = renderer_client
        .create_mesh(mesh)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let transform_handler = renderer_client
        .create_transform(transform)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
    let renderer_object_handler = renderer_client
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

    renderer_client
        .add_renderer_object_to_group(
            renderer_object_handler.clone(),
            renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();

    let rigid_body_descriptor = physics_engine
        .write()
        .create_sphere_rigid_body(position, radius);

    entity_container
        .lock()
        .entity_builder()
        .with_component(renderer_object_handler)
        .with_component(transform_handler)
        .with_component(rigid_body_descriptor)
        .build()
}
