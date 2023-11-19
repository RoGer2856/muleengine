use std::{
    any::TypeId,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bytifex_utils::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex},
    sync::{
        types::{arc_mutex_new, ArcMutex, MutexGuard},
        usage_counter::UsageCounter,
    },
};

use super::{component::ComponentTrait, ComponentId};

pub struct ComponentAnyGuard<'a> {
    components_guard: MutexGuard<'a, ObjectPool<Box<dyn ComponentTrait>>>,
    object_pool_index: ObjectPoolIndex,
}

impl<'a> Deref for ComponentAnyGuard<'a> {
    type Target = dyn ComponentTrait;

    fn deref(&self) -> &Self::Target {
        match self.components_guard.get_ref(self.object_pool_index) {
            Some(component_box) => component_box,
            None => unreachable!(),
        }
    }
}

pub struct ComponentGuard<'a, ComponentType> {
    components_guard: MutexGuard<'a, ObjectPool<Box<dyn ComponentTrait>>>,
    object_pool_index: ObjectPoolIndex,
    marker: PhantomData<ComponentType>,
}

impl<'a, ComponentType: ComponentTrait> Deref for ComponentGuard<'a, ComponentType> {
    type Target = ComponentType;
    fn deref(&self) -> &Self::Target {
        match self.components_guard.get_ref(self.object_pool_index) {
            Some(component_box) => {
                match (**component_box).as_any().downcast_ref::<ComponentType>() {
                    Some(component) => component,
                    None => unreachable!(),
                }
            }
            None => unreachable!(),
        }
    }
}

pub struct ComponentMutGuard<'a, ComponentType: ComponentTrait> {
    components_guard: MutexGuard<'a, ObjectPool<Box<dyn ComponentTrait>>>,
    object_pool_index: ObjectPoolIndex,
    marker: PhantomData<ComponentType>,
}

impl<'a, ComponentType: ComponentTrait> Deref for ComponentMutGuard<'a, ComponentType> {
    type Target = ComponentType;

    fn deref(&self) -> &Self::Target {
        match self.components_guard.get_ref(self.object_pool_index) {
            Some(component_box) => {
                match (**component_box).as_any().downcast_ref::<ComponentType>() {
                    Some(component) => component,
                    None => unreachable!(),
                }
            }
            None => unreachable!(),
        }
    }
}

impl<'a, ComponentType: ComponentTrait> DerefMut for ComponentMutGuard<'a, ComponentType> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.components_guard.get_mut(self.object_pool_index) {
            Some(component_box) => {
                match (**component_box)
                    .as_any_mut()
                    .downcast_mut::<ComponentType>()
                {
                    Some(component) => component,
                    None => unreachable!(),
                }
            }
            None => unreachable!(),
        }
    }
}

pub(super) struct ComponentStorage {
    pub component_type_id: TypeId,
    pub components: ArcMutex<ObjectPool<Box<dyn ComponentTrait>>>,
}

impl ComponentStorage {
    pub fn new(component_type_id: TypeId) -> Self {
        Self {
            component_type_id,
            components: arc_mutex_new(ObjectPool::new()),
        }
    }

    pub fn add_component<ComponentType>(&mut self, component: ComponentType) -> Option<ComponentId>
    where
        ComponentType: ComponentTrait,
    {
        self.add_component_any(&TypeId::of::<ComponentType>(), Box::new(component))
    }

    pub fn add_component_any(
        &mut self,
        component_type_id: &TypeId,
        component: Box<dyn ComponentTrait>,
    ) -> Option<ComponentId> {
        if self.component_type_id != *component_type_id {
            return None;
        }

        Some(ComponentId {
            component_type_id: *component_type_id,
            object_pool_index: self.components.lock().create_object(component),
            usage_counter: UsageCounter::new(),
            component_storage: self.components.clone(),
        })
    }

    pub fn get_component_ref<ComponentType>(
        &self,
        id: &ComponentId,
    ) -> Option<ComponentGuard<ComponentType>>
    where
        ComponentType: ComponentTrait,
    {
        if self.component_type_id != TypeId::of::<ComponentType>() {
            return None;
        }

        let mut components_guard = self.components.lock();
        components_guard.get_mut(id.object_pool_index)?;
        Some(ComponentGuard {
            components_guard,
            object_pool_index: id.object_pool_index,
            marker: PhantomData,
        })
    }

    pub fn get_component_ref_any(&self, id: &ComponentId) -> Option<ComponentAnyGuard> {
        let mut components_guard = self.components.lock();
        components_guard.get_mut(id.object_pool_index)?;
        Some(ComponentAnyGuard {
            components_guard,
            object_pool_index: id.object_pool_index,
        })
    }

    pub fn get_component_mut<ComponentType>(
        &mut self,
        id: &ComponentId,
    ) -> Option<ComponentMutGuard<ComponentType>>
    where
        ComponentType: ComponentTrait,
    {
        if self.component_type_id != TypeId::of::<ComponentType>() {
            return None;
        }

        let mut components_guard = self.components.lock();
        let component_box = components_guard.get_mut(id.object_pool_index)?;
        (**component_box)
            .as_any_mut()
            .downcast_mut::<ComponentType>()?;
        Some(ComponentMutGuard {
            components_guard,
            object_pool_index: id.object_pool_index,
            marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_read_modify_read_component() {
        let mut component_storage = ComponentStorage::new(TypeId::of::<String>());

        // add a component
        let id = component_storage
            .add_component("initial string".to_string())
            .unwrap();

        // read the component
        assert_eq!(
            component_storage
                .get_component_ref::<String>(&id)
                .unwrap()
                .clone(),
            "initial string".to_string()
        );

        // change the component
        {
            let mut component_mut = component_storage.get_component_mut::<String>(&id).unwrap();
            *component_mut = "modified string".to_string();
        }

        // read the component
        assert_eq!(
            component_storage
                .get_component_ref::<String>(&id)
                .unwrap()
                .clone(),
            "modified string".to_string()
        );

        let object_pool_index = id.object_pool_index;

        // remove the component
        drop(id);

        // read the component
        assert!(component_storage
            .components
            .lock()
            .get_mut(object_pool_index)
            .is_none());
    }
}
