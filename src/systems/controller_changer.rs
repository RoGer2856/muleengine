use std::{sync::Arc, time::Duration};

use muleengine::{
    bytifex_utils::result_option_inspect::ResultInspector,
    window_context::{Event, EventReceiver, Key},
};
use tokio::time::{interval, MissedTickBehavior};

use crate::essential_services::EssentialServices;

use super::{
    flying_spectator_camera::FlyingSpectatorCameraControllerClient,
    top_down_player_controller::TopDownPlayerControllerClient,
};

pub fn init(event_receiver: EventReceiver, essentials: &Arc<EssentialServices>) {
    let app_loop_state_watcher = essentials.app_loop_state_watcher.clone();

    let flying_spectator_camera_client = essentials
        .service_container
        .get_service::<FlyingSpectatorCameraControllerClient>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .as_ref()
        .clone();

    let top_town_player_controller_client = essentials
        .service_container
        .get_service::<TopDownPlayerControllerClient>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .as_ref()
        .clone();

    tokio::spawn(async move {
        let mut spectator_mode = false;

        let _ = flying_spectator_camera_client.async_disable().await;
        let _ = top_town_player_controller_client.async_enable().await;

        let interval_secs = 1.0 / 30.0;
        let mut interval = interval(Duration::from_secs_f32(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = app_loop_state_watcher.wait_for_quit() => {
                    break;
                }
                _ = interval.tick() => {
                    while let Some(event) = event_receiver.pop() {
                        if let Event::KeyDown { key } = event {
                            if key == Key::F1 {
                                spectator_mode = !spectator_mode;
                                if spectator_mode {
                                    let _ = top_town_player_controller_client.async_disable().await;
                                    let _ = flying_spectator_camera_client.async_enable().await;
                                } else {
                                    let _ = flying_spectator_camera_client.async_disable().await;
                                    let _ = top_town_player_controller_client.async_enable().await;
                                }
                            }
                        }
                    }
                }
            }
        }
        event_receiver.pop()
    });
}