use std::{sync::Arc, time::Duration};

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use muleengine::bytifex_utils::sync::app_loop_state::AppLoopStateWatcher;
use tokio::time::{interval, Instant, MissedTickBehavior};
use vek::Transform;

use crate::{
    essential_services::EssentialServices,
    physics::character_controller::CharacterControllerHandler,
};

pub fn run(essentials: &Arc<EssentialServices>) {
    let mut character_controller_system = CharacterControllerSystem::new(essentials);
    tokio::spawn(async move {
        character_controller_system.run().await;
    });
}

pub struct CharacterControllerSystem {
    app_loop_state_watcher: AppLoopStateWatcher,
    entity_container: EntityContainer,
    entity_group: EntityGroup,
}

impl CharacterControllerSystem {
    fn new(essentials: &Arc<EssentialServices>) -> Self {
        Self {
            app_loop_state_watcher: essentials.app_loop_state_watcher.clone(),
            entity_container: essentials.entity_container.clone(),
            entity_group: essentials.entity_container.lock().entity_group(
                component_type_list!(CharacterControllerHandler, Transform<f32, f32, f32>),
            ),
        }
    }

    async fn run(&mut self) {
        let interval_secs = 1.0 / 30.0;
        let mut interval = interval(Duration::from_secs_f32(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = self.app_loop_state_watcher.wait_for_quit() => {
                    break;
                }
                _ = interval.tick() => {
                    self.tick(interval_secs).await;
                }
            }
        }
    }

    async fn tick(&mut self, _delta_time_in_secs: f32) {
        let now = Instant::now();

        for entity_id in self.entity_group.iter_entity_ids() {
            if let Some(mut handler) = self.entity_container.lock().handler_for_entity(&entity_id) {
                let character_controller_handler = handler
                    .get_component_ref::<CharacterControllerHandler>()
                    .as_deref()
                    .cloned();

                if let Some(character_controller_handler) = character_controller_handler {
                    let position = character_controller_handler.get_interpolated_position(now);
                    handler.change_component::<Transform<f32, f32, f32>>(|transform| {
                        transform.position = position;
                    });
                }
            }
        }
    }
}
