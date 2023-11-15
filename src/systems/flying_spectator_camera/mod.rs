mod flying_spectator_camera_controller;
mod flying_spectator_camera_input;
mod flying_spectator_camera_input_provider;

use std::{ops::Deref, sync::Arc};

use flying_spectator_camera_controller::*;
use flying_spectator_camera_input_provider::*;

use method_taskifier::task_channel::task_channel;
use muleengine::{
    bytifex_utils::sync::types::ArcRwLock, system_container::SystemContainer,
    window_context::WindowContext,
};

use crate::essential_services::EssentialServices;

use self::flying_spectator_camera_controller::client::Client;

pub fn run(
    window_context: ArcRwLock<dyn WindowContext>,
    system_container: &mut SystemContainer,
    essentials: Arc<EssentialServices>,
) {
    let flying_spectator_camera_input_system =
        FlyingSpectatorCameraInputSystem::new(window_context.clone());
    let flying_spectator_camera_input = essentials
        .service_container
        .insert(flying_spectator_camera_input_system.data())
        .new_item;
    system_container.add_system(flying_spectator_camera_input_system);

    tokio::spawn(async move {
        let (command_sender, command_receiver) = task_channel();

        let flying_spectator_camera_client = Client::new(command_sender);
        essentials
            .service_container
            .insert(flying_spectator_camera_client.clone());

        FlyingSpectatorCameraController::new(
            essentials.app_loop_state_watcher.clone(),
            command_receiver,
            essentials.renderer_client.clone(),
            essentials
                .renderer_configuration
                .skydome_camera_transform_handler()
                .await,
            essentials
                .renderer_configuration
                .main_camera_transform_handler()
                .await,
            flying_spectator_camera_input.deref().clone(),
        )
        .run()
        .await;

        flying_spectator_camera_client.remove_later(&essentials.closure_task_sender);
    });
}
