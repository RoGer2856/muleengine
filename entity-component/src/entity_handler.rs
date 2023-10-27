use bytifex_utils::sync::{callback_event, types::MutexGuard};

use super::{
    component::ComponentTrait, entity::Entity, entity_modified_event::EntityModifiedEvent,
    multi_type_component_storage::MultiTypeComponentStorage, ComponentId,
};

pub struct EntityHandler<'a, 'entity_container_guards> {
    pub(super) entity: &'a mut Entity,
    pub(super) component_storages_guard:
        *mut MutexGuard<'entity_container_guards, MultiTypeComponentStorage>,
    pub(super) entity_modified_event_guard:
        &'a MutexGuard<'entity_container_guards, callback_event::Sender<EntityModifiedEvent>>,
}

impl EntityHandler<'_, '_> {
    pub fn add_component<ComponentType>(&mut self, component: ComponentType) -> Option<ComponentId>
    where
        ComponentType: ComponentTrait,
    {
        let component_type_id = std::any::TypeId::of::<ComponentType>();
        if !self
            .entity
            .component_ids
            .iter()
            .any(|component_id_ref| component_id_ref.component_type_id == component_type_id)
        {
            let component_storage = unsafe {
                (*self.component_storages_guard)
                    .component_storage_mut_for_type_id(&component_type_id)
            };
            let component_id = component_storage.add_component(component);
            self.entity.component_ids.push(component_id.clone());

            self.entity_modified_event_guard
                .trigger(&EntityModifiedEvent::ComponentAdded {
                    entity_id: self.entity.id,
                    component_ids: &self.entity.component_ids,
                    component_id: component_id.clone(),
                });

            Some(component_id)
        } else {
            None
        }
    }

    pub fn remove_component<ComponentType>(&mut self) -> Option<ComponentId>
    where
        ComponentType: ComponentTrait,
    {
        let component_type_id = std::any::TypeId::of::<ComponentType>();
        let index = self
            .entity
            .component_ids
            .iter()
            .position(|component_id_ref| component_id_ref.component_type_id == component_type_id)?;

        let component_id = self.entity.component_ids.swap_remove(index);

        self.entity_modified_event_guard
            .trigger(&EntityModifiedEvent::ComponentRemoved {
                entity_id: self.entity.id,
                component_ids: &self.entity.component_ids,
                component_id: component_id.clone(),
            });

        Some(component_id)
    }

    pub fn get_component_ref<ComponentType>(&self) -> Option<&ComponentType>
    where
        ComponentType: ComponentTrait,
    {
        let component_type_id = std::any::TypeId::of::<ComponentType>();
        let component_id = self
            .entity
            .component_ids
            .iter()
            .find(|component_id_ref| component_id_ref.component_type_id == component_type_id)?;

        let component_storage = unsafe {
            (*self.component_storages_guard).component_storage_mut_for_type_id(&component_type_id)
        };

        component_storage.get_component_ref::<ComponentType>(component_id)
    }

    pub fn change_component<ComponentType>(&mut self, f: impl FnOnce(&mut ComponentType)) -> bool
    where
        ComponentType: ComponentTrait,
    {
        let component_type_id = std::any::TypeId::of::<ComponentType>();
        if let Some(component_id_ref) = self
            .entity
            .component_ids
            .iter()
            .find(|component_id_ref| component_id_ref.component_type_id == component_type_id)
        {
            let component_storage = unsafe {
                (*self.component_storages_guard)
                    .component_storage_mut_for_type_id(&component_type_id)
            };

            if let Some(component_mut) = component_storage.get_component_mut(component_id_ref) {
                f(component_mut);

                self.entity_modified_event_guard
                    .trigger(&EntityModifiedEvent::ComponentChanged {
                        entity_id: self.entity.id,
                        component_id: component_id_ref.clone(),
                        component_ids: &self.entity.component_ids,
                    });

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn iter_entity_component_ids(&self) -> impl std::iter::Iterator<Item = &ComponentId> {
        self.entity.component_ids.iter()
    }
}
