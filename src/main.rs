mod application_context;

use std::time::{Duration, Instant};

use game_2::sdl2_opengl_engine::{self, GLProfile};
use sdl2::event::Event;
use vek::Vec3;

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

    let mut engine = sdl2_opengl_engine::init("game_2", 800, 600, GLProfile::Core, 4, 0).unwrap();

    let mut application_context = ApplicationContext::new();

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
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        // rendering
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        application_context.render();
    }
}
