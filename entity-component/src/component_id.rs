use std::{any::TypeId, sync::Arc};

use bytifex_utils::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex},
    sync::types::ArcMutex,
};

use super::component::ComponentTrait;

#[derive(Clone)]
pub struct ComponentId {
    pub(super) component_type_id: TypeId,
    pub(super) object_pool_index: ObjectPoolIndex,

    pub(super) usage_counter: Arc<()>,
    pub(super) component_storage: ArcMutex<ObjectPool<Box<dyn ComponentTrait>>>,
}

impl ComponentId {
    pub fn is_component_type_of<ComponentType>(&self) -> bool
    where
        ComponentType: ComponentTrait,
    {
        self.component_type_id == TypeId::of::<ComponentType>()
    }
}

impl PartialEq for ComponentId {
    fn eq(&self, other: &Self) -> bool {
        self.component_type_id == other.component_type_id
            && self.object_pool_index == other.object_pool_index
    }
}

impl Eq for ComponentId {}

impl Ord for ComponentId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.component_type_id.cmp(&other.component_type_id) {
            std::cmp::Ordering::Equal => self.object_pool_index.cmp(&other.object_pool_index),
            non_equal_order => non_equal_order,
        }
    }
}

impl PartialOrd for ComponentId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for ComponentId {
    fn drop(&mut self) {
        // only self.usage_counter exist
        if Arc::strong_count(&self.usage_counter) == 1 {
            self.component_storage
                .lock()
                .release_object(self.object_pool_index);
        }
    }
}
