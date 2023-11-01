use std::sync::Arc;

use muleengine::{bytifex_utils::result_option_inspect::ResultInspector, mesh_creator};
use vek::{Transform, Vec3};

use self::{skybox::add_skybox, spawner::Spawner};

pub mod rigid_body_objects;
pub mod skybox;
pub mod spawner;

pub async fn populate_with_objects(spawner: &Arc<Spawner>) {
    add_skybox(spawner).await;
    add_physics_entities(spawner).await;
    add_sample_capsule(spawner).await;
    add_sample_mesh(spawner).await;
}

async fn add_physics_entities(spawner: &Arc<Spawner>) {
    let position_offset = Vec3::new(0.0, 0.0, -30.0);
    let cube_dimensions = Vec3::broadcast(1.0);
    let sphere_radius = 0.5;
    let space_between_objects = 3.0;
    let mut is_cube = true;
    for x in 0..5 {
        for y in 0..5 {
            for z in 0..5 {
                let position = Vec3::new(
                    x as f32 * space_between_objects,
                    y as f32 * space_between_objects,
                    z as f32 * space_between_objects,
                ) + position_offset;
                if is_cube {
                    rigid_body_objects::create_box(spawner, position, cube_dimensions).await;
                } else {
                    rigid_body_objects::create_sphere(spawner, position, sphere_radius).await;
                }

                is_cube = !is_cube;
            }
        }
    }
}

pub async fn add_sample_capsule(spawner: &Arc<Spawner>) {
    let mut transform = Transform::<f32, f32, f32>::default();

    let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));

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

    spawner
        .entity_container
        .lock()
        .entity_builder()
        .with_component(renderer_object_handler)
        .build();

    transform.position.x = -2.0;
    transform.position.z = -5.0;
    spawner
        .renderer_client
        .update_transform(transform_handler, transform)
        .await
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .unwrap();
}

pub async fn add_sample_mesh(spawner: &Arc<Spawner>) {
    let mut transform = Transform::<f32, f32, f32>::default();
    transform.position.z = -5.0;

    let renderer_group_handler = spawner
        .renderer_configuration
        .main_renderer_group_handler()
        .await
        .clone();

    // let scene_path = "Assets/objects/MonkeySmooth.obj";
    let scene_path = "Assets/demo/wall/wallTextured.fbx";
    // let scene_path = "Assets/sponza/sponza.fbx";
    // let scene_path = "Assets/objects/skybox/Skybox.obj";

    let scene = spawner
        .asset_container
        .scene_container()
        .write()
        .get_scene(
            scene_path,
            spawner.asset_container.asset_reader(),
            &mut spawner.asset_container.image_container().write(),
        )
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap();

    let shader_handler = spawner
        .renderer_client
        .create_shader("Assets/shaders/lit_normal".to_string())
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
    for mesh in scene.meshes_ref().iter() {
        match &mesh {
            Ok(mesh) => {
                let material_handler = spawner
                    .renderer_client
                    .create_material(mesh.get_material().clone())
                    .await
                    .inspect_err(|e| log::error!("{e:?}"))
                    .unwrap()
                    .unwrap();
                let mesh_handler = spawner
                    .renderer_client
                    .create_mesh(mesh.clone())
                    .await
                    .inspect_err(|e| log::error!("{e:?}"))
                    .unwrap()
                    .unwrap();
                let renderer_object_handler = spawner
                    .renderer_client
                    .create_renderer_object_from_mesh(
                        mesh_handler,
                        shader_handler.clone(),
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
                        renderer_group_handler.clone(),
                    )
                    .await
                    .inspect_err(|e| log::error!("{e:?}"))
                    .unwrap()
                    .unwrap();

                spawner
                    .entity_container
                    .lock()
                    .entity_builder()
                    .with_component(renderer_object_handler)
                    .build();
            }
            Err(e) => {
                log::warn!("Invalid mesh in scene, path = {scene_path}, msg = {e:?}")
            }
        }
    }
}
