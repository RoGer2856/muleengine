use muleengine::system_container::System;
use muleengine::window_context::{Key, MouseButton};
use muleengine::{prelude::ArcRwLock, window_context::WindowContext};

use muleengine::sync::mpmc;
use vek::{Vec2, Vec3};

use super::spectator_camera_input::SpectatorCameraInput;

pub(super) struct SpectatorCameraInputSystem {
    window_context: ArcRwLock<dyn WindowContext>,
    data: SpectatorCameraInput,

    moving_event_sender: mpmc::Sender<Vec3<f32>>,
    turning_event_sender: mpmc::Sender<Vec2<f32>>,

    was_active_last_tick: bool,
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

            was_active_last_tick: false,
        }
    }

    pub fn data(&self) -> SpectatorCameraInput {
        self.data.clone()
    }
}

impl System for SpectatorCameraInputSystem {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        let mut window_context = self.window_context.write();

        let should_be_active = window_context.is_mouse_button_pressed(MouseButton::Right);
        if should_be_active {
            if self.was_active_last_tick {
                // moving the camera with the keyboard
                let mut moving_direction = Vec3::zero();
                if window_context.is_key_pressed(Key::W) {
                    moving_direction.z = -1.0;
                }
                if window_context.is_key_pressed(Key::S) {
                    moving_direction.z = 1.0;
                }

                if window_context.is_key_pressed(Key::A) {
                    moving_direction.x = -1.0;
                }
                if window_context.is_key_pressed(Key::D) {
                    moving_direction.x = 1.0;
                }

                if window_context.is_key_pressed(Key::Space) {
                    moving_direction.y = 1.0;
                }
                if window_context.is_key_pressed(Key::C)
                    || window_context.is_key_pressed(Key::CtrlLeft)
                    || window_context.is_key_pressed(Key::CtrlRight)
                {
                    moving_direction.y = -1.0;
                }

                self.moving_event_sender.send(moving_direction);

                // turning the camera with the mouse
                let mouse_camera_turn = {
                    let window_center = window_context.window_dimensions() / 2;

                    let mouse_pos = window_context.mouse_pos();
                    let mouse_motion_relative_to_center = Vec2::new(
                        mouse_pos.x as f32 - window_center.x as f32,
                        mouse_pos.y as f32 - window_center.y as f32,
                    );

                    mouse_motion_relative_to_center
                };

                self.turning_event_sender.send(mouse_camera_turn);
            }

            window_context.show_cursor(false);

            // putting the cursor back to the center of the window
            let window_center = window_context.window_dimensions() / 2;

            let mouse_pos = window_context.mouse_pos();
            if mouse_pos.x != window_center.x || mouse_pos.y != window_center.y {
                window_context.warp_mouse_normalized_screen_space(Vec2::new(0.5, 0.5));
            }
        } else {
            window_context.show_cursor(true);
        }

        self.was_active_last_tick = should_be_active;
    }
}
