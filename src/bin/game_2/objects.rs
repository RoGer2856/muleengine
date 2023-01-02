use std::sync::Arc;

use game_2::systems::renderer_configuration::RendererConfiguration;
use muleengine::{
    asset_container::AssetContainer,
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
    mesh_creator,
    prelude::{ArcRwLock, ResultInspector},
    renderer::{renderer_client::RendererClient, RendererObjectHandler},
    service_container::ServiceContainer,
};
use vek::{Transform, Vec3};

pub struct Objects {
    renderer_object_handlers: Vec<RendererObjectHandler>,
    renderer_configuration: ArcRwLock<RendererConfiguration>,
    renderer_client: RendererClient,
    asset_container: AssetContainer,
}

impl Objects {
    pub fn new(service_container: ServiceContainer) -> Self {
        Self {
            renderer_object_handlers: Vec::new(),
            renderer_configuration: service_container
                .get_service::<RendererConfiguration>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap(),
            renderer_client: service_container
                .get_service::<RendererClient>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .read()
                .clone(),
            asset_container: service_container
                .get_service::<AssetContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .read()
                .clone(),
        }
    }

    pub async fn populate_with_objects(&mut self) {
        self.add_skybox().await;

        let main_renderer_group_handler = self
            .renderer_configuration
            .read()
            .main_renderer_group_handler()
            .clone();

        {
            let mut transform = Transform::<f32, f32, f32>::default();

            let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));

            let shader_handler = self
                .renderer_client
                .create_shader("Assets/shaders/lit_wo_normal".to_string())
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap();
            let material_handler = self
                .renderer_client
                .create_material(mesh.get_material().clone())
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap();
            let mesh_handler = self
                .renderer_client
                .create_mesh(mesh)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap();
            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
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
                .unwrap();

            transform.position.x = -2.0;
            transform.position.z = -5.0;
            self.renderer_client
                .update_transform(transform_handler, transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
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
                    &self.asset_container.asset_reader().read(),
                    &mut self.asset_container.image_container().write(),
                )
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap();

            let shader_handler = self
                .renderer_client
                .create_shader("Assets/shaders/lit_normal".to_string())
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap();
            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap();
            for mesh in scene.meshes_ref().iter() {
                match &mesh {
                    Ok(mesh) => {
                        let material_handler = self
                            .renderer_client
                            .create_material(mesh.get_material().clone())
                            .await
                            .inspect_err(|e| log::error!("{e:?}"))
                            .unwrap();
                        let mesh_handler = self
                            .renderer_client
                            .create_mesh(mesh.clone())
                            .await
                            .inspect_err(|e| log::error!("{e:?}"))
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
                            .unwrap();
                    }
                    Err(e) => {
                        log::warn!("Invalid mesh in scene, path = {scene_path}, msg = {e:?}")
                    }
                }
            }
        }
    }

    pub async fn add_skybox(&mut self) {
        let transform = Transform::<f32, f32, f32>::default();

        let scene_path = "Assets/objects/skybox/Skybox.obj";
        let scene = self
            .asset_container
            .scene_container()
            .write()
            .get_scene(
                scene_path,
                &self.asset_container.asset_reader().read(),
                &mut self.asset_container.image_container().write(),
            )
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let skydome_renderer_group_handler = self
            .renderer_configuration
            .read()
            .skydome_renderer_group_handler()
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
                .unwrap();

            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap();

            for (index, texture_path) in texture_paths.iter().enumerate() {
                let material = Material {
                    textures: vec![MaterialTexture {
                        image: self
                            .asset_container
                            .image_container()
                            .write()
                            .get_image(texture_path, &self.asset_container.asset_reader().read()),
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
                    .unwrap();
                let mesh_handler = self
                    .renderer_client
                    .create_mesh(mesh)
                    .await
                    .inspect_err(|e| log::error!("{e:?}"))
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
                    .unwrap();
                self.renderer_object_handlers
                    .push(renderer_object_handler.clone());
                self.renderer_client
                    .add_renderer_object_to_group(
                        renderer_object_handler,
                        skydome_renderer_group_handler.clone(),
                    )
                    .await
                    .inspect_err(|e| log::error!("{e:?}"))
                    .unwrap();
            }
        } else {
            log::error!("Skybox does not contain exactly 6 meshes");
        }
    }
}
