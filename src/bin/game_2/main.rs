#![allow(unstable_name_collisions)]

use std::sync::Arc;

use game_2::{
    main_loop::MainLoop,
    systems::{game_manager::GameManager, spectator_camera_controller::SpectatorCameraInputSystem},
};
use muleengine::{
    asset_container::AssetContainer,
    asset_reader::AssetReader,
    image_container::ImageContainer,
    renderer::renderer_system::SyncRenderer,
    scene_container::SceneContainer,
    service_container::ServiceContainer,
    system_container::SystemContainer,
    window_context::{Event, WindowContext},
};
use parking_lot::RwLock;
use sdl2_opengl_muleengine::{
    sdl2_gl_context::{GLProfile, Sdl2GLContext},
    systems::renderer::Renderer,
};
use vek::Vec2;

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

    let mut service_container = init_services();

    let mut system_container = {
        let mut system_container = SystemContainer::new();

        // creating renderer system
        let renderer_impl = Renderer::new(
            initial_window_dimensions,
            sdl2_gl_context.clone(),
            service_container
                .get_service::<AssetContainer>()
                .unwrap()
                .read()
                .clone(),
        );

        let renderer_system = SyncRenderer::new(renderer_impl);
        let renderer_client = renderer_system.client();

        service_container.insert(renderer_client);

        let spectator_camera_input_system =
            SpectatorCameraInputSystem::new(sdl2_gl_context.clone());
        let spectator_camera_input = spectator_camera_input_system.data().clone();
        system_container.add_system(spectator_camera_input_system);

        system_container.add_system(GameManager::new(service_container, spectator_camera_input));

        // adding renderer system as the last system
        system_container.add_system(renderer_system);

        system_container
    };

    let event_receiver = sdl2_gl_context.read().event_receiver();

    const DESIRED_FPS: f32 = 30.0;
    let main_loop = MainLoop::new(DESIRED_FPS);
    'running: for delta_time_in_secs in main_loop.iter() {
        // handling events
        sdl2_gl_context.write().flush_events();

        while let Some(event) = event_receiver.pop() {
            log::debug!("EVENT = {event:?}");

            if let Event::Closed = event {
                break 'running;
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
