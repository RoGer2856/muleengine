#![allow(unstable_name_collisions)]

use std::sync::Arc;

use game_2::{
    main_loop::MainLoop,
    muleengine::{
        assets_reader::AssetsReader,
        image_container::ImageContainer,
        mesh::MaterialTextureType,
        mesh_creator, renderer,
        scene_container::SceneContainer,
        service_container::ServiceContainer,
        system_container::SystemContainer,
        window_context::{Event, WindowContext},
    },
    sdl2_opengl_engine::{
        drawable_object_creator::DrawableObjectCreator,
        gl_material::{GLMaterial, GLMaterialTexture},
        opengl_utils::texture_2d::GLTextureMapMode,
        systems::renderer as gl_renderer,
        GLProfile, Sdl2GLContext,
    },
    systems::spectator_camera_controller::SpectatorCameraControllerSystem,
};
use parking_lot::RwLock;
use renderer::RendererClient;
use vek::{Transform, Vec2, Vec3};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
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

        let service_container = {
            let mut service_container = ServiceContainer::new();

            service_container.insert(AssetsReader::new());
            service_container.insert(ImageContainer::new());
            service_container.insert(SceneContainer::new());
            service_container.insert(DrawableObjectCreator::new(&service_container));

            service_container
        };

        let (mut system_container, renderer_client) = {
            let mut system_container = SystemContainer::new();

            // creating renderer system
            let renderer_system =
                gl_renderer::Renderer::new(initial_window_dimensions, sdl2_gl_context.clone());
            let renderer_client = renderer_system.client();

            system_container.add_system(SpectatorCameraControllerSystem::new(
                renderer_client.clone(),
                sdl2_gl_context.clone(),
            ));

            // adding renderer system as the last system
            system_container.add_system(renderer_system);

            (system_container, renderer_client)
        };

        populate_with_objects(service_container.clone(), renderer_client.clone());

        const DESIRED_FPS: f32 = 30.0;

        let event_receiver = sdl2_gl_context.read().event_receiver();

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
        }
    });
}

fn populate_with_objects(
    service_container: ServiceContainer,
    renderer_client: Box<dyn RendererClient>,
) {
    let drawable_object_creator = service_container
        .get_service::<DrawableObjectCreator>()
        .unwrap();

    let drawable_object_creator_mut = &mut *drawable_object_creator.write();

    // add_skybox(drawable_object_creator_mut, renderer_client)
    //     .await
    //     .unwrap();

    {
        let mut transform = Transform::<f32, f32, f32>::default();
        transform.position.z = -5.0;

        let drawable_objects = drawable_object_creator_mut
            .get_drawable_objects_from_scene(
                "Assets/shaders/lit_normal",
                // "Assets/objects/MonkeySmooth.obj",
                "Assets/demo/wall/wallTextured.fbx",
                // "Assets/sponza/sponza.fbx",
            )
            .unwrap();

        for drawable_object in drawable_objects {
            renderer_client.add_drawable_object(drawable_object, transform);
        }
    }

    {
        let mut transform = Transform::<f32, f32, f32>::default();
        transform.position.x = -2.0;
        transform.position.z = -5.0;

        let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));
        let drawable_object = drawable_object_creator_mut
            .create_drawable_object_from_mesh("Assets/shaders/lit_normal", mesh)
            .unwrap();

        renderer_client.add_drawable_object(drawable_object, transform);
    }
}

fn add_skybox(
    drawable_object_creator: &mut DrawableObjectCreator,
    renderer_client: &dyn RendererClient,
) {
    let transform = Transform::<f32, f32, f32>::default();

    let drawable_objects = drawable_object_creator
        .get_drawable_objects_from_scene("Assets/shaders/unlit", "Assets/objects/skybox/Skybox.obj")
        .unwrap();

    if drawable_objects.len() == 6 {
        let textures = [
            "Assets/objects/skybox/skyboxRight.png",
            "Assets/objects/skybox/skyboxLeft.png",
            "Assets/objects/skybox/skyboxTop.png",
            "Assets/objects/skybox/skyboxBottom.png",
            "Assets/objects/skybox/skyboxFront.png",
            "Assets/objects/skybox/skyboxBack.png",
        ];

        for index in 0..6 {
            let mut drawable_object = drawable_objects[index].write();

            drawable_object.material = Some(GLMaterial {
                opacity: 1.0,
                albedo_color: Vec3::broadcast(1.0),
                emissive_color: Vec3::broadcast(0.0),
                shininess_color: Vec3::broadcast(0.0),
                textures: vec![GLMaterialTexture {
                    texture: drawable_object_creator
                        .get_texture(textures[index])
                        .unwrap(),
                    texture_type: MaterialTextureType::Albedo,
                    texture_map_mode: GLTextureMapMode::Clamp,
                    uv_channel_id: 0,
                    blend: 0.0,
                }],
            });

            renderer_client.add_drawable_object(drawable_objects[index].clone(), transform);
        }
    }
}
