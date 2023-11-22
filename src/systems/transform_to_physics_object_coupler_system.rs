use std::sync::Arc;

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use muleengine::system_container::System;
use tokio::time::Instant;
use vek::Transform;

use crate::{
    essential_services::EssentialServices,
    physics::{Rapier3dPhysicsEngineService, RigidBodyHandler},
};

pub struct TransformToPhysicsObjectCouplerSystem {
    physics_engine: Arc<Rapier3dPhysicsEngineService>,

    entity_container: EntityContainer,
    entity_group: EntityGroup,
}

impl TransformToPhysicsObjectCouplerSystem {
    pub fn new(essentials: &Arc<EssentialServices>) -> Self {
        let mut entity_container_guard = essentials.entity_container.lock();

        let entity_group = entity_container_guard.entity_group(component_type_list!(
            RigidBodyHandler,
            Transform<f32, f32, f32>,
        ));

        drop(entity_container_guard);

        Self {
            physics_engine: essentials.physics_engine.clone(),
            entity_container: essentials.entity_container.clone(),
            entity_group,
        }
    }
}

impl System for TransformToPhysicsObjectCouplerSystem {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        let now = Instant::now();

        let physics_engine = self.physics_engine.read();

        for entity_id in self.entity_group.iter_entity_ids() {
            if let Some(mut handler) = self.entity_container.lock().handler_for_entity(&entity_id) {
                let rigid_body_handler =
                    if let Some(component) = handler.get_component_ref::<RigidBodyHandler>() {
                        component.clone()
                    } else {
                        continue;
                    };

                if let Some((position, rotation)) =
                    physics_engine.get_interpolated_transform_of_rigidbody(&rigid_body_handler, now)
                {
                    handler.change_component::<Transform<f32, f32, f32>>(|transform| {
                        transform.position = position;
                        transform.orientation = rotation;
                    });
                }
            }
        }
    }
}
