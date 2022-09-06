mod application_context;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use game_2::{
    muleengine::{mesh::MaterialTextureType, mesh_creator},
    sdl2_opengl_engine::{
        self,
        gl_material::{GLMaterial, GLMaterialTexture},
        opengl_utils::texture_2d::GLTextureMapMode,
        GLProfile,
    },
};
use sdl2::event::{Event, WindowEvent};
use vek::{Transform, Vec3};

use crate::application_context::ApplicationContext;

pub struct MainLoop {
    desired_fps: f32,
}

impl MainLoop {
    pub fn new(desired_fps: f32) -> Self {
        Self { desired_fps }
    }

    pub fn iter(&self) -> MainLoopIterator {
        MainLoopIterator {
            desired_delta_time_in_secs: 1.0 / self.desired_fps,
            last_next_time: Instant::now(),
        }
    }
}

pub struct MainLoopIterator {
    desired_delta_time_in_secs: f32,
    last_next_time: Instant,
}

impl Iterator for MainLoopIterator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let now = Instant::now();
        let last_loop_duration = now - self.last_next_time;
        self.last_next_time = now;

        let mut last_loop_duration_in_secs = last_loop_duration.as_secs_f32();

        let count = f32::floor(last_loop_duration_in_secs / self.desired_delta_time_in_secs);
        last_loop_duration_in_secs -= count * self.desired_delta_time_in_secs;

        if last_loop_duration_in_secs < self.desired_delta_time_in_secs {
            let remaining_time_in_secs =
                self.desired_delta_time_in_secs - last_loop_duration_in_secs;
            std::thread::sleep(Duration::from_secs_f32(remaining_time_in_secs));
        }

        Some(self.desired_delta_time_in_secs)
    }
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let initial_window_dimensions = (800usize, 600usize);

    let mut engine = sdl2_opengl_engine::init(
        "game_2",
        initial_window_dimensions.0 as u32,
        initial_window_dimensions.1 as u32,
        GLProfile::Core,
        4,
        0,
    )
    .unwrap();

    let mut application_context = ApplicationContext::new(initial_window_dimensions);
    populate_with_objects(&mut application_context);

    const DESIRED_FPS: f32 = 30.0;

    let main_loop = MainLoop::new(DESIRED_FPS);
    'running: for delta_time_in_secs in main_loop.iter() {
        engine.gl_swap_window();

        // controlling the camera
        let keyboard_state = engine.keyboard_state();

        let mut moving_direction = Vec3::zero();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::W) {
            moving_direction.z -= 1.0;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::S) {
            moving_direction.z += 1.0;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::A) {
            moving_direction.x -= 1.0;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::D) {
            moving_direction.x += 1.0;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Space) {
            moving_direction.y += 1.0;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::C)
            || keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::LCtrl)
            || keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::RCtrl)
        {
            moving_direction.y -= 1.0;
        }

        application_context.set_moving_direction(moving_direction);

        application_context.tick(delta_time_in_secs);

        // handling events
        while let Some(event) = engine.poll_event() {
            log::info!("{:?}", event);
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

        // rendering
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Enable(gl::DEPTH_TEST);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        application_context.render();
    }
}

fn populate_with_objects(application_context: &mut ApplicationContext) {
    add_skybox(application_context);

    // {
    //     let mut transform = Transform::<f32, f32, f32>::default();
    //     transform.position.z = -5.0;

    //     application_context
    //         .add_scene_from_asset(
    //             "Assets/shaders/lit_normal",
    //             // "Assets/objects/MonkeySmooth.obj",
    //             "Assets/demo/wall/wallTextured.fbx",
    //             // "Assets/sponza/sponza.fbx",
    //             transform,
    //         )
    //         .unwrap();
    // }

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
        .add_scene_from_asset(
            "Assets/shaders/unlit",
            "Assets/objects/skybox/Skybox.obj",
            transform,
        )
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
        }
    }
}
