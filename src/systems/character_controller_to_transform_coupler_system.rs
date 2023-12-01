use std::sync::Arc;

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use muleengine::system_container::System;
use tokio::time::Instant;
use vek::Transform;

use crate::{
    essential_services::EssentialServices,
    physics::character_controller::CharacterControllerHandler,
};

pub struct CharacterControllerToTransformCouplerSystem {
    entity_container: EntityContainer,
    entity_group: EntityGroup,
}

impl CharacterControllerToTransformCouplerSystem {
    pub fn new(essentials: &Arc<EssentialServices>) -> Self {
        Self {
            entity_container: essentials.entity_container.clone(),
            entity_group: essentials.entity_container.lock().entity_group(
                component_type_list!(CharacterControllerHandler, Transform<f32, f32, f32>),
            ),
        }
    }
}

impl System for CharacterControllerToTransformCouplerSystem {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        let now = Instant::now();

        for entity_id in self.entity_group.iter_entity_ids() {
            if let Some(mut entity_handler) =
                self.entity_container.lock().handler_for_entity(&entity_id)
            {
                let character_controller_handler = entity_handler
                    .get_component_ref::<CharacterControllerHandler>()
                    .as_deref()
                    .cloned();

                if let Some(character_controller_handler) = character_controller_handler {
                    let position = character_controller_handler.get_interpolated_position(now);
                    entity_handler.change_component(|transform: &mut Transform<f32, f32, f32>| {
                        transform.position = position;
                    });
                }
            }
        }
    }
}
