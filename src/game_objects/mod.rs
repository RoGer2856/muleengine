use std::sync::Arc;

use muleengine::{
    bytifex_utils::result_option_inspect::ResultInspector,
    heightmap::HeightMap,
    mesh::{Material, MaterialTexture},
    mesh_creator,
};
use vek::{Transform, Vec3};

use crate::physics::{ColliderShape, RigidBodyType};

use self::{
    skybox::spawn_skybox,
    tools::{game_object_builder::GameObjectBuilder, spawner_services::Spawner},
};

pub mod rigid_bodies;
pub mod skybox;
pub mod tools;

pub async fn populate_with_objects(spawner: &Arc<Spawner>) {
    spawn_skybox(spawner).await;
    spawn_ground(spawner).await;
    spawn_player(spawner).await;
    spawn_physics_entities(spawner).await;

    spawn_sample_capsule(spawner).await;

    // let scene_path = "Assets/objects/MonkeySmooth.obj";
    let scene_path = "Assets/demo/wall/wallTextured.fbx";
    // let scene_path = "Assets/sponza/sponza.fbx";
    // let scene_path = "Assets/objects/skybox/Skybox.obj";
    spawn_scene_from_file(spawner, scene_path, Vec3::new(0.0, 0.0, -5.0)).await;
}

async fn spawn_ground(spawner: &Arc<Spawner>) {
    add_heightmap(
        spawner,
        Vec3::new(-25.0, -2.0, 0.0),
        "Assets/heightmap.png",
        None,
    )
    .await;

    let wall_prefix = "wall11";
    add_heightmap(
        spawner,
        Vec3::new(-25.0, -2.0, -50.0),
        &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Height.png"),
        Some(&format!(
            "Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Diffuse.png"
        )),
    )
    .await;

    add_heightmap(
        spawner,
        Vec3::new(25.0, -2.0, -50.0),
        &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Height.png"),
        None,
    )
    .await;

    let wall_prefix = "wall10";
    add_heightmap(
        spawner,
        Vec3::new(25.0, -2.0, 0.0),
        &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Height.png"),
        Some(&format!(
            "Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Diffuse.png"
        )),
    )
    .await;
}

async fn add_heightmap(
    spawner: &Arc<Spawner>,
    position: Vec3<f32>,
    heightmap_path: &str,
    albedo_path: Option<&str>,
) {
    let scale = Vec3::new(50.0, 2.0, 50.0);

    let mut material = Material::new();
    if let Some(albedo_path) = albedo_path {
        let albedo_image = spawner
            .asset_container
            .image_container()
            .write()
            .get_image(albedo_path, spawner.asset_container.asset_reader())
            .unwrap();

        material.add_texture(MaterialTexture::new(
            albedo_image,
            muleengine::mesh::MaterialTextureType::Albedo,
            muleengine::mesh::TextureMapMode::Mirror,
            1.0,
            0,
        ));
    }

    let heightmap_image = spawner
        .asset_container
        .image_container()
        .write()
        .get_image(heightmap_path, spawner.asset_container.asset_reader())
        .unwrap();

    let heightmap = Arc::new(HeightMap::from_images(&heightmap_image, None).unwrap());

    let entity_builder = GameObjectBuilder::new(spawner)
        .material(material)
        .await
        .mesh(Arc::new(mesh_creator::heightmap::create(&heightmap)))
        .await
        .shader("Assets/shaders/lit_normal")
        .await
        .transform(Transform {
            position,
            scale,
            ..Default::default()
        })
        .await
        .renderer_group_handler(
            spawner
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .simple_rigid_body(
            position,
            ColliderShape::Heightmap { heightmap, scale },
            RigidBodyType::Static,
        )
        .build()
        .await;

    entity_builder.build();
}

async fn spawn_physics_entities(spawner: &Arc<Spawner>) {
    const OBJECT_COUNT: Vec3<usize> = Vec3 { x: 5, y: 5, z: 5 };

    const OBJECT_SIZE: f32 = 1.0;
    const CUBE_DIMENSIONS: Vec3<f32> = Vec3 {
        x: OBJECT_SIZE,
        y: OBJECT_SIZE,
        z: OBJECT_SIZE,
    };
    const SPHERE_RADIUS: f32 = OBJECT_SIZE / 2.0;
    const SPACE_BETWEEN_OBJECTS: f32 = 3.0;

    const CENTER_OF_MASS: Vec3<f32> = Vec3::new(
        (OBJECT_COUNT.x as f32 - 1.0) * SPACE_BETWEEN_OBJECTS / 2.0,
        (OBJECT_COUNT.y as f32 - 1.0) * SPACE_BETWEEN_OBJECTS / 2.0,
        (OBJECT_COUNT.z as f32 - 1.0) * SPACE_BETWEEN_OBJECTS / 2.0,
    );

    const POSITION_OFFSET: Vec3<f32> = Vec3::new(-CENTER_OF_MASS.x, 0.0, -30.0);

    let mut is_cube = true;
    for x in 0..OBJECT_COUNT.x {
        for y in 0..OBJECT_COUNT.y {
            for z in 0..OBJECT_COUNT.z {
                let position = Vec3::new(
                    x as f32 * SPACE_BETWEEN_OBJECTS,
                    y as f32 * SPACE_BETWEEN_OBJECTS,
                    z as f32 * SPACE_BETWEEN_OBJECTS,
                ) + POSITION_OFFSET;
                if is_cube {
                    rigid_bodies::create_box(spawner, position, CUBE_DIMENSIONS).await;
                } else {
                    rigid_bodies::create_sphere(spawner, position, SPHERE_RADIUS).await;
                }

                is_cube = !is_cube;
            }
        }
    }
}

async fn spawn_sample_capsule(spawner: &Arc<Spawner>) {
    let entity_builder = GameObjectBuilder::new(spawner)
        .mesh(Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16)))
        .await
        .shader("Assets/shaders/lit_wo_normal")
        .await
        .transform(Transform {
            position: Vec3::new(-2.0, 0.0, -5.0),
            ..Default::default()
        })
        .await
        .renderer_group_handler(
            spawner
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .build()
        .await;

    entity_builder.build();
}

async fn spawn_scene_from_file(spawner: &Arc<Spawner>, scene_path: &str, position: Vec3<f32>) {
    let game_object_builder = GameObjectBuilder::new(spawner)
        .renderer_group_handler(
            spawner
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .shader("Assets/shaders/lit_normal")
        .await
        .transform(Transform {
            position,
            ..Default::default()
        })
        .await;

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

    for mesh in scene.meshes_ref().iter() {
        match &mesh {
            Ok(mesh) => {
                let game_object_builder = game_object_builder.clone();
                let entity_builder = game_object_builder.mesh(mesh.clone()).await.build().await;
                entity_builder.build();
            }
            Err(e) => {
                log::warn!("Invalid mesh in scene, path = {scene_path}, msg = {e:?}")
            }
        }
    }
}

async fn spawn_player(spawner: &Arc<Spawner>) {
    let position = Vec3::new(0.0, 0.0, 0.0);

    let capsule_radius = 0.3;
    let capsule_height = 1.8;

    let entity_builder = GameObjectBuilder::new(spawner)
        .simple_rigid_body(
            position,
            ColliderShape::Capsule {
                radius: capsule_radius,
                height: capsule_height,
            },
            RigidBodyType::Dynamic,
        )
        .mesh(Arc::new(mesh_creator::capsule::create(
            capsule_radius,
            capsule_height,
            16,
        )))
        .await
        .shader("Assets/shaders/lit_wo_normal")
        .await
        .transform(Transform {
            position,
            ..Default::default()
        })
        .await
        .renderer_group_handler(
            spawner
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .build()
        .await;

    entity_builder.build();
}
