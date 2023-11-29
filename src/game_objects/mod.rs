use std::sync::Arc;

use muleengine::{
    bytifex_utils::result_option_inspect::ResultInspector,
    heightmap::HeightMap,
    mesh::{Material, MaterialTexture},
    mesh_creator,
};
use vek::{Transform, Vec3};

use crate::{
    components::CurrentlyControlledCharacter,
    essential_services::EssentialServices,
    physics::{
        character_controller::CharacterLength, collider::ColliderShape, rigid_body::RigidBodyType,
    },
};

use self::{skybox::spawn_skybox, tools::game_object_builder::GameObjectBuilder};

pub mod rigid_bodies;
pub mod skybox;
pub mod tools;

pub async fn populate_with_objects(essentials: &Arc<EssentialServices>) {
    spawn_skybox(essentials).await;
    spawn_ground(essentials).await;
    spawn_player(essentials).await;
    spawn_physics_entities(essentials).await;

    spawn_sample_capsule(essentials).await;

    // let scene_path = "Assets/objects/MonkeySmooth.obj";
    let scene_path = "Assets/demo/wall/wallTextured.fbx";
    // let scene_path = "Assets/sponza/sponza.fbx";
    // let scene_path = "Assets/objects/skybox/Skybox.obj";
    spawn_scene_from_file(essentials, scene_path, Vec3::new(0.0, 0.0, -5.0)).await;
}

async fn spawn_ground(essentials: &Arc<EssentialServices>) {
    add_heightmap(
        essentials,
        Vec3::new(-25.0, -2.0, 0.0),
        "Assets/heightmap.png",
        None,
    )
    .await;

    let wall_prefix = "wall11";
    add_heightmap(
        essentials,
        Vec3::new(-25.0, -2.0, -50.0),
        &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Height.png"),
        Some(&format!(
            "Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Diffuse.png"
        )),
    )
    .await;

    add_heightmap(
        essentials,
        Vec3::new(25.0, -2.0, -50.0),
        &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Height.png"),
        None,
    )
    .await;

    let wall_prefix = "wall10";
    add_heightmap(
        essentials,
        Vec3::new(25.0, -2.0, 0.0),
        &format!("Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Height.png"),
        Some(&format!(
            "Assets/ADG_Textures/walls_vol1/{wall_prefix}/{wall_prefix}_Diffuse.png"
        )),
    )
    .await;
}

async fn add_heightmap(
    essentials: &Arc<EssentialServices>,
    position: Vec3<f32>,
    heightmap_path: &str,
    albedo_path: Option<&str>,
) {
    let scale = Vec3::new(50.0, 2.0, 50.0);

    let mut material = Material::new();
    if let Some(albedo_path) = albedo_path {
        let albedo_image = essentials
            .asset_container
            .image_container()
            .write()
            .get_image(albedo_path, essentials.asset_container.asset_reader())
            .unwrap();

        material.add_texture(MaterialTexture::new(
            albedo_image,
            muleengine::mesh::MaterialTextureType::Albedo,
            muleengine::mesh::TextureMapMode::Mirror,
            1.0,
            0,
        ));
    }

    let heightmap_image = essentials
        .asset_container
        .image_container()
        .write()
        .get_image(heightmap_path, essentials.asset_container.asset_reader())
        .unwrap();

    let heightmap = Arc::new(HeightMap::from_images(&heightmap_image, None).unwrap());

    let entity_builder = GameObjectBuilder::new(essentials)
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
            essentials
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

async fn spawn_physics_entities(essentials: &Arc<EssentialServices>) {
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
                    rigid_bodies::create_box(essentials, position, CUBE_DIMENSIONS).await;
                } else {
                    rigid_bodies::create_sphere(essentials, position, SPHERE_RADIUS).await;
                }

                is_cube = !is_cube;
            }
        }
    }
}

async fn spawn_sample_capsule(essentials: &Arc<EssentialServices>) {
    let entity_builder = GameObjectBuilder::new(essentials)
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
            essentials
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .build()
        .await;

    entity_builder.build();
}

async fn spawn_scene_from_file(
    essentials: &Arc<EssentialServices>,
    scene_path: &str,
    position: Vec3<f32>,
) {
    let game_object_builder = GameObjectBuilder::new(essentials)
        .renderer_group_handler(
            essentials
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

    let scene = essentials
        .asset_container
        .scene_container()
        .write()
        .get_scene(
            scene_path,
            essentials.asset_container.asset_reader(),
            &mut essentials.asset_container.image_container().write(),
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

async fn spawn_player(essentials: &Arc<EssentialServices>) {
    let position = Vec3::new(0.0, 0.0, 0.0);

    let capsule_radius = 0.3;
    let capsule_height = 1.8;

    let entity_builder = GameObjectBuilder::new(essentials)
        // .simple_rigid_body(
        //     position,
        //     ColliderShape::Capsule {
        //         radius: capsule_radius,
        //         height: capsule_height,
        //     },
        //     RigidBodyType::Dynamic,
        // )
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
            essentials
                .renderer_configuration
                .main_renderer_group_handler()
                .await
                .clone(),
        )
        .build()
        .await;

    let character_controller_builder = essentials
        .physics_engine
        .read()
        .character_controller_builder(ColliderShape::Capsule {
            radius: capsule_radius,
            height: capsule_height,
        });

    let character_controller_handler = character_controller_builder
        .mass(80.0)
        .margin(CharacterLength::Absolute(0.01))
        .max_slope_climb_angle(35.0)
        .min_slope_slide_angle(45.0)
        .autostep(false, CharacterLength::Absolute(0.3))
        .snap_to_ground(CharacterLength::Absolute(0.3))
        .build(&mut essentials.physics_engine.write());

    let entity_builder = entity_builder
        .with_component(character_controller_handler)
        .with_component(CurrentlyControlledCharacter);

    entity_builder.build();
}
