#![allow(unstable_name_collisions)]

mod objects;

use std::{sync::Arc, time::Duration};

use game_2::{
    async_systems_runner::AsyncSystemsRunner,
    systems::{
        renderer_configuration::RendererConfiguration,
        spectator_camera_controller::{self, SpectatorCameraInput, SpectatorCameraInputSystem},
    },
};
use muleengine::{
    asset_container::AssetContainer,
    asset_reader::AssetReader,
    fps_counter::FpsCounter,
    image_container::ImageContainer,
    prelude::ResultInspector,
    renderer::{renderer_client::RendererClient, renderer_system::SyncRenderer},
    scene_container::SceneContainer,
    service_container::ServiceContainer,
    stopwatch::Stopwatch,
    system_container::SystemContainer,
    window_context::{Event, WindowContext},
};
use parking_lot::RwLock;
use sdl2_opengl_muleengine::{
    sdl2_gl_context::{GLProfile, Sdl2GLContext},
    systems::renderer::Renderer,
};
use tokio::task::JoinHandle;
use vek::Vec2;

use crate::objects::Objects;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let multi_threaded = true;
    let rt = if multi_threaded {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .inspect_err(|e| {
                log::error!("Could not create tokio multi thread runtime, msg = {e}")
            })
            .unwrap()
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .inspect_err(|e| {
                log::error!("Could not create tokio current thread runtime, msg = {e}")
            })
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
        .inspect_err(|e| log::error!("Could not create Sdl2GlContext, msg = {e:?}"))
        .unwrap(),
    ));

    {
        let mut sdl2_gl_context = sdl2_gl_context.write();
        sdl2_gl_context.show_cursor(false);
        sdl2_gl_context.warp_mouse_normalized_screen_space(Vec2::new(0.5, 0.5));
    }

    let mut service_container = init_basic_services();

    let (mut system_container, spectator_camera_input) = {
        let mut system_container = SystemContainer::new();

        // creating renderer system
        let renderer_impl = Renderer::new(
            initial_window_dimensions,
            sdl2_gl_context.clone(),
            service_container
                .get_service::<AssetContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
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

        // adding renderer system as the last system
        system_container.add_system(renderer_system);

        (system_container, spectator_camera_input)
    };

    let _async_systems_runner = AsyncSystemsRunner::run(run_async_systems(
        service_container.clone(),
        spectator_camera_input,
    ));

    let event_receiver = sdl2_gl_context.read().event_receiver();

    let mut fps_counter_stopwatch = Stopwatch::start_new();
    let mut fps_counter = FpsCounter::new();

    let mut delta_time_stopwatch = Stopwatch::start_new();
    let mut delta_time_in_secs = 1.0 / 60.0;
    'running: loop {
        // handling events
        sdl2_gl_context.write().flush_events();

        while let Some(event) = event_receiver.pop() {
            log::debug!("EVENT = {event:?}");

            if let Event::Closed = event {
                // todo!("if the following is called, then spectator camera controller blocks the exit process")
                // async_systems_runner.join().await;
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

        fps_counter.draw_happened();
        if fps_counter_stopwatch.elapsed() > Duration::from_millis(1000) {
            log::debug!("Average FPS = {:?}", fps_counter.get_average_fps());
            fps_counter_stopwatch.restart();
            fps_counter.restart();
        }

        tokio::task::yield_now().await;

        delta_time_in_secs = delta_time_stopwatch.restart().as_secs_f32();
    }

    // service_container is dropped manually to making sure it lives until the end of the program
    drop(service_container);
}

pub fn init_basic_services() -> ServiceContainer {
    let mut service_container = ServiceContainer::new();

    service_container.insert(AssetReader::new());
    service_container.insert(ImageContainer::new());
    service_container.insert(SceneContainer::new());
    service_container.insert(AssetContainer::new(&service_container));

    service_container
}

pub async fn run_async_systems(
    mut service_container: ServiceContainer,
    spectator_camera_input: SpectatorCameraInput,
) -> Vec<JoinHandle<()>> {
    let mut ret = Vec::new();

    let renderer_configuration = RendererConfiguration::new(service_container.clone()).await;
    let renderer_configuration = service_container
        .insert(renderer_configuration)
        .new_item
        .as_arc_ref()
        .clone();

    let renderer_client = service_container
        .get_service::<RendererClient>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .read()
        .clone();

    ret.push(tokio::spawn(spectator_camera_controller::run(
        renderer_client.clone(),
        renderer_configuration
            .read()
            .skydome_camera_transform_handler(),
        renderer_configuration
            .read()
            .main_camera_transform_handler(),
        spectator_camera_input,
    )));

    let mut objects = Objects::new(service_container.clone());
    objects.populate_with_objects().await;
    service_container.insert(objects);

    ret
}
