mod fps_camera_controller;
mod fps_camera_input;
mod fps_camera_input_provider;

use fps_camera_controller::*;
use fps_camera_input::*;
use fps_camera_input_provider::*;

use method_taskifier::task_channel::task_channel;
use muleengine::{
    application_runner::ApplicationContext,
    bytifex_utils::{
        result_option_inspect::ResultInspector,
        sync::{app_loop_state::AppLoopStateWatcher, types::ArcRwLock},
    },
    renderer::renderer_system::renderer_decoupler,
    window_context::WindowContext,
};

use self::fps_camera_controller::client::Client;

use super::renderer_configuration::RendererConfiguration;

pub fn run(window_context: ArcRwLock<dyn WindowContext>, app_context: &mut ApplicationContext) {
    let service_container = app_context.service_container_ref().clone();
    let closure_task_sender = app_context.closure_tasks_ref().clone();
    let system_container = app_context.system_container_mut();

    let fps_camera_input_system = FpsCameraInputSystem::new(window_context.clone());
    service_container.insert(fps_camera_input_system.data());
    system_container.add_system(fps_camera_input_system);

    tokio::spawn(async move {
        let renderer_configuration = service_container
            .get_service::<RendererConfiguration>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .clone();

        let renderer_client = service_container
            .get_service::<renderer_decoupler::Client>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .as_ref()
            .clone();

        let fps_camera_input = service_container
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

        let (command_sender, command_receiver) = task_channel();

        let fps_camera_client = Client::new(command_sender);
        service_container.insert(fps_camera_client.clone());

        FpsCameraController::new(
            app_loop_state_watcher,
            command_receiver,
            renderer_client,
            skydome_camera_transform_handler,
            main_camera_transform_handler,
            fps_camera_input,
        )
        .run()
        .await;

        fps_camera_client.remove_later(closure_task_sender);
    });
}
