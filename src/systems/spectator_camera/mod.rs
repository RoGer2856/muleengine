mod spectator_camera_controller;
mod spectator_camera_input;
mod spectator_camera_input_provider;

pub use spectator_camera_controller::*;
pub use spectator_camera_input::*;
pub use spectator_camera_input_provider::*;

use muleengine::{
    app_loop_state::AppLoopStateWatcher,
    renderer::{renderer_client::RendererClient, TransformHandler},
};

pub async fn run(
    app_loop_state_watcher: AppLoopStateWatcher,
    renderer_client: RendererClient,
    skydome_camera_transform_handler: TransformHandler,
    main_camera_transform_handler: TransformHandler,
    spectator_camera_input: SpectatorCameraInput,
) {
    SpectatorCameraControllerSystem::new(
        app_loop_state_watcher,
        renderer_client,
        skydome_camera_transform_handler,
        main_camera_transform_handler,
        spectator_camera_input,
    )
    .run()
    .await;
}
