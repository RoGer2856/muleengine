use entity_component::{
    component_type_list, EntityContainer, EntityContainerGuard, EntityGroupEvent, EntityId,
};
use muleengine::{
    bytifex_utils::sync::types::ArcRwLock,
    window_context::{Event, WindowContext},
};
use vek::{Transform, Vec2};

#[derive(Clone)]
pub enum UiEntityPosition {
    TopLeftWindow { offset: Vec2<f32> },
    TopMiddleWindow { offset: Vec2<f32> },
    TopRightWindow { offset: Vec2<f32> },

    MiddleLeftWindow { offset: Vec2<f32> },
    MiddleMiddleWindow { offset: Vec2<f32> },
    MiddleRightWindow { offset: Vec2<f32> },

    BottomLeftWindow { offset: Vec2<f32> },
    BottomMiddleWindow { offset: Vec2<f32> },
    BottomRightWindow { offset: Vec2<f32> },
}

pub fn run(entity_container: EntityContainer, window_context: ArcRwLock<dyn WindowContext>) {
    let window_context_event_receiver = window_context.read().event_receiver();
    let mut window_dimensions = window_context.read().window_dimensions();

    tokio::spawn(async move {
        let entity_group = entity_container
            .lock()
            .entity_group(component_type_list![Transform<f32, f32, f32>, UiEntityPosition]);
        let entity_group_event_receiver =
            entity_group.event_receiver(true, &mut entity_container.lock());

        loop {
            tokio::select! {
                event = window_context_event_receiver.pop() => {
                    if event.is_err() {
                        break;
                    }

                    let mut entity_container_guard = entity_container.lock();

                    if let Ok(Event::Resized { width, height }) = event {
                        window_dimensions = Vec2::new(width, height);

                        for entity_id in entity_group.iter_entity_ids() {
                            set_transform_of_entity(entity_id, &mut entity_container_guard, window_dimensions);
                        }
                    }
                },
                event = entity_group_event_receiver.pop() => {
                    if event.is_err() {
                        break;
                    }

                    let mut entity_container_guard = entity_container.lock();

                    if let Ok(EntityGroupEvent::EntityAdded { entity_id }) = event {
                        set_transform_of_entity(
                            entity_id,
                            &mut entity_container_guard,
                            window_dimensions,
                        );
                    } else if let Ok(EntityGroupEvent::ComponentChanged {
                        entity_id,
                        component_id,
                    }) = event
                    {
                        if component_id.is_component_type_of::<UiEntityPosition>() {
                            set_transform_of_entity(
                                entity_id,
                                &mut entity_container_guard,
                                window_dimensions,
                            );
                        }
                    }
                }
            }
        }
    });
}

fn set_transform_of_entity(
    entity_id: EntityId,
    entity_container_guard: &mut EntityContainerGuard,
    window_dimensions: Vec2<usize>,
) {
    if let Some(mut entity_handler) = entity_container_guard.handler_for_entity(&entity_id) {
        let position =
            if let Some(component) = entity_handler.get_component_ref::<UiEntityPosition>() {
                component.clone()
            } else {
                return;
            };

        entity_handler.change_component(|transform: &mut Transform<f32, f32, f32>| {
            let pos = compute_position_of_entity(&position, window_dimensions);
            transform.position.x = pos.x;
            transform.position.y = pos.y;
        });
    }
}

fn compute_position_of_entity(
    position: &UiEntityPosition,
    window_dimensions: Vec2<usize>,
) -> Vec2<f32> {
    let ratio = window_dimensions.y as f32 / window_dimensions.x as f32;
    match position {
        UiEntityPosition::TopLeftWindow { offset } => Vec2::new(-1.0, ratio) + offset,
        UiEntityPosition::TopMiddleWindow { offset } => Vec2::new(0.0, ratio) + offset,
        UiEntityPosition::TopRightWindow { offset } => Vec2::new(1.0, ratio) + offset,

        UiEntityPosition::MiddleLeftWindow { offset } => Vec2::new(-1.0, 0.0) + offset,
        UiEntityPosition::MiddleMiddleWindow { offset } => Vec2::new(0.0, 0.0) + *offset,
        UiEntityPosition::MiddleRightWindow { offset } => Vec2::new(1.0, 0.0) + offset,

        UiEntityPosition::BottomLeftWindow { offset } => Vec2::new(-1.0, -ratio) + offset,
        UiEntityPosition::BottomMiddleWindow { offset } => Vec2::new(0.0, -ratio) + offset,
        UiEntityPosition::BottomRightWindow { offset } => Vec2::new(1.0, -ratio) + offset,
    }
}
