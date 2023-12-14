use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use method_taskifier::{method_taskifier_impl, task_channel::TaskReceiver};
use muleengine::{
    application_runner::ClosureTaskSender,
    bytifex_utils::sync::broadcast,
    bytifex_utils::sync::types::ArcRwLock,
    system_container::System,
    window_context::{Event, EventReceiver, MouseButton, WindowContext},
};

use vek::Vec2;

use super::{
    camera_controller::CameraController,
    flying_movement_input::{FlyingMovementEventProvider, FlyingMovementEventReceiver},
};

#[derive(Clone)]
pub enum VelocityChangeEvent {
    Accelerate,
    Decelerate,
}

pub(super) struct InputProvider {
    enabled: Arc<AtomicBool>,
    task_receiver: TaskReceiver<client::ChanneledTask>,

    window_context: ArcRwLock<dyn WindowContext>,
    data: InputReceiver,

    event_receiver: EventReceiver,

    velocity_change_event_sender: broadcast::Sender<VelocityChangeEvent>,
    movement_event_provider: FlyingMovementEventProvider,
    turning_event_sender: broadcast::Sender<Vec2<f32>>,

    was_active_last_tick: bool,
}

#[method_taskifier_impl(module_name = client)]
impl InputProvider {
    pub fn new(
        enabled: Arc<AtomicBool>,
        window_context: ArcRwLock<dyn WindowContext>,
        task_receiver: TaskReceiver<client::ChanneledTask>,
    ) -> Self {
        let velocity_change_event_sender = broadcast::Sender::new();
        let velocity_change_event_receiver = velocity_change_event_sender.create_receiver();

        let turning_event_sender = broadcast::Sender::new();
        let turning_event_receiver = turning_event_sender.create_receiver();

        let movement_event_provider = FlyingMovementEventProvider::new();
        let movement_event_receiver = movement_event_provider.create_receiver();

        let event_receiver = window_context.read().event_receiver();

        Self {
            enabled,
            task_receiver,

            window_context,
            data: InputReceiver {
                velocity_change_event_receiver,
                movement_event_receiver,
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

    #[method_taskifier_client_fn]
    pub fn disable(&self) {
        drop(self.async_disable());
    }

    #[method_taskifier_worker_fn]
    fn async_disable(&mut self) {
        self.event_receiver.stop();
        self.enabled.store(false, atomic::Ordering::SeqCst);
    }

    #[method_taskifier_client_fn]
    pub fn enable(&self) {
        drop(self.async_enable());
    }

    #[method_taskifier_worker_fn]
    fn async_enable(&mut self) {
        self.event_receiver.resume();
        self.enabled.store(true, atomic::Ordering::SeqCst);
    }

    #[method_taskifier_client_fn]
    pub fn remove_later(&self, closure_task_sender: &ClosureTaskSender) {
        closure_task_sender.add_task(|app_context| {
            app_context.system_container_mut().remove::<InputProvider>();
            app_context
                .system_container_mut()
                .remove::<CameraController>();
            app_context
                .service_container_ref()
                .remove::<client::Client>();
        });
    }
}

impl System for InputProvider {
    fn tick(&mut self, _loop_start: &std::time::Instant, _last_loop_time_secs: f32) {
        while let Ok(task) = self.task_receiver.try_recv() {
            self.execute_channeled_task(task);
        }

        if !self.enabled.load(atomic::Ordering::SeqCst) {
            return;
        }

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
