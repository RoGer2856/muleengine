use std::time::{Duration, Instant};

use tokio::time::{interval, MissedTickBehavior};
use vek::{num_traits::Zero, Vec2, Vec3};

use muleengine::{
    app_loop_state::AppLoopStateWatcher,
    camera::Camera,
    messaging::mpmc,
    prelude::{ArcRwLock, ResultInspector},
    renderer::{renderer_client::RendererClient, TransformHandler},
    system_container::System,
    window_context::{Key, WindowContext},
};

pub struct SpectatorCameraInputSystem {
    window_context: ArcRwLock<dyn WindowContext>,
    data: SpectatorCameraInput,

    moving_event_sender: mpmc::Sender<Vec3<f32>>,
    turning_event_sender: mpmc::Sender<Vec2<f32>>,
}

#[derive(Clone)]
pub struct SpectatorCameraInput {
    moving_event_receiver: mpmc::Receiver<Vec3<f32>>,
    turning_event_receiver: mpmc::Receiver<Vec2<f32>>,
}

impl SpectatorCameraInputSystem {
    pub fn new(window_context: ArcRwLock<dyn WindowContext>) -> Self {
        let turning_event_sender = mpmc::Sender::new();
        let turning_event_receiver = turning_event_sender.create_receiver();

        let moving_event_sender = mpmc::Sender::new();
        let moving_event_receiver = moving_event_sender.create_receiver();

        Self {
            window_context,
            data: SpectatorCameraInput {
                moving_event_receiver,
                turning_event_receiver,
            },

            moving_event_sender,
            turning_event_sender,
        }
    }

    pub fn data(&self) -> SpectatorCameraInput {
        self.data.clone()
    }
}

impl System for SpectatorCameraInputSystem {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        let engine = self.window_context.read();

        // moving the camera with the keyboard
        let mut moving_direction = Vec3::zero();
        if engine.is_key_pressed(Key::W) {
            moving_direction.z = -1.0;
        }
        if engine.is_key_pressed(Key::S) {
            moving_direction.z = 1.0;
        }

        if engine.is_key_pressed(Key::A) {
            moving_direction.x = -1.0;
        }
        if engine.is_key_pressed(Key::D) {
            moving_direction.x = 1.0;
        }

        if engine.is_key_pressed(Key::Space) {
            moving_direction.y = 1.0;
        }
        if engine.is_key_pressed(Key::C)
            || engine.is_key_pressed(Key::CtrlLeft)
            || engine.is_key_pressed(Key::CtrlRight)
        {
            moving_direction.y = -1.0;
        }

        self.moving_event_sender.send(moving_direction);

        // turning the camera with the keyboard
        let keyboard_camera_turn = {
            let mut camera_turn = Vec2::<i32>::zero();

            if engine.is_key_pressed(Key::Left) {
                camera_turn.x -= 1;
            }
            if engine.is_key_pressed(Key::Right) {
                camera_turn.x += 1;
            }
            if engine.is_key_pressed(Key::Up) {
                camera_turn.y -= 1;
            }
            if engine.is_key_pressed(Key::Down) {
                camera_turn.y += 1;
            }

            if camera_turn.is_zero() {
                None
            } else {
                Some(camera_turn)
            }
        };

        // turning the camera with the mouse
        let mouse_camera_turn = {
            let window_center = engine.window_dimensions() / 2;

            let mouse_pos = engine.mouse_pos();
            let mouse_motion_relative_to_center = Vec2::<i32>::new(
                mouse_pos.x as i32 - window_center.x as i32,
                mouse_pos.y as i32 - window_center.y as i32,
            );

            mouse_motion_relative_to_center
        };

        let final_camera_turn = keyboard_camera_turn.unwrap_or(mouse_camera_turn);

        self.turning_event_sender.send(Vec2::new(
            final_camera_turn.x as f32,
            final_camera_turn.y as f32,
        ));
    }
}

pub struct SpectatorCameraControllerSystem {
    app_loop_state_watcher: AppLoopStateWatcher,
    camera: Camera,
    skydome_camera_transform_handler: TransformHandler,
    main_camera_transform_handler: TransformHandler,
    renderer_client: RendererClient,
    spectator_camera_input: SpectatorCameraInput,
}

impl SpectatorCameraControllerSystem {
    pub fn new(
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

    async fn run(&mut self) {
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
