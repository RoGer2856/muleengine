use std::sync::Arc;

use entity_component::{EntityContainer, EntityId};
use muleengine::{
    asset_container::AssetContainer,
    bytifex_utils::result_option_inspect::ResultInspector,
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
    mesh_creator,
    renderer::{renderer_system::renderer_decoupler, RendererObjectHandler},
    service_container::ServiceContainer,
};
use vek::{Transform, Vec3};

use crate::{
    game_objects::rigid_body_object::RigidBodyObject,
    systems::renderer_configuration::RendererConfiguration,
};

pub struct Objects {
    service_container: ServiceContainer,
    rigid_body_objects: Vec<RigidBodyObject>,
    renderer_object_handlers: Vec<RendererObjectHandler>,
    renderer_configuration: Arc<RendererConfiguration>,
    renderer_client: renderer_decoupler::Client,
    asset_container: AssetContainer,
    entity_container: Arc<EntityContainer>,
    entity_ids: Vec<EntityId>,
}

impl Objects {
    pub fn new(service_container: ServiceContainer) -> Self {
        Self {
            service_container: service_container.clone(),
            rigid_body_objects: Vec::new(),
            renderer_object_handlers: Vec::new(),
            renderer_configuration: service_container
                .get_service::<RendererConfiguration>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap(),
            renderer_client: service_container
                .get_service::<renderer_decoupler::Client>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            asset_container: service_container
                .get_service::<AssetContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            entity_container: service_container.get_or_insert_service(EntityContainer::new),
            entity_ids: Vec::new(),
        }
    }

    pub async fn populate_with_objects(&mut self) {
        self.add_skybox().await;
        self.add_physics_objects(self.service_container.clone())
            .await;

        let main_renderer_group_handler = self
            .renderer_configuration
            .main_renderer_group_handler()
            .await
            .clone();

        {
            let mut transform = Transform::<f32, f32, f32>::default();

            let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));

            let shader_handler = self
                .renderer_client
                .create_shader("Assets/shaders/lit_wo_normal".to_string())
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();
            let material_handler = self
                .renderer_client
                .create_material(mesh.get_material().clone())
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();
            let mesh_handler = self
                .renderer_client
                .create_mesh(mesh)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();
            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();
            let renderer_object_handler = self
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
            self.renderer_object_handlers
                .push(renderer_object_handler.clone());
            self.renderer_client
                .add_renderer_object_to_group(
                    renderer_object_handler,
                    main_renderer_group_handler.clone(),
                )
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();

            transform.position.x = -2.0;
            transform.position.z = -5.0;
            self.renderer_client
                .update_transform(transform_handler, transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();
        }

        {
            let mut transform = Transform::<f32, f32, f32>::default();
            transform.position.z = -5.0;

            // let scene_path = "Assets/objects/MonkeySmooth.obj";
            let scene_path = "Assets/demo/wall/wallTextured.fbx";
            // let scene_path = "Assets/sponza/sponza.fbx";
            // let scene_path = "Assets/objects/skybox/Skybox.obj";
            let scene = self
                .asset_container
                .scene_container()
                .write()
                .get_scene(
                    scene_path,
                    self.asset_container.asset_reader(),
                    &mut self.asset_container.image_container().write(),
                )
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap();

            let shader_handler = self
                .renderer_client
                .create_shader("Assets/shaders/lit_normal".to_string())
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();
            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();
            for mesh in scene.meshes_ref().iter() {
                match &mesh {
                    Ok(mesh) => {
                        let material_handler = self
                            .renderer_client
                            .create_material(mesh.get_material().clone())
                            .await
                            .inspect_err(|e| log::error!("{e:?}"))
                            .unwrap()
                            .unwrap();
                        let mesh_handler = self
                            .renderer_client
                            .create_mesh(mesh.clone())
                            .await
                            .inspect_err(|e| log::error!("{e:?}"))
                            .unwrap()
                            .unwrap();
                        let renderer_object_handler = self
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
                        self.renderer_object_handlers
                            .push(renderer_object_handler.clone());
                        self.renderer_client
                            .add_renderer_object_to_group(
                                renderer_object_handler,
                                main_renderer_group_handler.clone(),
                            )
                            .await
                            .inspect_err(|e| log::error!("{e:?}"))
                            .unwrap()
                            .unwrap();
                    }
                    Err(e) => {
                        log::warn!("Invalid mesh in scene, path = {scene_path}, msg = {e:?}")
                    }
                }
            }
        }
    }

    async fn add_physics_objects(&mut self, service_container: ServiceContainer) {
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
                        self.rigid_body_objects.push(
                            RigidBodyObject::create_box(
                                service_container.clone(),
                                position,
                                cube_dimensions,
                            )
                            .await,
                        );
                    } else {
                        self.rigid_body_objects.push(
                            RigidBodyObject::create_sphere(
                                service_container.clone(),
                                position,
                                sphere_radius,
                            )
                            .await,
                        );
                    }

                    is_cube = !is_cube;
                }
            }
        }
    }

    async fn add_skybox(&mut self) {
        let transform = Transform::<f32, f32, f32>::default();

        let scene_path = "Assets/objects/skybox/Skybox.obj";
        let scene = self
            .asset_container
            .scene_container()
            .write()
            .get_scene(
                scene_path,
                self.asset_container.asset_reader(),
                &mut self.asset_container.image_container().write(),
            )
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let skydome_renderer_group_handler = self
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

            let shader_handler = self
                .renderer_client
                .create_shader("Assets/shaders/unlit".to_string())
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();

            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();

            for (index, texture_path) in texture_paths.iter().enumerate() {
                let material = Material {
                    textures: vec![MaterialTexture {
                        image: self
                            .asset_container
                            .image_container()
                            .write()
                            .get_image(texture_path, self.asset_container.asset_reader()),
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

                let material_handler = self
                    .renderer_client
                    .create_material(material)
                    .await
                    .inspect_err(|e| log::error!("{e:?}"))
                    .unwrap()
                    .unwrap();
                let mesh_handler = self
                    .renderer_client
                    .create_mesh(mesh)
                    .await
                    .inspect_err(|e| log::error!("{e:?}"))
                    .unwrap()
                    .unwrap();
                let renderer_object_handler = self
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

                let entity_id = self
                    .entity_container
                    .lock()
                    .entity_builder()
                    .with_component(renderer_object_handler.clone())
                    .build();
                self.entity_ids.push(entity_id);

                self.renderer_client
                    .add_renderer_object_to_group(
                        renderer_object_handler,
                        skydome_renderer_group_handler.clone(),
                    )
                    .await
                    .inspect_err(|e| log::error!("{e:?}"))
                    .unwrap()
                    .unwrap();
            }
        } else {
            log::error!("Skybox does not contain exactly 6 meshes");
        }
    }
}
