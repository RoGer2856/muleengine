use muleengine::system_container::System;
use muleengine::window_context::Key;
use muleengine::{prelude::ArcRwLock, window_context::WindowContext};

use muleengine::messaging::mpmc;
use vek::num_traits::Zero;
use vek::{Vec2, Vec3};

use super::spectator_camera_input::SpectatorCameraInput;

pub struct SpectatorCameraInputSystem {
    window_context: ArcRwLock<dyn WindowContext>,
    data: SpectatorCameraInput,

    moving_event_sender: mpmc::Sender<Vec3<f32>>,
    turning_event_sender: mpmc::Sender<Vec2<f32>>,
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
