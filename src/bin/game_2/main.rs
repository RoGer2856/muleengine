mod application_context;

use std::sync::Arc;

use game_2::{
    main_loop::MainLoop,
    muleengine::{camera::Camera, mesh::MaterialTextureType, mesh_creator},
    sdl2_opengl_engine::{
        self,
        gl_material::{GLMaterial, GLMaterialTexture},
        opengl_utils::texture_2d::GLTextureMapMode,
        GLProfile,
    },
    systems::spectator_camera_controller,
};
use parking_lot::RwLock;
use sdl2::event::{Event, WindowEvent};
use vek::{Transform, Vec2, Vec3};

use crate::application_context::ApplicationContext;

pub struct SystemContainer {
    systems: Vec<Box<dyn FnMut(f32)>>,
}

impl SystemContainer {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn tick(&mut self, delta_time_in_secs: f32) {
        for system in self.systems.iter_mut() {
            system(delta_time_in_secs);
        }
    }

    pub fn add_system(&mut self, system_executor: impl FnMut(f32) + 'static) {
        self.systems.push(Box::new(system_executor));
    }
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let initial_window_dimensions = (800usize, 600usize);

    let engine = Arc::new(RwLock::new(
        sdl2_opengl_engine::init(
            "game_2",
            initial_window_dimensions.0 as u32,
            initial_window_dimensions.1 as u32,
            GLProfile::Core,
            4,
            0,
        )
        .unwrap(),
    ));

    let mouse_util = engine.read().mouse_util();

    mouse_util.show_cursor(false);

    mouse_util.warp_mouse_in_window(
        engine.read().window(),
        engine.read().window().size().0 as i32 / 2,
        engine.read().window().size().1 as i32 / 2,
    );

    let mut application_context = ApplicationContext::new(initial_window_dimensions);
    populate_with_objects(&mut application_context);

    let mut system_container = SystemContainer::new();

    let camera = Arc::new(RwLock::new(Camera::new()));
    system_container.add_system(spectator_camera_controller::create(
        camera.clone(),
        engine.clone(),
    ));

    const DESIRED_FPS: f32 = 30.0;

    let main_loop = MainLoop::new(DESIRED_FPS);
    'running: for delta_time_in_secs in main_loop.iter() {
        // handling events
        while let Some(event) = engine.write().poll_event() {
            log::debug!("{:?}", event);
            match event {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(width, height) => {
                        application_context.window_resized(width as usize, height as usize);
                    }
                    _ => {}
                },
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        system_container.tick(delta_time_in_secs);

        let mouse_state = engine.read().mouse_state();

        let window_center = Vec2::new(
            engine.read().window().size().0 as i32 / 2,
            engine.read().window().size().1 as i32 / 2,
        );

        // putting the cursor back to the center of the window
        if mouse_state.x() != window_center.x || mouse_state.y() != window_center.y {
            mouse_util.warp_mouse_in_window(
                engine.read().window(),
                window_center.x,
                window_center.y,
            );
        }

        // rendering
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Enable(gl::DEPTH_TEST);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        application_context.render(&*camera.read());

        engine.write().gl_swap_window();
    }
}

fn populate_with_objects(application_context: &mut ApplicationContext) {
    add_skybox(application_context);

    {
        let mut transform = Transform::<f32, f32, f32>::default();
        transform.position.z = -5.0;

        let drawable_objects = application_context
            .get_drawable_objects_from_scene(
                "Assets/shaders/lit_normal",
                // "Assets/objects/MonkeySmooth.obj",
                "Assets/demo/wall/wallTextured.fbx",
                // "Assets/sponza/sponza.fbx",
            )
            .unwrap();

        for drawable_object in drawable_objects {
            application_context.add_drawable_object(drawable_object, transform);
        }
    }

    {
        let mut transform = Transform::<f32, f32, f32>::default();
        transform.position.x = -2.0;
        transform.position.z = -5.0;

        let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));
        application_context
            .add_mesh("Assets/shaders/lit_normal", mesh, transform)
            .unwrap();
    }
}

fn add_skybox(application_context: &mut ApplicationContext) {
    let transform = Transform::<f32, f32, f32>::default();

    let drawable_objects = application_context
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
                    texture: application_context.get_texture(textures[index]).unwrap(),
                    texture_type: MaterialTextureType::Albedo,
                    texture_map_mode: GLTextureMapMode::Clamp,
                    uv_channel_id: 0,
                    blend: 0.0,
                }],
            });

            application_context.add_drawable_object(drawable_objects[index].clone(), transform);
        }
    }
}
