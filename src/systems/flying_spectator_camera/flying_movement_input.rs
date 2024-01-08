use muleengine::{
    bytifex_utils::sync::broadcast,
    window_context::{Key, WindowContext},
};
use vek::Vec3;

use crate::systems::general_input_providers::movement_input::{
    MovementEventProvider, MovementEventReceiver,
};

pub struct FlyingMovementEventProvider {
    elevation_movement_event_sender: broadcast::Sender<Vec3<f32>>,
    movement_event_provider: MovementEventProvider,
}

impl FlyingMovementEventProvider {
    pub fn new() -> Self {
        Self {
            elevation_movement_event_sender: broadcast::Sender::new(),
            movement_event_provider: MovementEventProvider::new(),
        }
    }

    pub fn create_receiver(&self) -> FlyingMovementEventReceiver {
        FlyingMovementEventReceiver::new(
            self.elevation_movement_event_sender.create_receiver(),
            self.movement_event_provider.create_receiver(),
        )
    }

    pub fn tick(&self, window_context: &dyn WindowContext) {
        self.movement_event_provider.tick(window_context);

        // moving the camera with the keyboard
        let mut moving_direction = Vec3::zero();

        if window_context.is_key_pressed(Key::Space) {
            moving_direction.y = 1.0;
        }
        if window_context.is_key_pressed(Key::C)
            || window_context.is_key_pressed(Key::CtrlLeft)
            || window_context.is_key_pressed(Key::CtrlRight)
        {
            moving_direction.y = -1.0;
        }

        self.elevation_movement_event_sender.send(moving_direction);
    }
}

#[derive(Clone)]
pub struct FlyingMovementEventReceiver {
    elevation_movement_event_receiver: broadcast::Receiver<Vec3<f32>>,
    movement_event_receiver: MovementEventReceiver,
}

impl FlyingMovementEventReceiver {
    pub fn new(
        elevation_movement_event_receiver: broadcast::Receiver<Vec3<f32>>,
        movement_event_receiver: MovementEventReceiver,
    ) -> Self {
        Self {
            elevation_movement_event_receiver,
            movement_event_receiver,
        }
    }

    pub fn get_normalized_aggregated_moving_direction(&self) -> Vec3<f32> {
        let mut moving_direction = self.get_aggregated_moving_direction();
        if moving_direction != Vec3::zero() {
            moving_direction.normalize();
        }
        moving_direction
    }

    pub fn get_aggregated_moving_direction(&self) -> Vec3<f32> {
        let mut moving_direction = self
            .movement_event_receiver
            .get_aggregated_moving_direction();
        while let Ok(Some(moving_input)) = self.elevation_movement_event_receiver.try_pop() {
            moving_direction += moving_input;
        }

        moving_direction
    }
}
