mod fps_camera_controller;
mod fps_camera_input;
mod fps_camera_input_provider;

use fps_camera_controller::*;
use fps_camera_input::*;
use fps_camera_input_provider::*;

use muleengine::{
    app_loop_state::{AppLoopState, AppLoopStateWatcher},
    prelude::{ArcRwLock, ResultInspector},
    renderer::renderer_client::RendererClient,
    service_container::ServiceContainer,
    system_container::SystemContainer,
    window_context::WindowContext,
};

use super::renderer_configuration::RendererConfiguration;

pub struct FpsCameraLoopState(pub AppLoopState);

pub fn run(
    window_context: ArcRwLock<dyn WindowContext>,
    service_container: ServiceContainer,
    system_container: &mut SystemContainer,
) {
    let spectator_camera_input_system = FpsCameraInputSystem::new(window_context.clone());
    service_container.insert(spectator_camera_input_system.data());
    system_container.add_system(spectator_camera_input_system);

    tokio::spawn(async move {
        let renderer_configuration = service_container
            .get_service::<RendererConfiguration>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .clone();

        let renderer_client = service_container
            .get_service::<RendererClient>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .as_ref()
            .clone();

        let spectator_camera_input = service_container
            .get_service::<FpsCameraInput>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .as_ref()
            .clone();

        let app_loop_state_watcher = service_container
            .get_service::<AppLoopStateWatcher>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .as_ref()
            .clone();

        let skydome_camera_transform_handler = renderer_configuration
            .skydome_camera_transform_handler()
            .await;
        let main_camera_transform_handler =
            renderer_configuration.main_camera_transform_handler().await;

        let spectator_camera_loop_state = FpsCameraLoopState(AppLoopState::new());
        let spectator_camera_loop_state_watcher = spectator_camera_loop_state.0.watcher();
        service_container.insert(spectator_camera_loop_state);

        FpsCameraController::new(
            app_loop_state_watcher,
            spectator_camera_loop_state_watcher,
            renderer_client,
            skydome_camera_transform_handler,
            main_camera_transform_handler,
            spectator_camera_input,
        )
        .run()
        .await;

        todo!("remove SpectatorCameraLoopState from service_container");
        todo!("remove SpectatorCameraInputProvider from system_container");
    });
}
