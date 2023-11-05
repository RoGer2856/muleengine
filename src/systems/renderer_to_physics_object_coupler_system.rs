use std::sync::Arc;

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use muleengine::{
    application_runner::ApplicationContext,
    bytifex_utils::result_option_inspect::ResultInspector,
    renderer::{
        renderer_system::renderer_decoupler, RendererObjectHandler, RendererTransformHandler,
    },
    system_container::System,
};
use tokio::time::Instant;
use vek::{Transform, Vec3};

use crate::physics::{Rapier3dPhysicsEngineService, RigidBodyHandler};

pub struct RendererToPhysicsObjectCouplerSystem {
    renderer_client: renderer_decoupler::Client,
    physics_engine: Arc<Rapier3dPhysicsEngineService>,

    entity_container: Arc<EntityContainer>,
    entity_group: EntityGroup,
}

impl RendererToPhysicsObjectCouplerSystem {
    pub fn new(app_context: &mut ApplicationContext) -> Self {
        let renderer_client = app_context
            .service_container_ref()
            .get_service::<renderer_decoupler::Client>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .as_ref()
            .clone();

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
            RendererObjectHandler,
            RendererTransformHandler,
            RigidBodyHandler,
        ));

        drop(entity_container_guard);

        Self {
            renderer_client,
            physics_engine,

            entity_container,
            entity_group,
        }
    }
}

impl System for RendererToPhysicsObjectCouplerSystem {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        let now = Instant::now();

        let physics_engine = self.physics_engine.read();

        for entity_id in self.entity_group.iter_entity_ids() {
            if let Some(handler) = self.entity_container.lock().handler_for_entity(&entity_id) {
                let rigid_body_handler =
                    if let Some(component) = handler.get_component_ref::<RigidBodyHandler>() {
                        component
                    } else {
                        unreachable!()
                    };

                let transform_handler = if let Some(component) =
                    handler.get_component_ref::<RendererTransformHandler>()
                {
                    component
                } else {
                    unreachable!()
                };

                if let Some((position, rotation)) =
                    physics_engine.get_interpolated_transform_of_rigidbody(&rigid_body_handler, now)
                {
                    let new_transform = Transform {
                        position,
                        orientation: rotation,
                        scale: Vec3::broadcast(1.0),
                    };
                    drop(
                        self.renderer_client
                            .update_transform(transform_handler.clone(), new_transform),
                    );
                }
            }
        }
    }
}
