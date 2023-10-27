use std::sync::Arc;

use entity_component::{component_type_list, EntityContainer};
use muleengine::{application_runner::ApplicationContext, system_container::System};

pub struct RendererToPhysicsObjectCouplerSystem {
    entity_container: Arc<EntityContainer>,
}

impl RendererToPhysicsObjectCouplerSystem {
    pub fn new(app_context: &mut ApplicationContext) -> Self {
        let entity_container = app_context
            .service_container_ref()
            .get_or_insert_service(EntityContainer::new);
        Self { entity_container }
    }
}
impl System for RendererToPhysicsObjectCouplerSystem {
    fn tick(&mut self, delta_time_in_secs: f32) {
        // todo!();
    }
}
