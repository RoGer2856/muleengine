mod camera_controller;
mod flying_movement_input;
mod input;

use std::{
    ops::Deref,
    sync::{atomic::AtomicBool, Arc},
};

use camera_controller::*;

use method_taskifier::task_channel::task_channel;
use muleengine::{
    application_runner::ApplicationContext, bytifex_utils::sync::types::ArcRwLock,
    window_context::WindowContext,
};

use crate::essential_services::EssentialServices;

use self::input::InputProvider;
pub use self::input::InputProviderClient as FlyingSpectatorCameraControllerClient;

pub fn init(
    window_context: ArcRwLock<dyn WindowContext>,
    app_context: &mut ApplicationContext,
    essentials: Arc<EssentialServices>,
) {
    let enabled = Arc::new(AtomicBool::new(true));
    let (task_sender, task_receiver) = task_channel();

    let client = FlyingSpectatorCameraControllerClient::new(task_sender);
    essentials.service_container.insert(client.clone());

    let input_provider = InputProvider::new(enabled.clone(), window_context.clone(), task_receiver);
    let input_receiver = essentials
        .service_container
        .insert(input_provider.input_receiver())
        .new_item
        .deref()
        .clone();
    app_context
        .system_container_mut()
        .add_system(input_provider);

    let system_container_client = essentials.system_container_client.clone();

    tokio::spawn(async move {
        let camera_controller = CameraController::new(enabled, input_receiver, &essentials).await;
        system_container_client.execute_closure_async(|system_container| {
            system_container.add_system(camera_controller);
        });
    });
}
