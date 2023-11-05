use std::any::TypeId;

use bytifex_utils::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex, ObjectPoolIterMut},
    sync::{
        callback_event,
        types::{arc_mutex_new, ArcMutex, MutexGuard},
    },
};

use super::{
    component::ComponentTrait, component_type_list::ToSortedComponentTypeList, entity::Entity,
    entity_modified_event::EntityModifiedEvent,
    multi_type_component_storage::MultiTypeComponentStorage, ComponentId, EntityBuilder,
    EntityGroup, EntityHandler,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EntityId(pub ObjectPoolIndex);

pub struct EntityContainerGuard<'entity_container_guards> {
    entities_guard: MutexGuard<'entity_container_guards, ObjectPool<Entity>>,
    component_storages_guard: MutexGuard<'entity_container_guards, MultiTypeComponentStorage>,

    entity_modified_event_guard:
        MutexGuard<'entity_container_guards, callback_event::Sender<EntityModifiedEvent>>,
}

#[derive(Clone)]
pub struct EntityContainer {
    entities: ArcMutex<ObjectPool<Entity>>,
    component_storages: ArcMutex<MultiTypeComponentStorage>,

    entity_modified_event: ArcMutex<callback_event::Sender<EntityModifiedEvent>>,
}

impl Default for EntityContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityContainer {
    pub fn new() -> Self {
        Self {
            entities: arc_mutex_new(ObjectPool::new()),
            component_storages: arc_mutex_new(MultiTypeComponentStorage::new()),

            entity_modified_event: arc_mutex_new(callback_event::Sender::new()),
        }
    }

    pub fn lock(&self) -> EntityContainerGuard<'_> {
        EntityContainerGuard {
            entities_guard: self.entities.lock(),
            component_storages_guard: self.component_storages.lock(),

            entity_modified_event_guard: self.entity_modified_event.lock(),
        }
    }

    pub fn entity_builder(&self) -> EntityBuilder {
        EntityBuilder::new(self.clone())
    }
}

impl<'entity_container_guards> EntityContainerGuard<'entity_container_guards> {
    pub fn entity_group(
        &mut self,
        component_type_list: impl ToSortedComponentTypeList,
    ) -> EntityGroup {
        EntityGroup::new(
            self,
            component_type_list,
            self.entity_modified_event_guard.create_subscriber(),
        )
    }

    pub fn component_storage(&self) -> &MultiTypeComponentStorage {
        &self.component_storages_guard
    }

    pub fn add_entity(
        &mut self,
        components_iter: impl Iterator<Item = (TypeId, Box<dyn ComponentTrait>)>,
    ) -> EntityId {
        let component_ids: Vec<ComponentId> = components_iter
            .into_iter()
            .map(|(type_id, component)| {
                match self
                    .component_storages_guard
                    .component_storage_mut_for_type_id(&type_id)
                    .add_component_any(&type_id, component)
                {
                    Some(component_id) => component_id,
                    None => unreachable!(),
                }
            })
            .collect();

        let id = EntityId(self.entities_guard.create_object(Entity {
            id: EntityId(ObjectPoolIndex::invalid()),
            component_ids,
        }));

        if let Some(entity) = self.entities_guard.get_mut(id.0) {
            entity.id = id;

            self.entity_modified_event_guard
                .trigger(&EntityModifiedEvent::EntityAdded {
                    entity_id: id,
                    component_ids: &entity.component_ids,
                });
        } else {
            unreachable!();
        }

        id
    }

    pub fn remove_entity(&mut self, entity_id: &EntityId) -> bool {
        if let Some(entity) = self.entities_guard.release_object(entity_id.0) {
            self.entity_modified_event_guard
                .trigger(&EntityModifiedEvent::EntityRemoved {
                    entity_id: *entity_id,
                    component_ids: &entity.component_ids,
                });

            true
        } else {
            false
        }
    }

    pub fn iter(&mut self) -> EntityContainerIter<'_, 'entity_container_guards> {
        EntityContainerIter {
            inner_iterator: self.entities_guard.iter_mut(),
            component_storages_guard: &mut self.component_storages_guard,
            entity_modified_event_guard: &self.entity_modified_event_guard,
        }
    }

    pub fn handler_for_entity<'a>(
        &'a mut self,
        entity_id: &EntityId,
    ) -> Option<EntityHandler<'a, 'entity_container_guards>> {
        Some(EntityHandler {
            component_storages_guard: &mut self.component_storages_guard,
            entity: self.entities_guard.get_mut(entity_id.0)?,
            entity_modified_event_guard: &self.entity_modified_event_guard,
        })
    }
}

pub struct EntityContainerIter<'a, 'entity_container_guards> {
    inner_iterator: ObjectPoolIterMut<'a, Entity>,
    component_storages_guard:
        &'a mut MutexGuard<'entity_container_guards, MultiTypeComponentStorage>,
    entity_modified_event_guard:
        &'a MutexGuard<'entity_container_guards, callback_event::Sender<EntityModifiedEvent>>,
}

impl<'a, 'entity_container_guards> Iterator for EntityContainerIter<'a, 'entity_container_guards> {
    type Item = EntityHandler<'a, 'entity_container_guards>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = self.inner_iterator.next() {
            Some(EntityHandler {
                entity,
                component_storages_guard: self.component_storages_guard,
                entity_modified_event_guard: self.entity_modified_event_guard,
            })
        } else {
            None
        }
    }
}
