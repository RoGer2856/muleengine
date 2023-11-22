mod camera_controller;
mod flying_movement_input;
mod input;

use std::{ops::Deref, sync::Arc};

use camera_controller::*;

use method_taskifier::task_channel::task_channel;
use muleengine::{
    bytifex_utils::sync::types::ArcRwLock, system_container::SystemContainer,
    window_context::WindowContext,
};

use crate::essential_services::EssentialServices;

use self::{camera_controller::client::Client, input::InputProvider};

pub fn run(
    window_context: ArcRwLock<dyn WindowContext>,
    system_container: &mut SystemContainer,
    essentials: Arc<EssentialServices>,
) {
    let input_provider = InputProvider::new(window_context.clone());
    let input_receiver = essentials
        .service_container
        .insert(input_provider.input_receiver())
        .new_item
        .deref()
        .clone();
    system_container.add_system(input_provider);

    tokio::spawn(async move {
        let (command_sender, command_receiver) = task_channel();

        let client = Client::new(command_sender);
        essentials.service_container.insert(client.clone());

        CameraController::new(
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
            input_receiver,
        )
        .run()
        .await;

        client.remove_later(&essentials.closure_task_sender);
    });
}
