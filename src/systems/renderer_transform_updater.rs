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
use vek::Transform;

pub struct RendererTransformUpdaterSystem {
    renderer_client: renderer_decoupler::Client,

    entity_container: Arc<EntityContainer>,
    entity_group: EntityGroup,
}

impl RendererTransformUpdaterSystem {
    pub fn new(app_context: &mut ApplicationContext) -> Self {
        let renderer_client = app_context
            .service_container_ref()
            .get_service::<renderer_decoupler::Client>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .as_ref()
            .clone();

        let entity_container = app_context
            .service_container_ref()
            .get_service::<EntityContainer>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let mut entity_container_guard = entity_container.lock();

        let entity_group = entity_container_guard.entity_group(component_type_list!(
            RendererObjectHandler,
            RendererTransformHandler,
            Transform<f32, f32, f32>,
        ));

        drop(entity_container_guard);

        Self {
            renderer_client,

            entity_container,
            entity_group,
        }
    }
}

impl System for RendererTransformUpdaterSystem {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        for entity_id in self.entity_group.iter_entity_ids() {
            if let Some(handler) = self.entity_container.lock().handler_for_entity(&entity_id) {
                let transform = if let Some(component) =
                    handler.get_component_ref::<Transform<f32, f32, f32>>()
                {
                    *component
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

                drop(
                    self.renderer_client
                        .update_transform(transform_handler.clone(), transform),
                );
            }
        }
    }
}
