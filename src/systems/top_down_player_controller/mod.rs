mod input;
mod player_controller;

use std::{ops::Deref, sync::Arc};

use method_taskifier::task_channel::task_channel;
use muleengine::{
    application_runner::ApplicationContext, bytifex_utils::sync::types::ArcRwLock,
    window_context::WindowContext,
};

use crate::essential_services::EssentialServices;

use self::{
    input::InputProvider,
    player_controller::{client::Client, PlayerController},
};

pub use player_controller::client::Client as TopDownPlayerControllerClient;

pub fn init(
    window_context: ArcRwLock<dyn WindowContext>,
    app_context: &mut ApplicationContext,
    essentials: Arc<EssentialServices>,
) {
    let input_provider = InputProvider::new(window_context.clone());
    let input_receiver = essentials
        .service_container
        .insert(input_provider.input_receiver())
        .new_item
        .deref()
        .clone();
    app_context
        .system_container_mut()
        .add_system(input_provider);

    let (task_sender, task_receiver) = task_channel();

    let client = Client::new(task_sender);
    essentials.service_container.insert(client.clone());

    let sendable_system_container = essentials.sendable_system_container.clone();

    tokio::spawn(async move {
        let player_controller =
            PlayerController::new(task_receiver, input_receiver, &essentials).await;
        sendable_system_container
            .write()
            .add_system(player_controller);
    });
}
