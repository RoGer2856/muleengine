mod input;
mod player_controller;

use std::{ops::Deref, sync::Arc};

use method_taskifier::task_channel::task_channel;
use muleengine::{
    bytifex_utils::sync::types::ArcRwLock, system_container::SystemContainer,
    window_context::WindowContext,
};

use crate::essential_services::EssentialServices;

use self::{
    input::InputProvider,
    player_controller::{client::Client, PlayerController},
};

pub use player_controller::client::Client as TopDownPlayerControllerClient;

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

    let (task_sender, task_receiver) = task_channel();

    let client = Client::new(task_sender);
    essentials.service_container.insert(client.clone());

    tokio::spawn(async move {
        PlayerController::new(
            essentials.app_loop_state_watcher.clone(),
            task_receiver,
            input_receiver,
            essentials.entity_container.clone(),
        )
        .run()
        .await;

        client.remove_later(&essentials.closure_task_sender);
    });
}
