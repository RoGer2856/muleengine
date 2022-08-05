mod sdl2_opengl_engine;

use sdl2::event::Event;
use sdl2_opengl_engine::GLProfile;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut engine = sdl2_opengl_engine::init("game_2", 800, 600, GLProfile::Core, 3, 3).unwrap();

    'running: loop {
        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        engine.gl_swap_window();

        while let Some(event) = engine.poll_event() {
            log::info!("{:?}", event);
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
    }
}
