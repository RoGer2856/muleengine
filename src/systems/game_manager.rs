use std::sync::Arc;

use muleengine::{
    asset_container::AssetContainer,
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
    mesh_creator,
    renderer::{renderer_client::RendererClient, RendererGroupHandler, RendererObjectHandler},
    service_container::ServiceContainer,
    system_container::System,
};
use tokio::sync::RwLock;
use vek::{Transform, Vec3};

pub struct GameManager {
    first_tick: bool,
    service_container: ServiceContainer,
    inner: Arc<RwLock<Option<GameManagerPri>>>,
}

struct GameManagerPri {
    skydome_renderer_group_handler: RendererGroupHandler,
    main_renderer_group_handler: RendererGroupHandler,

    renderer_object_handlers: Vec<RendererObjectHandler>,

    renderer_client: RendererClient,
    asset_container: AssetContainer,
}

impl GameManager {
    pub fn new(service_container: ServiceContainer) -> Self {
        Self {
            first_tick: true,
            service_container,
            inner: Arc::new(RwLock::new(None)),
        }
    }
}

impl GameManagerPri {
    pub async fn new(service_container: ServiceContainer) -> Self {
        let renderer_client = service_container
            .get_service::<RendererClient>()
            .unwrap()
            .read()
            .clone();

        Self {
            skydome_renderer_group_handler: renderer_client.create_renderer_group().await.unwrap(),
            main_renderer_group_handler: renderer_client.create_renderer_group().await.unwrap(),

            renderer_object_handlers: Vec::new(),

            renderer_client,
            asset_container: service_container
                .get_service::<AssetContainer>()
                .unwrap()
                .read()
                .clone(),
        }
    }

    async fn populate_with_objects(&mut self) {
        self.add_skybox().await;

        {
            let mut transform = Transform::<f32, f32, f32>::default();

            let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));

            let shader_handler = self
                .renderer_client
                .create_shader("Assets/shaders/lit_wo_normal".to_string())
                .await
                .unwrap();
            let material_handler = self
                .renderer_client
                .create_material(mesh.get_material().clone())
                .await
                .unwrap();
            let mesh_handler = self.renderer_client.create_mesh(mesh).await.unwrap();
            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
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
                .unwrap();
            self.renderer_object_handlers
                .push(renderer_object_handler.clone());
            self.renderer_client
                .add_renderer_object_to_group(
                    renderer_object_handler,
                    self.main_renderer_group_handler.clone(),
                )
                .await
                .unwrap();

            transform.position.x = -2.0;
            transform.position.z = -5.0;
            self.renderer_client
                .update_transform(transform_handler, transform)
                .await
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
                .unwrap();

            let shader_handler = self
                .renderer_client
                .create_shader("Assets/shaders/lit_normal".to_string())
                .await
                .unwrap();
            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
                .unwrap();
            for mesh in scene.meshes_ref().iter() {
                match &mesh {
                    Ok(mesh) => {
                        let material_handler = self
                            .renderer_client
                            .create_material(mesh.get_material().clone())
                            .await
                            .unwrap();
                        let mesh_handler = self
                            .renderer_client
                            .create_mesh(mesh.clone())
                            .await
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
                            .unwrap();
                        self.renderer_object_handlers
                            .push(renderer_object_handler.clone());
                        self.renderer_client
                            .add_renderer_object_to_group(
                                renderer_object_handler,
                                self.main_renderer_group_handler.clone(),
                            )
                            .await
                            .unwrap();
                    }
                    Err(e) => {
                        log::warn!("Invalid mesh in scene, path = {scene_path}, error = {e:?}")
                    }
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
                &self.asset_container.asset_reader().read(),
                &mut self.asset_container.image_container().write(),
            )
            .unwrap();

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
                .unwrap();

            let transform_handler = self
                .renderer_client
                .create_transform(transform)
                .await
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
                    .unwrap();
                let mesh_handler = self.renderer_client.create_mesh(mesh).await.unwrap();
                let renderer_object_handler = self
                    .renderer_client
                    .create_renderer_object_from_mesh(
                        mesh_handler,
                        shader_handler.clone(),
                        material_handler,
                        transform_handler.clone(),
                    )
                    .await
                    .unwrap();
                self.renderer_object_handlers
                    .push(renderer_object_handler.clone());
                self.renderer_client
                    .add_renderer_object_to_group(
                        renderer_object_handler,
                        self.skydome_renderer_group_handler.clone(),
                    )
                    .await
                    .unwrap();
            }
        } else {
            panic!("Skybox does not contain exactly 6 meshes");
        }
    }
}

impl System for GameManager {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        if self.first_tick {
            self.first_tick = false;

            let inner = self.inner.clone();
            let service_container = self.service_container.clone();
            tokio::spawn(async move {
                *inner.write().await = Some(GameManagerPri::new(service_container.clone()).await)
            });
        }
    }
}
