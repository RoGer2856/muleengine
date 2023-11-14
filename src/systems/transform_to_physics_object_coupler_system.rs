use std::sync::Arc;

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use muleengine::{
    application_runner::ApplicationContext, bytifex_utils::result_option_inspect::ResultInspector,
    system_container::System,
};
use tokio::time::Instant;
use vek::Transform;

use crate::physics::{Rapier3dPhysicsEngineService, RigidBodyHandler};

pub struct TransformToPhysicsObjectCouplerSystem {
    physics_engine: Arc<Rapier3dPhysicsEngineService>,

    entity_container: Arc<EntityContainer>,
    entity_group: EntityGroup,
}

impl TransformToPhysicsObjectCouplerSystem {
    pub fn new(app_context: &mut ApplicationContext) -> Self {
        let physics_engine = app_context
            .service_container_ref()
            .get_service::<Rapier3dPhysicsEngineService>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let entity_container = app_context
            .service_container_ref()
            .get_service::<EntityContainer>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let mut entity_container_guard = entity_container.lock();

        let entity_group = entity_container_guard.entity_group(component_type_list!(
            RigidBodyHandler,
            Transform<f32, f32, f32>,
        ));

        drop(entity_container_guard);

        Self {
            physics_engine,

            entity_container,
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
