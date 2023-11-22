use muleengine::{
    bytifex_utils::sync::broadcast,
    bytifex_utils::sync::types::ArcRwLock,
    system_container::System,
    window_context::{Event, EventReceiver, MouseButton, WindowContext},
};

use vek::Vec2;

use super::flying_movement_input::{FlyingMovementEventProvider, FlyingMovementEventReceiver};

#[derive(Clone)]
pub enum VelocityChangeEvent {
    Accelerate,
    Decelerate,
}

pub(super) struct InputProvider {
    window_context: ArcRwLock<dyn WindowContext>,
    data: InputReceiver,

    event_receiver: EventReceiver,

    velocity_change_event_sender: broadcast::Sender<VelocityChangeEvent>,
    movement_event_provider: FlyingMovementEventProvider,
    turning_event_sender: broadcast::Sender<Vec2<f32>>,

    was_active_last_tick: bool,
}

impl InputProvider {
    pub fn new(window_context: ArcRwLock<dyn WindowContext>) -> Self {
        let velocity_change_event_sender = broadcast::Sender::new();
        let velocity_change_event_receiver = velocity_change_event_sender.create_receiver();

        let turning_event_sender = broadcast::Sender::new();
        let turning_event_receiver = turning_event_sender.create_receiver();

        let movement_event_provider = FlyingMovementEventProvider::new();
        let movement_event_consumer = movement_event_provider.create_receiver();

        let event_receiver = window_context.read().event_receiver();

        Self {
            window_context,
            data: InputReceiver {
                velocity_change_event_receiver,
                movement_event_receiver: movement_event_consumer,
                turning_event_receiver,
            },

            event_receiver,

            velocity_change_event_sender,
            movement_event_provider,
            turning_event_sender,

            was_active_last_tick: false,
        }
    }

    pub fn input_receiver(&self) -> InputReceiver {
        self.data.clone()
    }
}

impl System for InputProvider {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        let mut window_context = self.window_context.write();

        while let Some(event) = self.event_receiver.pop() {
            if let Event::MouseWheel { amount } = event {
                if amount > 0 {
                    self.velocity_change_event_sender
                        .send(VelocityChangeEvent::Accelerate);
                } else if amount < 0 {
                    self.velocity_change_event_sender
                        .send(VelocityChangeEvent::Decelerate);
                }
            }
        }

        let should_be_active = window_context.is_mouse_button_pressed(MouseButton::Right);
        if should_be_active {
            if self.was_active_last_tick {
                self.movement_event_provider.tick(&*window_context);

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

#[derive(Clone)]
pub(super) struct InputReceiver {
    pub(super) velocity_change_event_receiver: broadcast::Receiver<VelocityChangeEvent>,
    pub(super) movement_event_receiver: FlyingMovementEventReceiver,
    pub(super) turning_event_receiver: broadcast::Receiver<Vec2<f32>>,
}
