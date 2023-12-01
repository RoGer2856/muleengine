use std::sync::Arc;

use entity_component::{component_type_list, EntityContainer, EntityGroup};
use muleengine::{
    renderer::{renderer_system::RendererClient, RendererObjectHandler, RendererTransformHandler},
    system_container::System,
};
use vek::Transform;

use crate::essential_services::EssentialServices;

pub struct RendererTransformUpdaterSystem {
    renderer_client: RendererClient,

    entity_container: EntityContainer,
    entity_group: EntityGroup,
}

impl RendererTransformUpdaterSystem {
    pub fn new(essentials: &Arc<EssentialServices>) -> Self {
        let mut entity_container_guard = essentials.entity_container.lock();

        let entity_group = entity_container_guard.entity_group(component_type_list!(
            RendererObjectHandler,
            RendererTransformHandler,
            Transform<f32, f32, f32>,
        ));

        drop(entity_container_guard);

        Self {
            renderer_client: essentials.renderer_client.clone(),

            entity_container: essentials.entity_container.clone(),
            entity_group,
        }
    }
}

impl System for RendererTransformUpdaterSystem {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        for entity_id in self.entity_group.iter_entity_ids() {
            if let Some(entity_handler) =
                self.entity_container.lock().handler_for_entity(&entity_id)
            {
                let transform = if let Some(component) =
                    entity_handler.get_component_ref::<Transform<f32, f32, f32>>()
                {
                    *component
                } else {
                    continue;
                };

                let transform_handler = if let Some(component) =
                    entity_handler.get_component_ref::<RendererTransformHandler>()
                {
                    component.clone()
                } else {
                    continue;
                };

                drop(
                    self.renderer_client
                        .update_transform(transform_handler, transform),
                );
            }
        }
    }
}
