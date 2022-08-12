mod application_context;

use game_2::sdl2_opengl_engine::{self, GLProfile};
use sdl2::event::Event;
use vek::Vec3;

use crate::application_context::ApplicationContext;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut engine = sdl2_opengl_engine::init("game_2", 800, 600, GLProfile::Core, 4, 0).unwrap();

    let mut application_context = ApplicationContext::new();

    'running: loop {
        // rendering
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        application_context.render();

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

        application_context.tick(1.0);

        // handling events
        while let Some(event) = engine.poll_event() {
            log::info!("{:?}", event);
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
    }
}
