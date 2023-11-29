use std::{
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    time::Duration,
};

use method_taskifier::{method_taskifier_impl, task_channel::TaskReceiver};
use muleengine::{
    application_runner::ClosureTaskSender,
    bytifex_utils::{
        result_option_inspect::ResultInspector, sync::app_loop_state::AppLoopStateWatcher,
    },
    camera::Camera,
    renderer::{renderer_system::renderer_decoupler, RendererTransformHandler},
};
use tokio::time::{interval, MissedTickBehavior};
use vek::{Vec2, Vec3};

use super::input::{InputProvider, InputReceiver, VelocityChangeEvent};

pub(super) struct CameraController {
    app_loop_state_watcher: AppLoopStateWatcher,
    should_run: Arc<AtomicBool>,
    task_receiver: TaskReceiver<client::ChanneledTask>,
    camera: Camera,
    skydome_camera_transform_handler: RendererTransformHandler,
    main_camera_transform_handler: RendererTransformHandler,
    renderer_client: renderer_decoupler::Client,
    input_receiver: InputReceiver,
    mouse_sensitivity: f32,
    camera_vertical_angle_rad: f32,
    weighted_turn_value: Vec2<f32>,
    moving_velocity: f32,
}

#[method_taskifier_impl(module_name = client)]
impl CameraController {
    pub fn new(
        app_loop_state_watcher: AppLoopStateWatcher,
        task_receiver: TaskReceiver<client::ChanneledTask>,
        renderer_client: renderer_decoupler::Client,
        skydome_camera_transform_handler: RendererTransformHandler,
        main_camera_transform_handler: RendererTransformHandler,
        input_receiver: InputReceiver,
    ) -> Self {
        Self {
            app_loop_state_watcher,
            should_run: Arc::new(AtomicBool::new(true)),
            task_receiver,
            camera: Camera::new(),
            skydome_camera_transform_handler,
            main_camera_transform_handler,
            renderer_client,
            input_receiver,
            mouse_sensitivity: 0.5,
            camera_vertical_angle_rad: 0.0,
            weighted_turn_value: Vec2::zero(),
            moving_velocity: 0.5,
        }
    }

    #[method_taskifier_client_fn]
    pub fn remove_later(&self, closure_task_sender: &ClosureTaskSender) {
        closure_task_sender.add_task(|app_context| {
            app_context.system_container_mut().remove::<InputProvider>();
            app_context
                .service_container_ref()
                .remove::<client::Client>();
        });
    }

    #[method_taskifier_client_fn]
    pub fn pause(&self) {
        drop(self.async_pause());
    }

    #[method_taskifier_worker_fn]
    fn async_pause(&self) {
        self.should_run.store(false, atomic::Ordering::SeqCst);
    }

    #[method_taskifier_client_fn]
    pub fn start(&self) {
        drop(self.async_start());
    }

    #[method_taskifier_worker_fn]
    fn async_start(&self) {
        self.should_run.store(true, atomic::Ordering::SeqCst);
    }

    pub async fn run(&mut self) {
        let interval_secs = 1.0 / 30.0;
        let mut interval = interval(Duration::from_secs_f32(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut task_receiver = self.task_receiver.clone();

        // let mut delta_time_secs = 0.0;

        loop {
            // let start = Instant::now();

            tokio::select! {
                _ = self.app_loop_state_watcher.wait_for_quit() => {
                    break;
                }
                task = task_receiver.recv_async() => {
                    if let Ok(task) = task {
                        self.execute_channeled_task(task);
                    } else {
                        log::info!("All task sender is dropped, therefore exiting, module = {}", module_path!());
                        break;
                    }
                }
                _ = interval.tick() => {
                    // self.tick(delta_time_secs).await;
                    self.tick(interval_secs).await;
                }
            }

            // let end = Instant::now();
            // delta_time_secs = (end - start).as_secs_f32();
        }
    }

    async fn tick(&mut self, delta_time_in_secs: f32) {
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
        const MOVE_CAMERA_ON_Z_PLANE: bool = false;
        let mut axis_z = self.camera.axis_z();
        if MOVE_CAMERA_ON_Z_PLANE {
            axis_z.y = 0.0;
        }
        let axis_z = axis_z.try_normalized().unwrap_or_else(Vec3::zero);
        let corrected_moving_direction = self.camera.axis_x() * moving_direction.x
            + Vec3::unit_y() * moving_direction.y
            + axis_z * moving_direction.z;

        if self.should_run.load(atomic::Ordering::SeqCst) {
            self.camera
                .move_by(corrected_moving_direction * self.moving_velocity * delta_time_in_secs);

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
            drop(self.renderer_client.update_transform(
                self.skydome_camera_transform_handler.clone(),
                skybox_camera_transform,
            ));
        }
    }
}
