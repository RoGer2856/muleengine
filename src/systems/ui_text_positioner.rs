use entity_component::{
    component_type_list, EntityContainer, EntityContainerGuard, EntityGroup, EntityGroupEvent,
    EntityId,
};
use muleengine::{
    bytifex_utils::sync::{broadcast::Receiver, types::ArcRwLock},
    system_container::System,
    window_context::{Event, EventReceiver, WindowContext},
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

pub struct UiTextPositioner {
    entity_container: EntityContainer,
    entity_group: EntityGroup,
    entity_group_event_receiver: Receiver<EntityGroupEvent>,
    window_context_event_receiver: EventReceiver,
    window_dimensions: Vec2<usize>,
}

impl UiTextPositioner {
    pub fn new(
        entity_container: EntityContainer,
        window_context: ArcRwLock<dyn WindowContext>,
    ) -> Self {
        let entity_group = entity_container
            .lock()
            .entity_group(component_type_list![Transform<f32, f32, f32>, UiEntityPosition]);
        let entity_group_event_receiver =
            entity_group.event_receiver(true, &mut entity_container.lock());
        Self {
            entity_container,
            entity_group,
            entity_group_event_receiver,
            window_context_event_receiver: window_context.read().event_receiver(),
            window_dimensions: window_context.read().window_dimensions(),
        }
    }

    fn set_transform_of_entity(
        &self,
        entity_id: EntityId,
        entity_container_guard: &mut EntityContainerGuard,
    ) {
        if let Some(mut entity_handler) = entity_container_guard.handler_for_entity(&entity_id) {
            log::error!("this is called, because the sender is not specified");
            let position =
                if let Some(component) = entity_handler.get_component_ref::<UiEntityPosition>() {
                    component.clone()
                } else {
                    return;
                };

            entity_handler.change_component(|transform: &mut Transform<f32, f32, f32>| {
                let pos = self.compute_position_of_entity(&position);
                transform.position.x = pos.x;
                transform.position.y = pos.y;
            });
        }
    }

    fn compute_position_of_entity(&self, position: &UiEntityPosition) -> Vec2<f32> {
        let ratio = self.window_dimensions.y as f32 / self.window_dimensions.x as f32;
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
}

impl System for UiTextPositioner {
    fn tick(&mut self, _loop_start: &std::time::Instant, _last_loop_time_secs: f32) {
        let mut entity_container_guard = self.entity_container.lock();

        let mut new_window_dimensions = None;
        while let Some(event) = self.window_context_event_receiver.pop() {
            if let Event::Resized { width, height } = event {
                new_window_dimensions = Some(Vec2::new(width, height));
            }
        }

        if let Some(window_dimensions) = new_window_dimensions {
            self.window_dimensions = window_dimensions;

            for entity_id in self.entity_group.iter_entity_ids() {
                self.set_transform_of_entity(entity_id, &mut entity_container_guard);
            }
        }

        while let Some(event) = self.entity_group_event_receiver.pop() {
            if new_window_dimensions.is_none() {
                if let EntityGroupEvent::EntityAdded { entity_id } = event {
                    self.set_transform_of_entity(entity_id, &mut entity_container_guard);
                } else if let EntityGroupEvent::ComponentChanged {
                    entity_id,
                    component_id,
                } = event
                {
                    if component_id.is_component_type_of::<UiEntityPosition>() {
                        self.set_transform_of_entity(entity_id, &mut entity_container_guard);
                    }
                }
            }
        }
    }
}
