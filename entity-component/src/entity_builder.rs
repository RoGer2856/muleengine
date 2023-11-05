use std::any::TypeId;

use crate::EntityContainer;

use super::{component::ComponentTrait, EntityId};

pub struct EntityBuilder {
    entity_container: EntityContainer,
    components: Vec<(TypeId, Box<dyn ComponentTrait>)>,
}

impl EntityBuilder {
    pub(super) fn new(entity_container: EntityContainer) -> Self {
        Self {
            entity_container,
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

    pub fn build(self) -> EntityId {
        let mut entity_container_guard = self.entity_container.lock();
        entity_container_guard.add_entity(self.components.into_iter())
    }
}
