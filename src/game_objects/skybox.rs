use std::sync::Arc;

use muleengine::{
    bytifex_utils::result_option_inspect::ResultInspector,
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
};
use vek::{Transform, Vec3};

use super::tools::{game_object_builder::GameObjectBuilder, spawner_services::Spawner};

pub async fn add_skybox(spawner: &Arc<Spawner>) {
    let game_object_builder = GameObjectBuilder::new(spawner)
        .renderer_group_handler(
            spawner
                .renderer_configuration
                .skydome_renderer_group_handler()
                .await
                .clone(),
        )
        .shader("Assets/shaders/unlit")
        .await
        .transform(Transform::<f32, f32, f32>::default())
        .await;

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

    let texture_paths = [
        "Assets/objects/skybox/skyboxRight.png",
        "Assets/objects/skybox/skyboxLeft.png",
        "Assets/objects/skybox/skyboxTop.png",
        "Assets/objects/skybox/skyboxBottom.png",
        "Assets/objects/skybox/skyboxFront.png",
        "Assets/objects/skybox/skyboxBack.png",
    ];

    if scene.meshes_ref().len() == 6 {
        for (index, texture_path) in texture_paths.iter().enumerate() {
            let game_object_builder = game_object_builder.clone();

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

            let entity_builder = game_object_builder
                .mesh(mesh)
                .await
                .material(material)
                .await
                .build()
                .await;

            entity_builder.build();
        }
    } else {
        log::error!("Skybox does not contain exactly 6 meshes");
    }
}
