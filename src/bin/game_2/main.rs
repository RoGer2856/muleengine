#![allow(unstable_name_collisions)]

mod objects;

use game_2::{
    app_loop_state::{AppLoopState, AppLoopStateWatcher},
    application_runner::{self, ApplicationCallbacks, ApplicationContext},
    async_systems_runner::AsyncSystemsRunner,
    systems::{
        renderer_configuration::RendererConfiguration,
        spectator_camera_controller::{self, SpectatorCameraInput, SpectatorCameraInputSystem},
    },
};
use muleengine::{
    asset_container::AssetContainer,
    asset_reader::AssetReader,
    image_container::ImageContainer,
    prelude::{ArcRwLock, ResultInspector},
    renderer::{renderer_client::RendererClient, renderer_system::SyncRenderer},
    scene_container::SceneContainer,
    service_container::ServiceContainer,
    window_context::{Event, EventReceiver, WindowContext},
};
use sdl2_opengl_muleengine::{
    sdl2_gl_context::{GlProfile, Sdl2GlContext},
    systems::renderer::Renderer,
};
use tokio::task::JoinHandle;
use vek::Vec2;

use crate::objects::Objects;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    application_runner::run(true, |app_context| Game2::new(app_context));
}

struct Game2 {
    app_loop_state: AppLoopState,
    service_container: ServiceContainer,
    window_context: ArcRwLock<dyn WindowContext>,
    event_receiver: EventReceiver,
}

impl Game2 {
    fn add_basic_services(service_container: &mut ServiceContainer) {
        service_container.get_or_insert_service(|| AssetReader::new());
        service_container.get_or_insert_service(|| ImageContainer::new());
        service_container.get_or_insert_service(|| SceneContainer::new());
        service_container.get_or_insert_service(|| AssetContainer::new(service_container));
    }

    pub fn new(app_context: &mut ApplicationContext) -> Self {
        Self::add_basic_services(app_context.service_container());

        let window_context = {
            let initial_window_dimensions = Vec2::new(800, 600);

            let mut window_context = Sdl2GlContext::new(
                "game_2",
                initial_window_dimensions.x as u32,
                initial_window_dimensions.y as u32,
                GlProfile::Core,
                4,
                0,
            )
            .inspect_err(|e| log::error!("Could not create Sdl2GlContext, msg = {e:?}"))
            .unwrap();

            window_context.show_cursor(false);
            window_context.warp_mouse_normalized_screen_space(Vec2::new(0.5, 0.5));

            window_context
        };

        let window_context = app_context
            .system_container()
            .add_system(window_context)
            .new_item
            .as_arc_ref()
            .clone();

        let renderer_impl = Renderer::new(
            window_context.read().window_dimensions(),
            window_context.clone(),
            app_context
                .service_container()
                .get_service::<AssetContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .read()
                .clone(),
        );

        // todo!("choose between SyncRenderer and AsyncRenderer automatically");
        let renderer_system = SyncRenderer::new(renderer_impl);
        let renderer_client = renderer_system.client();

        app_context.service_container().insert(renderer_client);

        // adding Renderer as the first system
        app_context.system_container().add_system(renderer_system);

        let spectator_camera_input_system = SpectatorCameraInputSystem::new(window_context.clone());
        app_context
            .service_container()
            .insert(spectator_camera_input_system.data());
        app_context
            .system_container()
            .add_system(spectator_camera_input_system);

        let event_receiver = window_context.read().event_receiver();

        let app_loop_state = AppLoopState::new();

        app_context
            .service_container()
            .insert(app_loop_state.watcher());

        Self {
            app_loop_state,
            service_container: app_context.service_container().clone(),
            window_context,
            event_receiver,
        }
    }

    async fn run_async_systems(mut service_container: ServiceContainer) -> Vec<JoinHandle<()>> {
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

        let spectator_camera_input = service_container
            .get_service::<SpectatorCameraInput>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .read()
            .clone();

        let app_loop_state_watcher = service_container
            .get_service::<AppLoopStateWatcher>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .read()
            .clone();

        ret.push(tokio::spawn(spectator_camera_controller::run(
            app_loop_state_watcher,
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

    fn process_event(&mut self, event: Event, _app_context: &mut ApplicationContext) {
        if let Event::Closed = event {
            self.app_loop_state.stop_loop();
        }
    }
}

impl ApplicationCallbacks for Game2 {
    fn async_systems(&mut self) -> AsyncSystemsRunner {
        AsyncSystemsRunner::run(Self::run_async_systems(self.service_container.clone()))
    }

    fn should_run(&self, _app_context: &mut ApplicationContext) -> bool {
        self.app_loop_state.should_run()
    }

    fn tick(&mut self, _delta_time_in_secs: f32, app_context: &mut ApplicationContext) {
        while let Some(event) = self.event_receiver.pop() {
            log::trace!("EVENT = {event:?}");

            self.process_event(event, app_context);
        }

        // putting the cursor back to the center of the window
        let window_center = self.window_context.read().window_dimensions() / 2;

        let mouse_pos = self.window_context.read().mouse_pos();
        if mouse_pos.x != window_center.x || mouse_pos.y != window_center.y {
            self.window_context
                .write()
                .warp_mouse_normalized_screen_space(Vec2::new(0.5, 0.5));
        }
    }
}
