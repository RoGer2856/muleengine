use vek::{num_traits::Zero, Vec2, Vec3};

use muleengine::{
    camera::Camera,
    prelude::ArcRwLock,
    renderer::renderer_client::RendererClient,
    system_container::System,
    window_context::{Key, WindowContext},
};

pub struct SpectatorCameraControllerSystem {
    camera: Camera,
    window_context: ArcRwLock<dyn WindowContext>,
    renderer_client: RendererClient,
}

impl SpectatorCameraControllerSystem {
    pub fn new(
        renderer_client: RendererClient,
        window_context: ArcRwLock<dyn WindowContext>,
    ) -> Self {
        Self {
            camera: Camera::new(),
            window_context,
            renderer_client,
        }
    }
}

impl System for SpectatorCameraControllerSystem {
    fn tick(&mut self, delta_time_in_secs: f32) {
        let mut camera_turn = Vec2::<f32>::zero(); // x: vertical turn, y: horizontal turn

        let engine = self.window_context.read();

        // moving the camera with the keyboard
        let moving_direction = {
            let mut direction = Vec3::<f32>::zero();

            if engine.is_key_pressed(Key::W) {
                direction.z -= 1.0;
            }
            if engine.is_key_pressed(Key::S) {
                direction.z += 1.0;
            }
            if engine.is_key_pressed(Key::A) {
                direction.x -= 1.0;
            }
            if engine.is_key_pressed(Key::D) {
                direction.x += 1.0;
            }
            if engine.is_key_pressed(Key::Space) {
                direction.y += 1.0;
            }
            if engine.is_key_pressed(Key::C)
                || engine.is_key_pressed(Key::CtrlLeft)
                || engine.is_key_pressed(Key::CtrlRight)
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

        self.renderer_client.set_camera(self.camera.clone());
    }
}
