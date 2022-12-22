use std::sync::Arc;

use super::spectator_camera_controller::{self, SpectatorCameraInput};
use muleengine::{
    asset_container::AssetContainer,
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
    mesh_creator,
    prelude::ResultInspector,
    renderer::{
        renderer_client::RendererClient, renderer_pipeline_step::RendererPipelineStep,
        CameraHandler, RendererGroupHandler, RendererLayerHandler, RendererObjectHandler,
        TransformHandler,
    },
    service_container::ServiceContainer,
    system_container::System,
};
use tokio::sync::RwLock;
use vek::{Transform, Vec2, Vec3};

pub struct GameManager {
    first_tick: bool,
    service_container: ServiceContainer,
    inner: Arc<RwLock<Option<GameManagerPri>>>,
    spectator_camera_input: SpectatorCameraInput,
}

struct GameManagerPri {
    _skydome_camera_transform_handler: TransformHandler,
    _skydome_camera_handler: CameraHandler,

    _main_camera_transform_handler: TransformHandler,
    _main_camera_handler: CameraHandler,

    _skydome_renderer_layer_handler: RendererLayerHandler,
    _main_renderer_layer_handler: RendererLayerHandler,

    skydome_renderer_group_handler: RendererGroupHandler,
    main_renderer_group_handler: RendererGroupHandler,

    renderer_object_handlers: Vec<RendererObjectHandler>,

    renderer_client: RendererClient,
    asset_container: AssetContainer,
}

impl GameManager {
    pub fn new(
        service_container: ServiceContainer,
        spectator_camera_input: SpectatorCameraInput,
    ) -> Self {
        Self {
            first_tick: true,
            service_container,
            inner: Arc::new(RwLock::new(None)),
            spectator_camera_input,
        }
    }
}

impl GameManagerPri {
    pub async fn new(
        service_container: ServiceContainer,
        spectator_camera_input: SpectatorCameraInput,
    ) -> Self {
        let renderer_client = service_container
            .get_service::<RendererClient>()
            .unwrap()
            .read()
            .clone();

        let skydome_camera_transform_handler = renderer_client
            .create_transform(Transform::default())
            .await
            .unwrap();
        let skydome_camera_handler = renderer_client
            .create_camera(skydome_camera_transform_handler.clone())
            .await
            .unwrap();

        let main_camera_transform_handler = renderer_client
            .create_transform(Transform::default())
            .await
            .unwrap();
        let main_camera_handler = renderer_client
            .create_camera(main_camera_transform_handler.clone())
            .await
            .unwrap();

        let skydome_renderer_layer_handler = renderer_client
            .create_renderer_layer(skydome_camera_handler.clone())
            .await
            .unwrap();
        let main_renderer_layer_handler = renderer_client
            .create_renderer_layer(main_camera_handler.clone())
            .await
            .unwrap();

        let skydome_renderer_group_handler = renderer_client.create_renderer_group().await.unwrap();
        renderer_client
            .add_renderer_group_to_layer(
                skydome_renderer_group_handler.clone(),
                skydome_renderer_layer_handler.clone(),
            )
            .await
            .unwrap();

        let main_renderer_group_handler = renderer_client.create_renderer_group().await.unwrap();
        renderer_client
            .add_renderer_group_to_layer(
                main_renderer_group_handler.clone(),
                main_renderer_layer_handler.clone(),
            )
            .await
            .unwrap();

        renderer_client
            .set_renderer_pipeline(vec![
                RendererPipelineStep::Clear {
                    depth: true,
                    color: true,

                    viewport_start_ndc: Vec2::broadcast(0.0),
                    viewport_end_ndc: Vec2::broadcast(1.0),
                },
                RendererPipelineStep::Draw {
                    renderer_layer_handler: skydome_renderer_layer_handler.clone(),

                    viewport_start_ndc: Vec2::broadcast(0.0),
                    viewport_end_ndc: Vec2::broadcast(1.0),
                },
                RendererPipelineStep::Clear {
                    viewport_start_ndc: Vec2::broadcast(0.0),
                    viewport_end_ndc: Vec2::broadcast(1.0),
                    depth: true,
                    color: false,
                },
                RendererPipelineStep::Draw {
                    renderer_layer_handler: main_renderer_layer_handler.clone(),

                    viewport_start_ndc: Vec2::broadcast(0.0),
                    viewport_end_ndc: Vec2::broadcast(1.0),
                },
            ])
            .await
            .unwrap();

        tokio::spawn(spectator_camera_controller::run(
            renderer_client.clone(),
            skydome_camera_transform_handler.clone(),
            main_camera_transform_handler.clone(),
            spectator_camera_input.clone(),
        ));

        let mut ret = Self {
            _skydome_camera_transform_handler: skydome_camera_transform_handler,
            _skydome_camera_handler: skydome_camera_handler,

            _main_camera_handler: main_camera_handler,
            _main_camera_transform_handler: main_camera_transform_handler,

            _skydome_renderer_layer_handler: skydome_renderer_layer_handler,
            _main_renderer_layer_handler: main_renderer_layer_handler,

            skydome_renderer_group_handler,
            main_renderer_group_handler,

            renderer_object_handlers: Vec::new(),

            renderer_client,
            asset_container: service_container
                .get_service::<AssetContainer>()
                .unwrap()
                .read()
                .clone(),
        };

        ret.populate_with_objects().await;

        ret
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
                        log::warn!("Invalid mesh in scene, path = {scene_path}, msg = {e:?}")
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
            let spectator_camera_input = self.spectator_camera_input.clone();
            tokio::spawn(async move {
                tokio::spawn(async move {
                    let game_manager_pri =
                        GameManagerPri::new(service_container, spectator_camera_input).await;
                    *inner.write().await = Some(game_manager_pri);
                })
                .await
                .inspect_err(|e| {
                    log::error!("Error during initialization of GameManager, msg = {e}")
                })
                .unwrap();
            });
        }
    }
}
