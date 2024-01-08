use std::sync::Arc;

use entity_component::{component_type_list, EntityContainer, EntityGroupEvent, EntityId};
use muleengine::renderer::{
    renderer_system::RendererClient, RendererObjectHandler, RendererTransformHandler,
};
use vek::Transform;

use crate::essential_services::EssentialServices;

pub fn run(essentials: &Arc<EssentialServices>) {
    let mut entity_container = essentials.entity_container.clone();
    let renderer_client = essentials.renderer_client.clone();

    tokio::spawn(async move {
        let entity_group = entity_container.lock().entity_group(component_type_list!(
            RendererObjectHandler,
            RendererTransformHandler,
            Transform<f32, f32, f32>,
        ));
        let event_receiver = entity_group.event_receiver(true, &mut entity_container.lock());

        while let Ok(event) = event_receiver.pop().await {
            if let EntityGroupEvent::EntityAdded { entity_id } = event {
                update_renderer_transform_of_entity(
                    entity_id,
                    &renderer_client,
                    &mut entity_container,
                );
            } else if let EntityGroupEvent::ComponentChanged { entity_id, .. } = event {
                update_renderer_transform_of_entity(
                    entity_id,
                    &renderer_client,
                    &mut entity_container,
                );
            }
        }
    });
}

fn update_renderer_transform_of_entity(
    entity_id: EntityId,
    renderer_client: &RendererClient,
    entity_container: &mut EntityContainer,
) {
    if let Some(entity_handler) = entity_container.lock().handler_for_entity(&entity_id) {
        let transform = if let Some(component) =
            entity_handler.get_component_ref::<Transform<f32, f32, f32>>()
        {
            *component
        } else {
            return;
        };

        let transform_handler = if let Some(component) =
            entity_handler.get_component_ref::<RendererTransformHandler>()
        {
            component.clone()
        } else {
            return;
        };

        drop(renderer_client.update_transform(transform_handler, transform));
    }
}
