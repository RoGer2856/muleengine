use muleengine::{
    bytifex_utils::sync::types::{arc_rw_lock_new, ArcRwLock},
    system_container::System,
    window_context::WindowContext,
};
use vek::Vec2;

use crate::systems::general_input_providers::movement_input::{
    MovementEventProvider, MovementEventReceiver,
};

pub(super) struct InputProvider {
    window_context: ArcRwLock<dyn WindowContext>,
    input_receiver: InputReceiver,

    movement_event_provider: MovementEventProvider,
    looking_direction: ArcRwLock<Vec2<f32>>,
}

#[derive(Clone)]
pub(super) struct InputReceiver {
    pub(super) movement_event_receiver: MovementEventReceiver,
    pub(super) looking_direction: ArcRwLock<Vec2<f32>>,
}

impl InputProvider {
    pub fn new(window_context: ArcRwLock<dyn WindowContext>) -> Self {
        let movement_event_provider = MovementEventProvider::new();
        let movement_event_receiver = movement_event_provider.create_receiver();

        let looking_direction = arc_rw_lock_new(Vec2::zero());

        Self {
            window_context,
            input_receiver: InputReceiver {
                movement_event_receiver,
                looking_direction: looking_direction.clone(),
            },

            movement_event_provider,
            looking_direction,
        }
    }

    pub fn input_receiver(&self) -> InputReceiver {
        self.input_receiver.clone()
    }
}

impl System for InputProvider {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        let window_context = self.window_context.read();
        self.movement_event_provider.tick(&*window_context);

        let mouse_pos_ndc = window_context.mouse_pos_ndc();
        *self.looking_direction.write() = mouse_pos_ndc;
    }
}
