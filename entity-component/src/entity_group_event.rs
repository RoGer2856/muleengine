use super::{ComponentId, EntityId};

#[derive(Clone)]
pub enum EntityGroupEvent {
    EntityAdded {
        entity_id: EntityId,
    },
    EntityRemoved {
        entity_id: EntityId,
    },
    ComponentAdded {
        entity_id: EntityId,
        component_id: ComponentId,
    },
    ComponentRemoved {
        entity_id: EntityId,
        component_id: ComponentId,
    },
    ComponentChanged {
        entity_id: EntityId,
        component_id: ComponentId,
    },
}
