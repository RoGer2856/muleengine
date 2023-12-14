use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use muleengine::{
    camera::Camera,
    renderer::{renderer_system::RendererClient, RendererTransformHandler},
    system_container::System,
};
use vek::{Vec2, Vec3};

use crate::essential_services::EssentialServices;

use super::input::{InputReceiver, VelocityChangeEvent};

pub(super) struct CameraController {
    enabled: Arc<AtomicBool>,
    camera: Camera,
    skydome_camera_transform_handler: RendererTransformHandler,
    main_camera_transform_handler: RendererTransformHandler,
    renderer_client: RendererClient,
    input_receiver: InputReceiver,
    mouse_sensitivity: f32,
    camera_vertical_angle_rad: f32,
    weighted_turn_value: Vec2<f32>,
    moving_velocity: f32,
}

impl CameraController {
    pub async fn new(
        enabled: Arc<AtomicBool>,
        input_receiver: InputReceiver,
        essentials: &Arc<EssentialServices>,
    ) -> Self {
        Self {
            enabled,
            camera: Camera::new(),
            skydome_camera_transform_handler: essentials
                .renderer_configuration
                .skydome_camera_transform_handler()
                .await,
            main_camera_transform_handler: essentials
                .renderer_configuration
                .main_camera_transform_handler()
                .await,
            renderer_client: essentials.renderer_client.clone(),
            input_receiver,
            mouse_sensitivity: 0.5,
            camera_vertical_angle_rad: 0.0,
            weighted_turn_value: Vec2::zero(),
            moving_velocity: 0.5,
        }
    }
}

impl System for CameraController {
    fn tick(&mut self, _loop_start: &std::time::Instant, last_loop_time_secs: f32) {
        // accelerating or decelerating the camera
        while let Some(event) = self.input_receiver.velocity_change_event_receiver.pop() {
            match event {
                VelocityChangeEvent::Accelerate => {
                    self.moving_velocity *= 1.5;
                }
                VelocityChangeEvent::Decelerate => {
                    self.moving_velocity /= 1.5;
                }
            }
        }

        // moving the camera
        let moving_direction = self
            .input_receiver
            .movement_event_receiver
            .get_normalized_aggregated_moving_direction();

        // turning the camera
        const TURNING_VELOCITY_RAD: f32 = std::f32::consts::FRAC_PI_2 * 0.1;
        let mut accumulated_camera_turn_input = Vec2::<f32>::zero();
        while let Some(camera_turn_input) = self.input_receiver.turning_event_receiver.pop() {
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
        } * last_loop_time_secs;

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
        const MOVE_CAMERA_ON_Z_PLANE: bool = false;
        let mut axis_z = self.camera.axis_z();
        if MOVE_CAMERA_ON_Z_PLANE {
            axis_z.y = 0.0;
        }
        let axis_z = axis_z.try_normalized().unwrap_or_else(Vec3::zero);
        let corrected_moving_direction = self.camera.axis_x() * moving_direction.x
            + Vec3::unit_y() * moving_direction.y
            + axis_z * moving_direction.z;

        if self.enabled.load(atomic::Ordering::SeqCst) {
            self.camera
                .move_by(corrected_moving_direction * self.moving_velocity * last_loop_time_secs);

            self.camera_vertical_angle_rad += self.weighted_turn_value.x;
            self.camera.pitch(self.weighted_turn_value.x);
            self.camera.rotate_around_unit_y(self.weighted_turn_value.y);

            drop(self.renderer_client.update_transform(
                self.main_camera_transform_handler.clone(),
                *self.camera.transform_ref(),
            ));

            let mut skybox_camera_transform = *self.camera.transform_ref();
            skybox_camera_transform.position = Vec3::zero();
            drop(self.renderer_client.update_transform(
                self.skydome_camera_transform_handler.clone(),
                skybox_camera_transform,
            ));
        }
    }
}
