use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use method_taskifier::{method_taskifier_impl, task_channel::TaskReceiver};
use muleengine::{
    application_runner::ClosureTaskSender,
    bytifex_utils::sync::types::{arc_rw_lock_new, ArcRwLock},
    system_container::System,
    window_context::WindowContext,
};
use vek::Vec2;

use crate::systems::general_input_providers::movement_input::{
    MovementEventProvider, MovementEventReceiver,
};

use super::player_controller::PlayerController;

pub(super) struct InputProvider {
    enabled: Arc<AtomicBool>,
    task_receiver: TaskReceiver<client::ChanneledTask>,
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

#[method_taskifier_impl(module_name = client)]
impl InputProvider {
    pub fn new(
        enabled: Arc<AtomicBool>,
        window_context: ArcRwLock<dyn WindowContext>,
        task_receiver: TaskReceiver<client::ChanneledTask>,
    ) -> Self {
        let movement_event_provider = MovementEventProvider::new();
        let movement_event_receiver = movement_event_provider.create_receiver();

        let looking_direction = arc_rw_lock_new(Vec2::zero());

        Self {
            enabled,
            task_receiver,
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

    #[method_taskifier_client_fn]
    pub fn enable(&self) {
        drop(self.async_enable());
    }

    #[method_taskifier_worker_fn]
    fn async_enable(&self) {
        self.enabled.store(true, atomic::Ordering::SeqCst);
    }

    #[method_taskifier_client_fn]
    pub fn disable(&self) {
        drop(self.async_disable());
    }

    #[method_taskifier_worker_fn]
    fn async_disable(&self) {
        self.enabled.store(false, atomic::Ordering::SeqCst);
    }

    #[method_taskifier_client_fn]
    pub fn remove_later(&self, closure_task_sender: &ClosureTaskSender) {
        closure_task_sender.add_task(|app_context| {
            app_context.system_container_mut().remove::<InputProvider>();
            app_context
                .system_container_mut()
                .remove::<PlayerController>();
            app_context
                .service_container_ref()
                .remove::<client::Client>();
        });
    }
}

impl System for InputProvider {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        while let Ok(task) = self.task_receiver.try_pop() {
            self.execute_channeled_task(task);
        }

        if !self.enabled.load(atomic::Ordering::SeqCst) {
            return;
        }

        let window_context = self.window_context.read();
        self.movement_event_provider.tick(&*window_context);

        let mouse_pos_ndc = window_context.mouse_pos_ndc();
        *self.looking_direction.write() = mouse_pos_ndc;
    }
}
