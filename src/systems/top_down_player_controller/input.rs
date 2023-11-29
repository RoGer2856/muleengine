use muleengine::{
    bytifex_utils::sync::types::ArcRwLock, system_container::System, window_context::WindowContext,
};

use crate::systems::general_input_providers::movement_input::{
    MovementEventProvider, MovementEventReceiver,
};

pub(super) struct InputProvider {
    window_context: ArcRwLock<dyn WindowContext>,
    data: InputReceiver,

    movement_event_provider: MovementEventProvider,
}

impl InputProvider {
    pub fn new(window_context: ArcRwLock<dyn WindowContext>) -> Self {
        let movement_event_provider = MovementEventProvider::new();
        let movement_event_consumer = movement_event_provider.create_receiver();

        Self {
            window_context,
            data: InputReceiver {
                movement_event_receiver: movement_event_consumer,
            },

            movement_event_provider,
        }
    }

    pub fn input_receiver(&self) -> InputReceiver {
        self.data.clone()
    }
}

impl System for InputProvider {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        let window_context = self.window_context.write();
        self.movement_event_provider.tick(&*window_context);
    }
}

#[derive(Clone)]
pub(super) struct InputReceiver {
    pub(super) movement_event_receiver: MovementEventReceiver,
}
