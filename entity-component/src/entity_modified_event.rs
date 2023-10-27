use super::{ComponentId, EntityId};

pub(super) enum EntityModifiedEvent {
    EntityAdded {
        entity_id: EntityId,
        component_ids: *const Vec<ComponentId>,
    },
    EntityRemoved {
        entity_id: EntityId,
        component_ids: *const Vec<ComponentId>,
    },
    ComponentAdded {
        entity_id: EntityId,
        component_id: ComponentId,
        component_ids: *const Vec<ComponentId>,
    },
    ComponentRemoved {
        entity_id: EntityId,
        component_ids: *const Vec<ComponentId>,
        component_id: ComponentId,
    },
    ComponentChanged {
        entity_id: EntityId,
        component_id: ComponentId,
        component_ids: *const Vec<ComponentId>,
    },
}
