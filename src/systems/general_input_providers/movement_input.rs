use muleengine::{
    bytifex_utils::sync::broadcast,
    window_context::{Key, WindowContext},
};
use vek::Vec3;

pub struct MovementEventProvider(broadcast::Sender<Vec3<f32>>);

impl Default for MovementEventProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MovementEventProvider {
    pub fn new() -> Self {
        Self(broadcast::Sender::new())
    }

    pub fn create_receiver(&self) -> MovementEventReceiver {
        MovementEventReceiver::new(self.0.create_receiver())
    }

    pub fn tick(&self, window_context: &dyn WindowContext) {
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

        self.0.send(moving_direction);
    }
}

#[derive(Clone)]
pub struct MovementEventReceiver(broadcast::Receiver<Vec3<f32>>);

impl MovementEventReceiver {
    pub fn new(receiver: broadcast::Receiver<Vec3<f32>>) -> Self {
        Self(receiver)
    }

    pub fn get_normalized_aggregated_moving_direction(&self) -> Vec3<f32> {
        let mut moving_direction = self.get_aggregated_moving_direction();
        if moving_direction != Vec3::zero() {
            moving_direction.normalize();
        }
        moving_direction
    }

    pub fn get_aggregated_moving_direction(&self) -> Vec3<f32> {
        let mut moving_direction = Vec3::<f32>::zero();
        while let Some(moving_input) = self.0.pop() {
            moving_direction += moving_input;
        }

        moving_direction
    }
}
