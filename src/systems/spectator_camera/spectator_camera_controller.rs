use std::time::Duration;

use muleengine::{
    app_loop_state::AppLoopStateWatcher,
    camera::Camera,
    prelude::ResultInspector,
    renderer::{renderer_client::RendererClient, TransformHandler},
};
use tokio::time::{interval, Instant, MissedTickBehavior};
use vek::{Vec2, Vec3};

use super::spectator_camera_input::SpectatorCameraInput;

pub struct SpectatorCameraControllerSystem {
    app_loop_state_watcher: AppLoopStateWatcher,
    camera: Camera,
    skydome_camera_transform_handler: TransformHandler,
    main_camera_transform_handler: TransformHandler,
    renderer_client: RendererClient,
    spectator_camera_input: SpectatorCameraInput,
}

impl SpectatorCameraControllerSystem {
    pub(super) fn new(
        app_loop_state_watcher: AppLoopStateWatcher,
        renderer_client: RendererClient,
        skydome_camera_transform_handler: TransformHandler,
        main_camera_transform_handler: TransformHandler,
        spectator_camera_input: SpectatorCameraInput,
    ) -> Self {
        Self {
            app_loop_state_watcher,
            camera: Camera::new(),
            skydome_camera_transform_handler,
            main_camera_transform_handler,
            renderer_client,
            spectator_camera_input,
        }
    }

    pub(super) async fn run(&mut self) {
        let interval_secs = 1.0 / 30.0;
        let mut interval = interval(Duration::from_secs_f32(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut delta_time_secs = 0.0;

        loop {
            let start = Instant::now();

            tokio::select! {
                _ = self.app_loop_state_watcher.wait_for_quit() => {
                    break;
                }
                _ = interval.tick() => {
                    self.tick(delta_time_secs).await;
                }
            }

            let end = Instant::now();
            delta_time_secs = (end - start).as_secs_f32();
        }
    }

    async fn tick(&mut self, delta_time_in_secs: f32) {
        // moving the camera
        let mut moving_direction = Vec3::<f32>::zero();
        while let Some(moving_input) = self.spectator_camera_input.moving_event_receiver.pop() {
            moving_direction += moving_input;
        }

        if moving_direction != Vec3::zero() {
            moving_direction.normalize();
        }

        // turning the camera
        let mut camera_turn = Vec2::<f32>::zero();
        while let Some(camera_turn_input) = self.spectator_camera_input.turning_event_receiver.pop()
        {
            if camera_turn_input.x < 0.0 {
                // left
                camera_turn.y += 1.0;
            } else if camera_turn_input.x > 0.0 {
                // right
                camera_turn.y -= 1.0;
            }

            if camera_turn_input.y < 0.0 {
                // down
                camera_turn.x += 1.0;
            } else if camera_turn_input.y > 0.0 {
                // up
                camera_turn.x -= 1.0;
            }
        }
        if !camera_turn.is_approx_zero() {
            camera_turn.normalize();
        }

        // transform the camera
        let velocity = 0.5;

        let mut axis_z = self.camera.axis_z();
        axis_z.y = 0.0;
        let axis_z = axis_z.try_normalized().unwrap_or_else(Vec3::zero);
        let corrected_moving_direction = self.camera.axis_x() * moving_direction.x
            + Vec3::unit_y() * moving_direction.y
            + axis_z * moving_direction.z;

        self.camera
            .move_by(corrected_moving_direction * velocity * delta_time_in_secs);

        let turning_velocity_rad = std::f32::consts::FRAC_PI_2;
        self.camera
            .pitch(camera_turn.x * turning_velocity_rad * delta_time_in_secs);
        self.camera
            .rotate_around_unit_y(camera_turn.y * turning_velocity_rad * delta_time_in_secs);

        let _ = self
            .renderer_client
            .update_transform(
                self.main_camera_transform_handler.clone(),
                self.camera.transform,
            )
            .await
            .inspect_err(|e| log::error!("{e:?}"));

        let mut skybox_camera_transform = self.camera.transform;
        skybox_camera_transform.position = Vec3::zero();
        let _ = self
            .renderer_client
            .update_transform(
                self.skydome_camera_transform_handler.clone(),
                skybox_camera_transform,
            )
            .await
            .inspect_err(|e| log::error!("{e:?}"));
    }
}
