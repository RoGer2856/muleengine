use std::time::Duration;

use method_taskifier::task_channel::TaskReceiver;
use muleengine::{
    app_loop_state::AppLoopStateWatcher,
    camera::Camera,
    prelude::ResultInspector,
    renderer::{renderer_system::renderer_decoupler, RendererTransformHandler},
};
use tokio::time::{interval, Instant, MissedTickBehavior};
use vek::{Vec2, Vec3};

use super::{fps_camera_command::FpsCameraCommand, fps_camera_input::FpsCameraInput};

pub(super) struct FpsCameraController {
    app_loop_state_watcher: AppLoopStateWatcher,
    task_receiver: TaskReceiver<FpsCameraCommand>,
    camera: Camera,
    skydome_camera_transform_handler: RendererTransformHandler,
    main_camera_transform_handler: RendererTransformHandler,
    renderer_client: renderer_decoupler::Client,
    fps_camera_input: FpsCameraInput,
    mouse_sensitivity: f32,
    camera_vertical_angle_rad: f32,
    weighted_turn_value: Vec2<f32>,
}

impl FpsCameraController {
    pub(super) fn new(
        app_loop_state_watcher: AppLoopStateWatcher,
        task_receiver: TaskReceiver<FpsCameraCommand>,
        renderer_client: renderer_decoupler::Client,
        skydome_camera_transform_handler: RendererTransformHandler,
        main_camera_transform_handler: RendererTransformHandler,
        fps_camera_input: FpsCameraInput,
    ) -> Self {
        Self {
            app_loop_state_watcher,
            task_receiver,
            camera: Camera::new(),
            skydome_camera_transform_handler,
            main_camera_transform_handler,
            renderer_client,
            fps_camera_input,
            mouse_sensitivity: 0.5,
            camera_vertical_angle_rad: 0.0,
            weighted_turn_value: Vec2::zero(),
        }
    }

    pub(super) async fn run(&mut self) {
        let interval_secs = 1.0 / 30.0;
        let mut interval = interval(Duration::from_secs_f32(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut delta_time_secs = 0.0;

        let mut should_run = true;

        loop {
            let start = Instant::now();

            tokio::select! {
                _ = self.app_loop_state_watcher.wait_for_quit() => {
                    break;
                }
                task = self.task_receiver.recv_async() => {
                    match task {
                        Ok(FpsCameraCommand::Stop) => {
                            should_run = false;
                        }
                        Ok(FpsCameraCommand::Start) => {
                            should_run = true;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    if should_run {
                        self.tick(delta_time_secs).await;
                    }
                }
            }

            let end = Instant::now();
            delta_time_secs = (end - start).as_secs_f32();
        }
    }

    async fn tick(&mut self, delta_time_in_secs: f32) {
        // moving the camera
        let mut moving_direction = Vec3::<f32>::zero();
        while let Some(moving_input) = self.fps_camera_input.moving_event_receiver.pop() {
            moving_direction += moving_input;
        }

        if moving_direction != Vec3::zero() {
            moving_direction.normalize();
        }

        // turning the camera
        const TURNING_VELOCITY_RAD: f32 = std::f32::consts::FRAC_PI_2 * 0.1;
        let mut accumulated_camera_turn_input = Vec2::<f32>::zero();
        while let Some(camera_turn_input) = self.fps_camera_input.turning_event_receiver.pop() {
            let direction = camera_turn_input
                .try_normalized()
                .unwrap_or_else(Vec2::zero);

            let magnitude = ((camera_turn_input.magnitude() - 1.0).max(0.0) * TURNING_VELOCITY_RAD)
                * self.mouse_sensitivity;

            accumulated_camera_turn_input += direction * magnitude;
        }

        accumulated_camera_turn_input = -accumulated_camera_turn_input.yx();

        self.weighted_turn_value = if accumulated_camera_turn_input.is_approx_zero() {
            accumulated_camera_turn_input
        } else {
            self.weighted_turn_value * (1.0 - self.mouse_sensitivity)
                + accumulated_camera_turn_input * self.mouse_sensitivity
        } * delta_time_in_secs;

        const MIN_VERTICAL_ANGLE_RAD: f32 = -std::f32::consts::FRAC_PI_2;
        const MAX_VERTICAL_ANGLE_RAD: f32 = std::f32::consts::FRAC_PI_2;
        if self.camera_vertical_angle_rad + self.weighted_turn_value.x > MAX_VERTICAL_ANGLE_RAD {
            self.weighted_turn_value.x = MAX_VERTICAL_ANGLE_RAD - self.camera_vertical_angle_rad
        } else if self.camera_vertical_angle_rad + self.weighted_turn_value.x
            < MIN_VERTICAL_ANGLE_RAD
        {
            self.weighted_turn_value.x = MIN_VERTICAL_ANGLE_RAD - self.camera_vertical_angle_rad
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

        self.camera_vertical_angle_rad += self.weighted_turn_value.x;
        self.camera.pitch(self.weighted_turn_value.x);
        self.camera.rotate_around_unit_y(self.weighted_turn_value.y);

        let _ = self
            .renderer_client
            .update_transform(
                self.main_camera_transform_handler.clone(),
                *self.camera.transform_ref(),
            )
            .await
            .inspect_err(|e| log::error!("{e:?}"));

        let mut skybox_camera_transform = *self.camera.transform_ref();
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
