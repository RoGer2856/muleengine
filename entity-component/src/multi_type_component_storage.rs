use std::{any::TypeId, collections::BTreeMap};

use crate::component_storage::{ComponentAnyGuard, ComponentGuard};

use super::{component::ComponentTrait, component_storage::ComponentStorage, ComponentId};

pub struct MultiTypeComponentStorage {
    pub(super) component_storages: BTreeMap<TypeId, ComponentStorage>,
}

impl Default for MultiTypeComponentStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiTypeComponentStorage {
    pub fn new() -> Self {
        Self {
            component_storages: BTreeMap::new(),
        }
    }

    pub fn get_component_ref<ComponentType>(
        &self,
        id: &ComponentId,
    ) -> Option<ComponentGuard<ComponentType>>
    where
        ComponentType: ComponentTrait,
    {
        let storage = self.component_storage_ref_for_type_id(&id.component_type_id)?;
        storage.get_component_ref(id)
    }

    pub fn get_component_ref_any(&self, id: &ComponentId) -> Option<ComponentAnyGuard> {
        let storage = self.component_storage_ref_for_type_id(&id.component_type_id)?;
        storage.get_component_ref_any(id)
    }

    pub(super) fn component_storage_ref_for_type_id(
        &self,
        component_type_id: &TypeId,
    ) -> Option<&ComponentStorage> {
        self.component_storages.get(component_type_id)
    }

    pub(super) fn component_storage_mut_for_type_id<'a>(
        &'a mut self,
        component_type_id: &TypeId,
    ) -> &'a mut ComponentStorage {
        self.component_storages
            .entry(*component_type_id)
            .or_insert_with(|| ComponentStorage::new(*component_type_id))
    }
}
