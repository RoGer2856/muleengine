use std::any::TypeId;

use super::{component::ComponentTrait, EntityContainerGuard, EntityId};

pub struct EntityBuilder<'entity_container_guards> {
    entity_container_guard: EntityContainerGuard<'entity_container_guards>,
    components: Vec<(TypeId, Box<dyn ComponentTrait>)>,
}

impl<'entity_container_guards> EntityBuilder<'entity_container_guards> {
    pub(super) fn new(
        entity_container_guard: EntityContainerGuard<'entity_container_guards>,
    ) -> Self {
        Self {
            entity_container_guard,
            components: Vec::new(),
        }
    }

    pub fn with_component<ComponentType>(mut self, component: ComponentType) -> Self
    where
        ComponentType: ComponentTrait,
    {
        let component_type_id = TypeId::of::<ComponentType>();
        if let Some(component_mut) =
            self.components
                .iter_mut()
                .find_map(|(type_id, component_mut)| {
                    if *type_id == component_type_id {
                        Some(component_mut)
                    } else {
                        None
                    }
                })
        {
            *component_mut = Box::new(component);
        } else {
            self.components
                .push((component_type_id, Box::new(component)));
        }

        self
    }

    pub fn build(mut self) -> EntityId {
        self.entity_container_guard
            .add_entity(self.components.into_iter())
    }
}
