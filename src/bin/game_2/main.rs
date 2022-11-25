#![allow(unstable_name_collisions)]

use std::sync::Arc;

use game_2::{
    main_loop::MainLoop, systems::spectator_camera_controller::SpectatorCameraControllerSystem,
};
use muleengine::{
    asset_container::AssetContainer,
    asset_reader::AssetReader,
    image_container::ImageContainer,
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
    mesh_creator,
    renderer::renderer_client::RendererClient,
    renderer::renderer_system::Renderer,
    scene_container::SceneContainer,
    service_container::ServiceContainer,
    system_container::SystemContainer,
    window_context::{Event, WindowContext},
};
use parking_lot::RwLock;
use sdl2_opengl_muleengine::{systems::renderer, GLProfile, Sdl2GLContext};
use vek::{Transform, Vec2, Vec3};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let multi_threaded = true;
    let rt = if multi_threaded {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    };

    rt.block_on(async_main());
}

async fn async_main() {
    let initial_window_dimensions = Vec2::new(800usize, 600usize);

    let sdl2_gl_context = Arc::new(RwLock::new(
        Sdl2GLContext::new(
            "game_2",
            initial_window_dimensions.x as u32,
            initial_window_dimensions.y as u32,
            GLProfile::Core,
            4,
            0,
        )
        .unwrap(),
    ));

    {
        let mut sdl2_gl_context = sdl2_gl_context.write();
        sdl2_gl_context.show_cursor(false);
        sdl2_gl_context.warp_mouse_normalized_screen_space(Vec2::new(0.5, 0.5));
    }

    let service_container = init_services();

    let (mut system_container, renderer_client) = {
        let mut system_container = SystemContainer::new();

        // creating renderer system
        let renderer_impl = renderer::Renderer::new(
            initial_window_dimensions,
            sdl2_gl_context.clone(),
            service_container
                .get_service::<AssetContainer>()
                .unwrap()
                .read()
                .clone(),
        );

        let renderer_system = Renderer::new(renderer_impl);
        let renderer_client = renderer_system.client();

        system_container.add_system(SpectatorCameraControllerSystem::new(
            renderer_client.clone(),
            sdl2_gl_context.clone(),
        ));

        // adding renderer system as the last system
        system_container.add_system(renderer_system);

        (system_container, renderer_client)
    };

    tokio::spawn(populate_with_objects(
        service_container,
        renderer_client.clone(),
    ));

    let event_receiver = sdl2_gl_context.read().event_receiver();

    const DESIRED_FPS: f32 = 30.0;
    let main_loop = MainLoop::new(DESIRED_FPS);
    'running: for delta_time_in_secs in main_loop.iter() {
        // handling events
        sdl2_gl_context.write().flush_events();

        while let Some(event) = event_receiver.pop() {
            log::debug!("EVENT = {event:?}");

            match event {
                Event::Resized { width, height } => {
                    renderer_client.set_window_dimensions(Vec2::new(width, height));
                }
                Event::Closed => break 'running,
                _ => (),
            }
        }

        system_container.tick(delta_time_in_secs);

        // putting the cursor back to the center of the window
        let window_center = sdl2_gl_context.read().window_dimensions() / 2;

        let mouse_pos = sdl2_gl_context.read().mouse_pos();
        if mouse_pos.x != window_center.x || mouse_pos.y != window_center.y {
            sdl2_gl_context
                .write()
                .warp_mouse_normalized_screen_space(Vec2::new(0.5, 0.5));
        }

        tokio::task::yield_now().await;
    }
}

pub fn init_services() -> ServiceContainer {
    let mut service_container = ServiceContainer::new();

    service_container.insert(AssetReader::new());
    service_container.insert(ImageContainer::new());
    service_container.insert(SceneContainer::new());
    service_container.insert(AssetContainer::new(&service_container));

    service_container
}

async fn populate_with_objects(
    service_container: ServiceContainer,
    renderer_client: RendererClient,
) {
    let asset_container_arc = service_container.get_service::<AssetContainer>().unwrap();
    let asset_container = asset_container_arc.read().clone();

    add_skybox(asset_container.clone(), renderer_client.clone()).await;

    {
        let mut transform = Transform::<f32, f32, f32>::default();
        transform.position.x = -2.0;
        transform.position.z = -5.0;

        let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));

        let shader_handler = renderer_client
            .create_shader("Assets/shaders/lit_wo_normal".to_string())
            .await
            .unwrap();
        let material_handler = renderer_client
            .create_material(mesh.get_material().clone())
            .await
            .unwrap();
        let mesh_handler = renderer_client.create_mesh(mesh).await.unwrap();
        let renderer_object_handler = renderer_client
            .create_renderer_object_from_mesh(mesh_handler, shader_handler, material_handler)
            .await
            .unwrap();
        renderer_client
            .add_renderer_object(renderer_object_handler, transform)
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
        let scene = asset_container
            .scene_container()
            .write()
            .get_scene(
                scene_path,
                &asset_container.asset_reader().read(),
                &mut asset_container.image_container().write(),
            )
            .unwrap();

        let shader_handler = renderer_client
            .create_shader("Assets/shaders/lit_normal".to_string())
            .await
            .unwrap();
        for mesh in scene.meshes_ref().iter() {
            match &mesh {
                Ok(mesh) => {
                    let material_handler = renderer_client
                        .create_material(mesh.get_material().clone())
                        .await
                        .unwrap();
                    let mesh_handler = renderer_client.create_mesh(mesh.clone()).await.unwrap();
                    let renderer_object_handler = renderer_client
                        .create_renderer_object_from_mesh(
                            mesh_handler,
                            shader_handler.clone(),
                            material_handler,
                        )
                        .await
                        .unwrap();
                    renderer_client
                        .add_renderer_object(renderer_object_handler, transform)
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

async fn add_skybox(asset_container: AssetContainer, renderer_client: RendererClient) {
    let transform = Transform::<f32, f32, f32>::default();

    let scene_path = "Assets/objects/skybox/Skybox.obj";
    let scene = asset_container
        .scene_container()
        .write()
        .get_scene(
            scene_path,
            &asset_container.asset_reader().read(),
            &mut asset_container.image_container().write(),
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

        let shader_handler = renderer_client
            .create_shader("Assets/shaders/unlit".to_string())
            .await
            .unwrap();

        for (index, texture_path) in texture_paths.iter().enumerate() {
            let material = Material {
                textures: vec![MaterialTexture {
                    image: asset_container
                        .image_container()
                        .write()
                        .get_image(texture_path, &asset_container.asset_reader().read()),
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

            let material_handler = renderer_client.create_material(material).await.unwrap();
            let mesh_handler = renderer_client.create_mesh(mesh).await.unwrap();
            let renderer_object_handler = renderer_client
                .create_renderer_object_from_mesh(
                    mesh_handler,
                    shader_handler.clone(),
                    material_handler,
                )
                .await
                .unwrap();
            renderer_client
                .add_renderer_object(renderer_object_handler, transform)
                .await
                .unwrap();
        }
    } else {
        panic!("Skybox does not contain exactly 6 meshes");
    }
}
