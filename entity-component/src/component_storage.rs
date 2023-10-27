use std::{any::TypeId, sync::Arc};

use bytifex_utils::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex},
    sync::types::{arc_mutex_new, ArcMutex},
};

use super::{component::ComponentTrait, ComponentId};

pub(super) struct ComponentStorage {
    pub _component_type_id: TypeId,
    pub components: ObjectPool<Box<dyn ComponentTrait>>,
    to_be_removed_indices: ArcMutex<Vec<ObjectPoolIndex>>,
}

impl ComponentStorage {
    pub fn new(component_type_id: TypeId) -> Self {
        Self {
            _component_type_id: component_type_id,
            components: ObjectPool::new(),
            to_be_removed_indices: arc_mutex_new(Vec::new()),
        }
    }

    pub fn add_component<ComponentType>(&mut self, component: ComponentType) -> ComponentId
    where
        ComponentType: ComponentTrait,
    {
        self.add_component_any(&TypeId::of::<ComponentType>(), Box::new(component))
    }

    pub fn add_component_any(
        &mut self,
        component_type_id: &TypeId,
        component: Box<dyn ComponentTrait>,
    ) -> ComponentId {
        self.handle_to_be_removed_indices();

        ComponentId {
            component_type_id: *component_type_id,
            object_pool_index: self.components.create_object(component),
            usage_counter: Arc::new(()),
            to_be_removed_indices: self.to_be_removed_indices.clone(),
        }
    }

    pub fn get_component_ref<ComponentType>(&self, id: &ComponentId) -> Option<&ComponentType>
    where
        ComponentType: ComponentTrait,
    {
        (*self.get_component_ref_any(id)?)
            .as_any()
            .downcast_ref::<ComponentType>()
    }

    pub fn get_component_ref_any(&self, id: &ComponentId) -> Option<&dyn ComponentTrait> {
        Some(self.components.get_ref(id.object_pool_index)?.as_ref())
    }

    pub fn get_component_mut<ComponentType>(
        &mut self,
        id: &ComponentId,
    ) -> Option<&mut ComponentType>
    where
        ComponentType: ComponentTrait,
    {
        (**self.get_component_mut_any(id)?)
            .as_any_mut()
            .downcast_mut::<ComponentType>()
    }

    pub fn get_component_mut_any(
        &mut self,
        id: &ComponentId,
    ) -> Option<&mut Box<dyn ComponentTrait>> {
        self.handle_to_be_removed_indices();

        let component_box = self.components.get_mut(id.object_pool_index)?;
        Some(component_box)
    }

    fn handle_to_be_removed_indices(&mut self) {
        let mut guard = self.to_be_removed_indices.lock();
        for index in guard.iter() {
            self.components.release_object(*index);
        }

        guard.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_read_modify_read_component() {
        let mut component_storage = ComponentStorage::new(TypeId::of::<String>());

        // add a component
        let id = component_storage.add_component("initial string".to_string());

        // read the component
        assert_eq!(
            component_storage.get_component_ref::<String>(&id).cloned(),
            Some("initial string".to_string())
        );

        // change the component
        let component_opt_mut = component_storage.get_component_mut::<String>(&id);
        assert!(component_opt_mut.is_some());
        if let Some(component_mut) = component_opt_mut {
            *component_mut = "modified string".to_string();
        }

        // read the component
        assert_eq!(
            component_storage.get_component_ref::<String>(&id).cloned(),
            Some("modified string".to_string())
        );

        let object_pool_index = id.object_pool_index;

        // remove the component
        drop(id);

        // force component_storage to run its cleanup method
        component_storage.handle_to_be_removed_indices();

        // read the component
        assert!(component_storage
            .components
            .get_mut(object_pool_index)
            .is_none());
    }
}
