use entity_component::EntityContainer;
use muleengine::{
    application_runner::{Application, ApplicationContext},
    asset_container::AssetContainer,
    asset_reader::AssetReader,
    bytifex_utils::{result_option_inspect::ResultInspector, sync::app_loop_state::AppLoopState},
    image_container::ImageContainer,
    renderer::renderer_system::SyncRenderer,
    scene_container::SceneContainer,
    service_container::ServiceContainer,
    window_context::{Event, EventReceiver, WindowContext},
};
use parking_lot::RwLock;
use sdl2_opengl_muleengine::{
    sdl2_gl_context::{GlProfile, Sdl2GlContext},
    systems::renderer::Renderer,
};
use vek::Vec2;

use crate::{
    essential_services::EssentialServices,
    game_objects::populate_with_objects,
    physics,
    systems::{
        character_controller_to_transform_coupler_system::CharacterControllerToTransformCouplerSystem,
        controller_changer, flying_spectator_camera,
        physics_object_to_transform_coupler_system::PhysicsObjectToTransformCouplerSystem,
        renderer_configuration::RendererConfiguration,
        renderer_transform_updater::RendererTransformUpdaterSystem, top_down_player_controller,
    },
};

pub struct Game2 {
    app_loop_state: AppLoopState,
    event_receiver: EventReceiver,
}

impl Game2 {
    fn add_basic_services(service_container: &ServiceContainer) {
        service_container.get_or_insert_service(AssetReader::new);
        service_container.get_or_insert_service(|| RwLock::new(ImageContainer::new()));
        service_container.get_or_insert_service(|| RwLock::new(SceneContainer::new()));
        service_container.get_or_insert_service(|| AssetContainer::new(service_container));
        service_container.get_or_insert_service(EntityContainer::new);
    }

    pub fn new(app_context: &mut ApplicationContext) -> Self {
        let app_loop_state = AppLoopState::new();
        Self::add_basic_services(app_context.service_container_ref());

        let window_context = {
            let initial_window_dimensions = Vec2::new(800, 600);

            let window_context = Sdl2GlContext::new(
                "game_2",
                initial_window_dimensions.x as u32,
                initial_window_dimensions.y as u32,
                GlProfile::Core,
                4,
                0,
            )
            .inspect_err(|e| log::error!("Could not create Sdl2GlContext, msg = {e:?}"))
            .unwrap();

            window_context
        };

        let event_receiver = window_context.event_receiver();

        let window_context = app_context
            .system_container_mut()
            .add_system(window_context)
            .new_item
            .as_arc_ref()
            .clone();

        let renderer_impl = Renderer::new(
            window_context.read().window_dimensions(),
            window_context.clone(),
            app_context
                .service_container_ref()
                .get_service::<AssetContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
        );

        // todo!("choose between SyncRenderer and AsyncRenderer automatically");
        let renderer_system = SyncRenderer::new(renderer_impl);

        let renderer_client = renderer_system.client();
        app_context.service_container_ref().insert(renderer_client);

        app_context
            .service_container_ref()
            .insert(event_receiver.clone());

        app_context
            .service_container_ref()
            .insert(app_loop_state.watcher());

        app_context
            .service_container_ref()
            .insert(RendererConfiguration::new(
                app_context.service_container_ref().clone(),
            ));

        physics::init(app_context);

        let essentials = app_context
            .service_container_ref()
            .insert(EssentialServices::new(app_context))
            .new_item
            .as_arc_ref()
            .clone();

        app_context
            .system_container_mut()
            .add_system(PhysicsObjectToTransformCouplerSystem::new(&essentials));
        app_context.system_container_mut().add_system(
            CharacterControllerToTransformCouplerSystem::new(&essentials),
        );

        let renderer_transform_updater = RendererTransformUpdaterSystem::new(&essentials);
        app_context
            .system_container_mut()
            .add_system(renderer_transform_updater);

        flying_spectator_camera::init(window_context.clone(), app_context, essentials.clone());
        top_down_player_controller::init(window_context.clone(), app_context, essentials.clone());
        controller_changer::init(window_context.read().event_receiver().clone(), &essentials);

        // adding Renderer as the last system
        app_context
            .system_container_mut()
            .add_system(renderer_system);

        tokio::spawn(async move {
            populate_with_objects(&essentials).await;
        });

        Self {
            app_loop_state,
            event_receiver,
        }
    }

    fn process_event(&mut self, event: Event, _app_context: &mut ApplicationContext) {
        if let Event::Closed = event {
            self.app_loop_state.stop_loop();
        }
    }
}

impl Application for Game2 {
    fn should_run(&self, _app_context: &mut ApplicationContext) -> bool {
        self.app_loop_state.should_run()
    }

    fn tick(
        &mut self,
        loop_start: &std::time::Instant,
        last_loop_time_secs: f32,
        app_context: &mut ApplicationContext,
    ) {
        app_context
            .system_container_mut()
            .tick(loop_start, last_loop_time_secs);

        while let Some(event) = self.event_receiver.pop() {
            log::trace!("EVENT = {event:?}");

            self.process_event(event, app_context);
        }
    }
}
