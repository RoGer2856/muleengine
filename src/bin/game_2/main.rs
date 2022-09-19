#![allow(unstable_name_collisions)]

use std::sync::Arc;

use game_2::{
    main_loop::MainLoop,
    muleengine::{
        assets_reader::AssetsReader, image_container::ImageContainer, mesh::MaterialTextureType,
        mesh_creator, result_option_inspect::ResultInspector, scene_container::SceneContainer,
        service_container::ServiceContainer, system_container::SystemContainer,
    },
    sdl2_opengl_engine::{
        drawable_object_creator::DrawableObjectCreator,
        gl_material::{GLMaterial, GLMaterialTexture},
        opengl_utils::texture_2d::GLTextureMapMode,
        systems::renderer,
        GLProfile, Sdl2GLContext,
    },
    systems::spectator_camera_controller::SpectatorCameraControllerSystem,
};
use parking_lot::RwLock;
use sdl2::event::{Event, WindowEvent};
use vek::{Transform, Vec2, Vec3};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

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

    let mouse_util = sdl2_gl_context.read().mouse_util();

    mouse_util.show_cursor(false);

    mouse_util.warp_mouse_in_window(
        sdl2_gl_context.read().window(),
        sdl2_gl_context.read().window().size().0 as i32 / 2,
        sdl2_gl_context.read().window().size().1 as i32 / 2,
    );

    let service_container = {
        let mut service_container = ServiceContainer::new();

        service_container.insert(AssetsReader::new());
        service_container.insert(ImageContainer::new());
        service_container.insert(SceneContainer::new());
        service_container.insert(DrawableObjectCreator::new(&service_container));

        service_container
    };

    let (mut system_container, renderer_command_sender) = {
        let mut system_container = SystemContainer::new();

        // creating renderer system
        let renderer_system =
            renderer::System::new(initial_window_dimensions, sdl2_gl_context.clone());
        let renderer_command_sender = renderer_system.get_sender();

        system_container.add_system(SpectatorCameraControllerSystem::new(
            renderer_command_sender.clone(),
            sdl2_gl_context.clone(),
        ));

        // adding renderer system as the last system
        system_container.add_system(renderer_system);

        (system_container, renderer_command_sender)
    };

    populate_with_objects(&service_container, &renderer_command_sender);

    const DESIRED_FPS: f32 = 30.0;

    let main_loop = MainLoop::new(DESIRED_FPS);
    'running: for delta_time_in_secs in main_loop.iter() {
        // handling events
        while let Some(event) = sdl2_gl_context.write().poll_event() {
            log::debug!("{:?}", event);

            match event {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(width, height) => {
                        let _ = renderer_command_sender
                            .send(renderer::Command::SetWindowDimensions {
                                dimensions: Vec2::new(width as usize, height as usize),
                            })
                            .inspect_err(|e| {
                                log::error!("Setting window dimensions of renderer, error = {e}")
                            });
                    }
                    _ => {}
                },
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        system_container.tick(delta_time_in_secs);

        // putting the cursor back to the center of the window
        let window_center = Vec2::new(
            sdl2_gl_context.read().window().size().0 as i32 / 2,
            sdl2_gl_context.read().window().size().1 as i32 / 2,
        );

        let mouse_state = sdl2_gl_context.read().mouse_state();
        if mouse_state.x() != window_center.x || mouse_state.y() != window_center.y {
            mouse_util.warp_mouse_in_window(
                sdl2_gl_context.read().window(),
                window_center.x,
                window_center.y,
            );
        }
    }
}

fn populate_with_objects(
    service_container: &ServiceContainer,
    renderer_command_sender: &renderer::CommandSender,
) {
    let drawable_object_creator = service_container
        .get_service::<DrawableObjectCreator>()
        .unwrap();

    let drawable_object_creator_mut = &mut *drawable_object_creator.write();

    add_skybox(drawable_object_creator_mut, renderer_command_sender);

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
            let _ = renderer_command_sender
                .send(renderer::Command::AddDrawableObject {
                    drawable_object,
                    transform,
                })
                .inspect_err(|e| log::error!("Adding drawable object to renderer, error = {e}"));
        }
    }

    {
        let mut transform = Transform::<f32, f32, f32>::default();
        transform.position.x = -2.0;
        transform.position.z = -5.0;

        let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));
        let drawable_object = drawable_object_creator_mut
            .get_drawable_object_from_mesh("Assets/shaders/lit_normal", mesh)
            .unwrap();

        let _ = renderer_command_sender
            .send(renderer::Command::AddDrawableObject {
                drawable_object,
                transform,
            })
            .inspect_err(|e| log::error!("Adding drawable object to renderer, error = {e}"));
    }
}

fn add_skybox(
    drawable_object_creator: &mut DrawableObjectCreator,
    renderer_command_sender: &renderer::CommandSender,
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

            let _ = renderer_command_sender
                .send(renderer::Command::AddDrawableObject {
                    drawable_object: drawable_objects[index].clone(),
                    transform,
                })
                .inspect_err(|e| log::error!("Adding drawable object to renderer, error = {e}"));
        }
    }
}
