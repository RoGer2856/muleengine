use std::{ops::DerefMut, sync::Arc, time::Duration};

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use muleengine::bytifex_utils::sync::app_loop_state::AppLoopStateWatcher;
use tokio::time::{interval, MissedTickBehavior};
use vek::{Transform, Vec3};

use crate::{
    essential_services::EssentialServices,
    physics::{character_controller::CharacterController, Rapier3dPhysicsEngineService},
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
    physics_engine: Arc<Rapier3dPhysicsEngineService>,
}

impl CharacterControllerSystem {
    fn new(essentials: &Arc<EssentialServices>) -> Self {
        Self {
            app_loop_state_watcher: essentials.app_loop_state_watcher.clone(),
            entity_container: essentials.entity_container.clone(),
            entity_group: essentials
                .entity_container
                .lock()
                .entity_group(component_type_list!(CharacterController, Transform<f32, f32, f32>)),
            physics_engine: essentials.physics_engine.clone(),
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

    async fn tick(&mut self, delta_time_in_secs: f32) {
        let mut physics_engine = self.physics_engine.write();

        for entity_id in self.entity_group.iter_entity_ids() {
            if let Some(mut handler) = self.entity_container.lock().handler_for_entity(&entity_id) {
                let mut new_position = None;

                handler.change_component::<CharacterController>(|mut character_controller| {
                    physics_engine.move_character(
                        delta_time_in_secs,
                        character_controller.deref_mut(),
                        Vec3::new(1.0, -1.0, -1.0) * 0.1,
                        // -Vec3::unit_y(),
                    );
                    new_position = Some(character_controller.get_position());
                });

                if let Some(new_position) = new_position {
                    handler.change_component::<Transform<f32, f32, f32>>(|transform| {
                        transform.position = new_position;
                    });
                }
            }
        }
    }
}
