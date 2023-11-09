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
    skybox::add_skybox,
    tools::{game_object_builder::GameObjectBuilder, spawner_services::Spawner},
};

pub mod rigid_bodies;
pub mod skybox;
pub mod tools;

pub async fn populate_with_objects(spawner: &Arc<Spawner>) {
    add_skybox(spawner).await;
    // add_heightmap(spawner, "Assets/heightmap.png").await;

    // let wall_prefix = "wall11";
    // add_heightmap(spawner, &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Height.png"),).await;

    let wall_prefix = "wall10";
    add_heightmap(spawner, &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Height.png"),).await;

    add_simple_ground(spawner).await;
    add_physics_entities(spawner).await;

    add_sample_capsule(spawner).await;

    // let scene_path = "Assets/objects/MonkeySmooth.obj";
    let scene_path = "Assets/demo/wall/wallTextured.fbx";
    // let scene_path = "Assets/sponza/sponza.fbx";
    // let scene_path = "Assets/objects/skybox/Skybox.obj";
    add_scene_from_file(spawner, scene_path, Vec3::new(0.0, 0.0, -5.0)).await;
}

pub async fn add_heightmap(spawner: &Arc<Spawner>, heightmap_path: &str) {
    let position = Vec3::new(25.0, -2.0, -20.0);
    let scale = Vec3::new(50.0, 2.0, 50.0);

    let heightmap_image = spawner
        .asset_container
        .image_container()
        .write()
        .get_image(heightmap_path, spawner.asset_container.asset_reader())
        .unwrap();

    let heightmap = Arc::new(HeightMap::from_images(&heightmap_image, None).unwrap());

    let entity_builder = GameObjectBuilder::new(spawner)
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

pub async fn add_simple_ground(spawner: &Arc<Spawner>) {
    let dimensions = Vec3::new(50.0, 2.0, 50.0);
    let position = Vec3::new(-25.0, -2.0, -20.0);

    let mut material = Material::new();
    let wall_prefix = "wall10";
    // let wall_prefix = "wall04";
    // let wall_prefix = "wall02";
    let albedo_image = spawner
        .asset_container
        .image_container()
        .write()
        .get_image(
            &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Diffuse.png"),
            spawner.asset_container.asset_reader(),
        )
        .unwrap();
    let normal_image = spawner
        .asset_container
        .image_container()
        .write()
        .get_image(
            &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Normal.png"),
            spawner.asset_container.asset_reader(),
        )
        .unwrap();
    material.add_texture(MaterialTexture::new(
        albedo_image,
        muleengine::mesh::MaterialTextureType::Albedo,
        muleengine::mesh::TextureMapMode::Mirror,
        1.0,
        0,
    ));
    material.add_texture(MaterialTexture::new(
        normal_image,
        muleengine::mesh::MaterialTextureType::Normal,
        muleengine::mesh::TextureMapMode::Mirror,
        1.0,
        0,
    ));

    let entity_builder = GameObjectBuilder::new(spawner)
        .renderer_group_handler(
            spawner
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .shader("Assets/shaders/lit_normal")
        .await
        .transform(Transform::<f32, f32, f32> {
            position,
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Transform::<f32, f32, f32>::default()
        })
        .await
        .mesh(Arc::new(mesh_creator::rectangle3d::create(
            dimensions.x,
            dimensions.y,
            dimensions.z,
        )))
        .await
        .material(material)
        .await
        .simple_rigid_body(
            position,
            ColliderShape::Box {
                x: dimensions.x,
                y: dimensions.y,
                z: dimensions.z,
            },
            RigidBodyType::Static,
        )
        .build()
        .await;

    entity_builder.build();
}

async fn add_physics_entities(spawner: &Arc<Spawner>) {
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

pub async fn add_sample_capsule(spawner: &Arc<Spawner>) {
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

pub async fn add_scene_from_file(spawner: &Arc<Spawner>, scene_path: &str, position: Vec3<f32>) {
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
