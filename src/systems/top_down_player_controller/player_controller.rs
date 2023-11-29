use std::{
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    time::Duration,
};

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use method_taskifier::{method_taskifier_impl, task_channel::TaskReceiver};
use muleengine::{
    application_runner::ClosureTaskSender, bytifex_utils::sync::app_loop_state::AppLoopStateWatcher,
};
use tokio::time::{interval, MissedTickBehavior};

use crate::{
    components::CurrentlyControlledCharacter,
    physics::character_controller::CharacterControllerHandler,
};

use super::input::{InputProvider, InputReceiver};

pub struct PlayerController {
    app_loop_state_watcher: AppLoopStateWatcher,
    should_run: Arc<AtomicBool>,
    task_receiver: TaskReceiver<client::ChanneledTask>,
    input_receiver: InputReceiver,
    entity_container: EntityContainer,
    entity_group: EntityGroup,
}

#[method_taskifier_impl(module_name = client)]
impl PlayerController {
    pub fn new(
        app_loop_state_watcher: AppLoopStateWatcher,
        task_receiver: TaskReceiver<client::ChanneledTask>,
        input_receiver: InputReceiver,
        entity_container: EntityContainer,
    ) -> Self {
        let entity_group = entity_container.lock().entity_group(component_type_list![
            CurrentlyControlledCharacter,
            CharacterControllerHandler
        ]);

        Self {
            app_loop_state_watcher,
            should_run: Arc::new(AtomicBool::new(true)),
            task_receiver,
            input_receiver,
            entity_container,
            entity_group,
        }
    }

    #[method_taskifier_client_fn]
    pub fn pause(&self) {
        drop(self.async_pause());
    }

    #[method_taskifier_worker_fn]
    fn async_pause(&self) {
        self.should_run.store(false, atomic::Ordering::SeqCst);
    }

    #[method_taskifier_client_fn]
    pub fn start(&self) {
        drop(self.async_start());
    }

    #[method_taskifier_worker_fn]
    fn async_start(&self) {
        self.should_run.store(true, atomic::Ordering::SeqCst);
    }

    pub async fn run(&mut self) {
        let interval_secs = 1.0 / 30.0;
        let mut interval = interval(Duration::from_secs_f32(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut task_receiver = self.task_receiver.clone();

        loop {
            tokio::select! {
                _ = self.app_loop_state_watcher.wait_for_quit() => {
                    break;
                }
                task = task_receiver.recv_async() => {
                    if let Ok(task) = task {
                        self.execute_channeled_task(task);
                    } else {
                        log::info!("All task sender is dropped, therefore exiting, module = {}", module_path!());
                        break;
                    }
                }
                _ = interval.tick() => {
                    self.tick(interval_secs).await;
                }
            }
        }
    }

    async fn tick(&mut self, _delta_time_in_secs: f32) {
        const VELOCITY_MAGNITUDE: f32 = 2.0;

        // moving the camera
        let velocity = self
            .input_receiver
            .movement_event_receiver
            .get_normalized_aggregated_moving_direction()
            * VELOCITY_MAGNITUDE;

        if self.should_run.load(atomic::Ordering::SeqCst) {
            let mut entity_container = self.entity_container.lock();
            for entity_id in self.entity_group.iter_entity_ids() {
                if let Some(mut entity_handler) = entity_container.handler_for_entity(&entity_id) {
                    entity_handler.change_component(
                        |character_controller: &mut CharacterControllerHandler| {
                            character_controller.set_velocity(velocity);
                        },
                    );
                }
            }
        }
    }

    #[method_taskifier_client_fn]
    pub fn remove_later(&self, closure_task_sender: &ClosureTaskSender) {
        closure_task_sender.add_task(|app_context| {
            app_context.system_container_mut().remove::<InputProvider>();
            app_context
                .service_container_ref()
                .remove::<client::Client>();
        });
    }
}
