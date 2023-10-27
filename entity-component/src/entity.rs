use super::{ComponentId, EntityId};

pub(super) struct Entity {
    pub(super) id: EntityId,
    pub(super) component_ids: Vec<ComponentId>,
}
