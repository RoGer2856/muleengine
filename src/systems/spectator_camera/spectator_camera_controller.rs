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

pub(super) struct SpectatorCameraController {
    app_loop_state_watcher: AppLoopStateWatcher,
    camera: Camera,
    skydome_camera_transform_handler: TransformHandler,
    main_camera_transform_handler: TransformHandler,
    renderer_client: RendererClient,
    spectator_camera_input: SpectatorCameraInput,
    mouse_sensitivity: f32,

    current_turn_value: Vec2<f32>,
}

impl SpectatorCameraController {
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

            mouse_sensitivity: 0.5,
            current_turn_value: Vec2::zero(),
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
        const TURNING_VELOCITY_RAD: f32 = std::f32::consts::FRAC_PI_2 * 0.1;
        let mut accumulated_camera_turn_input = Vec2::<f32>::zero();
        while let Some(camera_turn_input) = self.spectator_camera_input.turning_event_receiver.pop()
        {
            let direction = camera_turn_input
                .try_normalized()
                .unwrap_or_else(Vec2::zero);

            let magnitude = ((camera_turn_input.magnitude() - 1.0).max(0.0) * TURNING_VELOCITY_RAD)
                * self.mouse_sensitivity;

            accumulated_camera_turn_input += direction * magnitude;
        }

        accumulated_camera_turn_input = -accumulated_camera_turn_input.yx();

        self.current_turn_value = if accumulated_camera_turn_input.is_approx_zero() {
            accumulated_camera_turn_input
        } else {
            self.current_turn_value * (1.0 - self.mouse_sensitivity)
                + accumulated_camera_turn_input * self.mouse_sensitivity
        };

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

        self.camera
            .pitch(self.current_turn_value.x * delta_time_in_secs);
        self.camera
            .rotate_around_unit_y(self.current_turn_value.y * delta_time_in_secs);

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
