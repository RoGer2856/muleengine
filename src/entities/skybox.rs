use std::sync::Arc;

use muleengine::{
    bytifex_utils::result_option_inspect::ResultInspector,
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
};
use vek::{Transform, Vec3};

use super::spawner::Spawner;

pub async fn add_skybox(spawner: &Arc<Spawner>) {
    let transform = Transform::<f32, f32, f32>::default();

    let scene_path = "Assets/objects/skybox/Skybox.obj";
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

    let skydome_renderer_group_handler = spawner
        .renderer_configuration
        .skydome_renderer_group_handler()
        .await
        .clone();

    if scene.meshes_ref().len() == 6 {
        let texture_paths = [
            "Assets/objects/skybox/skyboxRight.png",
            "Assets/objects/skybox/skyboxLeft.png",
            "Assets/objects/skybox/skyboxTop.png",
            "Assets/objects/skybox/skyboxBottom.png",
            "Assets/objects/skybox/skyboxFront.png",
            "Assets/objects/skybox/skyboxBack.png",
        ];

        let shader_handler = spawner
            .renderer_client
            .create_shader("Assets/shaders/unlit".to_string())
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

        for (index, texture_path) in texture_paths.iter().enumerate() {
            let material = Material {
                textures: vec![MaterialTexture {
                    image: spawner
                        .asset_container
                        .image_container()
                        .write()
                        .get_image(texture_path, spawner.asset_container.asset_reader()),
                    texture_type: MaterialTextureType::Albedo,
                    texture_map_mode: TextureMapMode::Clamp,
                    blend: 0.0,
                    uv_channel_id: 0,
                }],
                opacity: 1.0,
                albedo_color: Vec3::broadcast(1.0),
                shininess_color: Vec3::broadcast(0.0),
                emissive_color: Vec3::broadcast(0.0),
            };

            let mesh = scene.meshes_ref()[index].as_ref().unwrap().clone();

            let material_handler = spawner
                .renderer_client
                .create_material(material)
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
                    skydome_renderer_group_handler.clone(),
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
    } else {
        log::error!("Skybox does not contain exactly 6 meshes");
    }
}
