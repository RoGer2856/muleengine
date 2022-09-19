use std::sync::Arc;

use parking_lot::RwLock;
use vek::{num_traits::Zero, Vec2, Vec3};

use crate::{
    muleengine::{
        camera::Camera, result_option_inspect::ResultInspector, system_container::System,
    },
    sdl2_opengl_engine::{systems::renderer, Sdl2GLContext},
};

pub struct SpectatorCameraControllerSystem {
    camera: Camera,
    sdl2_gl_context: Arc<RwLock<Sdl2GLContext>>,
    renderer_command_sender: renderer::CommandSender,
}

impl SpectatorCameraControllerSystem {
    pub fn new(
        renderer_command_sender: renderer::CommandSender,
        sdl2_gl_context: Arc<RwLock<Sdl2GLContext>>,
    ) -> Self {
        Self {
            camera: Camera::new(),
            sdl2_gl_context,
            renderer_command_sender,
        }
    }
}

impl System for SpectatorCameraControllerSystem {
    fn tick(&mut self, delta_time_in_secs: f32) {
        let mut camera_turn = Vec2::<f32>::zero(); // x: vertical turn, y: horizontal turn

        let engine = self.sdl2_gl_context.read();

        let keyboard_state = engine.keyboard_state();
        let mouse_state = engine.mouse_state();

        // moving the camera with the keyboard
        let moving_direction = {
            let mut direction = Vec3::<f32>::zero();

            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::W) {
                direction.z -= 1.0;
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::S) {
                direction.z += 1.0;
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::A) {
                direction.x -= 1.0;
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::D) {
                direction.x += 1.0;
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Space) {
                direction.y += 1.0;
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::C)
                || keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::LCtrl)
                || keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::RCtrl)
            {
                direction.y -= 1.0;
            }

            if direction != Vec3::zero() {
                direction.normalize();
            }

            direction
        };

        // turning the camera with the keyboard
        let keyboard_camera_turn = {
            let mut camera_turn = Vec2::<i32>::zero();

            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
                camera_turn.x -= 1;
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
                camera_turn.x += 1;
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
                camera_turn.y -= 1;
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
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
            let window_center = Vec2::new(
                engine.window().size().0 as i32 / 2,
                engine.window().size().1 as i32 / 2,
            );

            let mouse_motion_relative_to_center = Vec2::<i32>::new(
                mouse_state.x() - window_center.x,
                mouse_state.y() - window_center.y,
            );

            mouse_motion_relative_to_center
        };

        // turning the camera
        {
            let final_camera_turn = keyboard_camera_turn.unwrap_or(mouse_camera_turn);

            if final_camera_turn.x < 0 {
                // left
                camera_turn.y = 1.0;
            } else if final_camera_turn.x > 0 {
                // right
                camera_turn.y = -1.0;
            } else {
                camera_turn.y = 0.0;
            }

            if final_camera_turn.y < 0 {
                // down
                camera_turn.x = 1.0;
            } else if final_camera_turn.y > 0 {
                // up
                camera_turn.x = -1.0;
            } else {
                camera_turn.x = 0.0;
            }
        }

        // transform the camera
        let velocity = 0.5;

        let mut axis_z = self.camera.axis_z();
        axis_z.y = 0.0;
        let axis_z = axis_z.try_normalized().unwrap_or(Vec3::zero());
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
            .renderer_command_sender
            .send(renderer::Command::SetCamera {
                camera: self.camera.clone(),
            })
            .inspect_err(|e| log::error!("Setting camera of renderer, error = {e}"));
    }
}
