use std::{
    any::TypeId,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use bytifex_utils::sync::{
    callback_event,
    types::{arc_mutex_new, ArcMutex, MutexGuard},
};
use EntityHandlingType::*;

use crate::{EntityGroupEventReceiver, EntityGroupEventSender};

use super::{
    component_type_list::ToSortedComponentTypeList, entity_modified_event::EntityModifiedEvent,
    EntityContainerGuard, EntityGroupEvent, EntityId,
};

pub struct EntityGroup {
    sorted_component_type_list: Vec<TypeId>,
    entity_ids: ArcMutex<Vec<EntityId>>,

    to_be_handled_entity_ids: ArcMutex<Vec<(EntityId, EntityHandlingType)>>,
    is_locked_by_itself: Arc<AtomicBool>,

    entity_group_event_sender: EntityGroupEventSender,

    _entity_modified_event_subscription: callback_event::Subscription<EntityModifiedEvent>,
}

impl EntityGroup {
    pub fn event_receiver(
        &self,
        resend_events: bool,
        entity_container_guard: &mut EntityContainerGuard<'_>,
    ) -> EntityGroupEventReceiver {
        let receiver = self.entity_group_event_sender.create_receiver();

        if resend_events {
            self.resend_events_directly(&receiver, entity_container_guard);
        }

        receiver
    }

    pub fn contains(&self, entity_id: &EntityId) -> bool {
        self.entity_ids.lock().contains(entity_id)
    }

    pub fn iter_entity_ids(&self) -> EntityGroupIterator {
        self.is_locked_by_itself.fetch_or(true, Ordering::SeqCst);
        EntityGroupIterator {
            entity_ids_guard: self.entity_ids.lock(),
            next_index: 0,

            is_locked_by_itself: self.is_locked_by_itself.clone(),
            to_be_handled_entity_ids: self.to_be_handled_entity_ids.clone(),
        }
    }

    pub(super) fn new(
        entity_container_guard: &mut EntityContainerGuard,
        component_type_list: impl ToSortedComponentTypeList,
        entity_modified_event_subscriber: callback_event::Subscriber<EntityModifiedEvent>,
    ) -> Self {
        let entity_ids = arc_mutex_new(Vec::new());
        let sorted_component_type_list = component_type_list.to_sorted_component_type_list();
        let entity_group_event = EntityGroupEventSender::new();
        let is_locked_by_itself = Arc::new(AtomicBool::new(false));
        let to_be_handled_entity_ids = arc_mutex_new(Vec::new());

        let mut ret = Self {
            sorted_component_type_list: sorted_component_type_list.clone(),
            entity_ids: entity_ids.clone(),

            is_locked_by_itself: is_locked_by_itself.clone(),
            to_be_handled_entity_ids: to_be_handled_entity_ids.clone(),

            entity_group_event_sender: entity_group_event.clone(),

            _entity_modified_event_subscription: entity_modified_event_subscriber.subscribe(
                move |event| {
                    process_entity_container_event(
                        event,
                        &entity_ids,
                        &sorted_component_type_list,
                        &entity_group_event,
                        &is_locked_by_itself,
                        &to_be_handled_entity_ids,
                    )
                },
            ),
        };

        ret.initialize(entity_container_guard);

        ret
    }

    fn initialize(&mut self, entity_container_guard: &mut EntityContainerGuard) {
        *self.entity_ids.lock() = entity_container_guard
            .iter()
            .filter_map(|entity_handler| {
                let mut add_entity_id = true;
                for necessary_component_type_id_ref in self.sorted_component_type_list.iter() {
                    if !entity_handler
                        .iter_entity_component_ids()
                        .any(|component_id_ref| {
                            component_id_ref.component_type_id == *necessary_component_type_id_ref
                        })
                    {
                        add_entity_id = false;
                        break;
                    }
                }

                if add_entity_id {
                    Some(entity_handler.entity.id)
                } else {
                    None
                }
            })
            .collect();
    }

    fn resend_events_directly(
        &self,
        receiver: &EntityGroupEventReceiver,
        entity_container_guard: &mut EntityContainerGuard<'_>,
    ) {
        for entity_id_ref in self.entity_ids.lock().iter() {
            if let Some(entity_handler) = entity_container_guard.handler_for_entity(entity_id_ref) {
                self.entity_group_event_sender.send_directly(
                    EntityGroupEvent::EntityAdded {
                        entity_id: *entity_id_ref,
                    },
                    receiver,
                );

                for component_id_ref in entity_handler.iter_entity_component_ids() {
                    self.entity_group_event_sender.send_directly(
                        EntityGroupEvent::ComponentAdded {
                            entity_id: *entity_id_ref,
                            component_id: component_id_ref.clone(),
                        },
                        receiver,
                    )
                }
            }
        }
    }
}

pub struct EntityGroupIterator<'a> {
    entity_ids_guard: MutexGuard<'a, Vec<EntityId>>,
    next_index: usize,

    to_be_handled_entity_ids: ArcMutex<Vec<(EntityId, EntityHandlingType)>>,
    is_locked_by_itself: Arc<AtomicBool>,
}

impl EntityGroupIterator<'_> {
    fn handle_to_be_removed_entity_ids(&mut self) {
        let mut to_be_handled_entity_ids = self.to_be_handled_entity_ids.lock();
        for (entity_id, action) in to_be_handled_entity_ids.iter() {
            match action {
                Add => {
                    self.entity_ids_guard.push(*entity_id);
                }
                Remove => {
                    if let Some(index) = self
                        .entity_ids_guard
                        .iter()
                        .position(|entity_id_ref| *entity_id_ref == *entity_id)
                    {
                        self.entity_ids_guard.swap_remove(index);
                    }
                }
            }
        }
        to_be_handled_entity_ids.clear();
    }
}

impl Iterator for EntityGroupIterator<'_> {
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.entity_ids_guard.len() {
            let ret = self.entity_ids_guard[self.next_index];
            self.next_index += 1;
            Some(ret)
        } else {
            self.handle_to_be_removed_entity_ids();

            None
        }
    }
}

impl Drop for EntityGroupIterator<'_> {
    fn drop(&mut self) {
        self.is_locked_by_itself.fetch_and(false, Ordering::SeqCst);
        self.handle_to_be_removed_entity_ids();
    }
}

fn matching_component_type_count(
    type_ids: impl Iterator<Item = TypeId>,
    component_type_list: &[std::any::TypeId],
) -> usize {
    let mut component_match_count = 0;

    for type_id in type_ids {
        for component_type_id in component_type_list {
            if *component_type_id == type_id {
                component_match_count += 1;
                break;
            }
        }
    }

    component_match_count
}

enum EntityHandlingType {
    Add,
    Remove,
}

fn process_entity_container_event(
    event: &EntityModifiedEvent,
    entity_ids: &ArcMutex<Vec<EntityId>>,
    sorted_component_type_list: &[TypeId],
    entity_group_event: &EntityGroupEventSender,
    is_locked_by_itself: &Arc<AtomicBool>,
    to_be_handled_entity_ids: &ArcMutex<Vec<(EntityId, EntityHandlingType)>>,
) {
    match event {
        EntityModifiedEvent::EntityAdded {
            entity_id,
            component_ids,
        } => {
            let component_ids_ref = unsafe { &**component_ids };
            if matching_component_type_count(
                component_ids_ref
                    .iter()
                    .map(|component_id| component_id.component_type_id),
                sorted_component_type_list,
            ) == sorted_component_type_list.len()
            {
                if is_locked_by_itself.fetch_and(true, Ordering::SeqCst) {
                    to_be_handled_entity_ids.lock().push((*entity_id, Add));
                } else {
                    entity_ids.lock().push(*entity_id);
                }

                // send events
                entity_group_event.send(EntityGroupEvent::EntityAdded {
                    entity_id: *entity_id,
                });

                for component_id_ref in component_ids_ref.iter() {
                    entity_group_event.send(EntityGroupEvent::ComponentAdded {
                        entity_id: *entity_id,
                        component_id: component_id_ref.clone(),
                    });
                }
            }
        }
        EntityModifiedEvent::EntityRemoved {
            entity_id,
            component_ids,
        } => {
            let component_ids_ref = unsafe { &**component_ids };
            if matching_component_type_count(
                component_ids_ref
                    .iter()
                    .map(|component_id| component_id.component_type_id),
                sorted_component_type_list,
            ) == sorted_component_type_list.len()
            {
                if is_locked_by_itself.fetch_and(true, Ordering::SeqCst) {
                    to_be_handled_entity_ids.lock().push((*entity_id, Remove));
                } else {
                    let mut entity_ids = entity_ids.lock();
                    if let Some(index) = entity_ids
                        .iter()
                        .position(|entity_id_ref| *entity_id_ref == *entity_id)
                    {
                        entity_ids.swap_remove(index);
                    }
                }

                // send events
                for component_id in component_ids_ref.iter() {
                    entity_group_event.send(EntityGroupEvent::ComponentRemoved {
                        entity_id: *entity_id,
                        component_id: component_id.clone(),
                    });
                }

                entity_group_event.send(EntityGroupEvent::EntityRemoved {
                    entity_id: *entity_id,
                });
            }
        }
        EntityModifiedEvent::ComponentAdded {
            entity_id,
            component_id,
            component_ids,
        } => {
            let component_ids_ref = unsafe { &**component_ids };
            if sorted_component_type_list.contains(&component_id.component_type_id) {
                if matching_component_type_count(
                    component_ids_ref
                        .iter()
                        .map(|component_id| component_id.component_type_id),
                    sorted_component_type_list,
                ) == sorted_component_type_list.len()
                {
                    if is_locked_by_itself.fetch_and(true, Ordering::SeqCst) {
                        to_be_handled_entity_ids.lock().push((*entity_id, Add));
                    } else {
                        entity_ids.lock().push(*entity_id);
                    }

                    // send events
                    entity_group_event.send(EntityGroupEvent::EntityAdded {
                        entity_id: *entity_id,
                    });

                    for component_id_ref in component_ids_ref.iter() {
                        entity_group_event.send(EntityGroupEvent::ComponentAdded {
                            entity_id: *entity_id,
                            component_id: component_id_ref.clone(),
                        });
                    }
                }
            } else {
                #[warn(clippy::collapsible_else_if)]
                if matching_component_type_count(
                    component_ids_ref
                        .iter()
                        .map(|component_id| component_id.component_type_id),
                    sorted_component_type_list,
                ) == sorted_component_type_list.len()
                {
                    // send event
                    entity_group_event.send(EntityGroupEvent::ComponentAdded {
                        entity_id: *entity_id,
                        component_id: component_id.clone(),
                    });
                }
            }
        }
        EntityModifiedEvent::ComponentRemoved {
            entity_id,
            component_ids,
            component_id,
        } => {
            let component_ids_ref = unsafe { &**component_ids };
            if sorted_component_type_list.contains(&component_id.component_type_id) {
                if matching_component_type_count(
                    component_ids_ref
                        .iter()
                        .map(|component_id| component_id.component_type_id),
                    sorted_component_type_list,
                ) + 1
                    == sorted_component_type_list.len()
                {
                    if is_locked_by_itself.fetch_and(true, Ordering::SeqCst) {
                        to_be_handled_entity_ids.lock().push((*entity_id, Remove));
                    } else {
                        let mut entity_ids = entity_ids.lock();
                        if let Some(index) = entity_ids
                            .iter()
                            .position(|entity_id_ref| *entity_id_ref == *entity_id)
                        {
                            entity_ids.swap_remove(index);
                        }
                    }

                    // send events
                    entity_group_event.send(EntityGroupEvent::ComponentRemoved {
                        entity_id: *entity_id,
                        component_id: component_id.clone(),
                    });

                    for component_id_ref in component_ids_ref.iter() {
                        entity_group_event.send(EntityGroupEvent::ComponentRemoved {
                            entity_id: *entity_id,
                            component_id: component_id_ref.clone(),
                        });
                    }

                    entity_group_event.send(EntityGroupEvent::EntityRemoved {
                        entity_id: *entity_id,
                    });
                }
            } else {
                #[warn(clippy::collapsible_else_if)]
                if matching_component_type_count(
                    component_ids_ref
                        .iter()
                        .map(|component_id| component_id.component_type_id),
                    sorted_component_type_list,
                ) == sorted_component_type_list.len()
                {
                    // send event
                    entity_group_event.send(EntityGroupEvent::ComponentRemoved {
                        entity_id: *entity_id,
                        component_id: component_id.clone(),
                    });
                }
            }
        }
        EntityModifiedEvent::ComponentChanged {
            entity_id,
            component_id,
            component_ids,
        } => {
            if sorted_component_type_list.contains(&component_id.component_type_id) {
                let component_ids_ref = unsafe { &**component_ids };
                if matching_component_type_count(
                    component_ids_ref
                        .iter()
                        .map(|component_id| component_id.component_type_id),
                    sorted_component_type_list,
                ) == sorted_component_type_list.len()
                {
                    // send event
                    entity_group_event.send(EntityGroupEvent::ComponentChanged {
                        entity_id: *entity_id,
                        component_id: component_id.clone(),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{component_type_list, EntityContainer};

    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Position(String);

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Velocity(String);

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Orientation(String);

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Camera(String);

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Playable(String);

    fn add_test_entities(entity_container: &mut EntityContainer) -> Vec<EntityId> {
        let mut entity_ids = Vec::new();

        entity_ids.push(
            entity_container
                .entity_builder()
                .with_component(Position("pos0".to_string()))
                .with_component(Velocity("vel0".to_string()))
                .with_component(Orientation("ori0".to_string()))
                .build(),
        );

        entity_ids.push(
            entity_container
                .entity_builder()
                .with_component(Position("pos1".to_string()))
                .with_component(Velocity("vel1".to_string()))
                .with_component(Orientation("ori1".to_string()))
                .with_component(Camera("cam1".to_string()))
                .build(),
        );

        entity_ids.push(
            entity_container
                .entity_builder()
                .with_component(Position("pos2".to_string()))
                .with_component(Velocity("vel2".to_string()))
                .with_component(Orientation("ori2".to_string()))
                .with_component(Playable("pla2".to_string()))
                .build(),
        );

        entity_ids.push(
            entity_container
                .entity_builder()
                .with_component(Position("pos3".to_string()))
                .with_component(Orientation("ori3".to_string()))
                .with_component(Playable("pla3".to_string()))
                .build(),
        );

        entity_ids.push(
            entity_container
                .entity_builder()
                .with_component(Playable("pla4".to_string()))
                .with_component(Camera("cam4".to_string()))
                .build(),
        );

        entity_ids
    }

    #[test]
    fn entity_group_new() {
        let mut entity_container = EntityContainer::new();

        let ids = add_test_entities(&mut entity_container);

        let entity_group = entity_container.lock().entity_group(component_type_list![
            Position,
            Velocity,
            Orientation,
        ]);

        assert_eq!(entity_group.entity_ids.lock().len(), 3);

        let entity_group_event = entity_group.event_receiver(true, &mut entity_container.lock());

        let mut entity_added_counter: i32 = 0;
        let mut component_added_counter: i32 = 0;
        let mut id0_found = false;
        let mut id1_found = false;
        let mut id2_found = false;
        while let Some(e) = entity_group_event.pop() {
            match e {
                EntityGroupEvent::EntityAdded { entity_id } => {
                    entity_added_counter += 1;

                    if entity_id == ids[0] {
                        id0_found = true;
                    } else if entity_id == ids[1] {
                        id1_found = true;
                    } else if entity_id == ids[2] {
                        id2_found = true;
                    }
                }
                EntityGroupEvent::ComponentAdded { .. } => component_added_counter += 1,
                _ => assert!(false),
            }
        }
        assert_eq!(entity_added_counter, 3);
        assert_eq!(component_added_counter, 11);
        assert!(id0_found);
        assert!(id1_found);
        assert!(id2_found);

        let mut entity_counter: i32 = 0;
        let mut id0_found = false;
        let mut id1_found = false;
        let mut id2_found = false;
        for entity_id in entity_group.iter_entity_ids() {
            entity_counter += 1;

            if entity_id == ids[0] {
                id0_found = true;
            } else if entity_id == ids[1] {
                id1_found = true;
            } else if entity_id == ids[2] {
                id2_found = true;
            }
        }
        assert_eq!(entity_counter, 3);
        assert!(id0_found);
        assert!(id1_found);
        assert!(id2_found);

        // contains
        assert!(entity_group.contains(&ids[0]));
        assert!(entity_group.contains(&ids[1]));
        assert!(entity_group.contains(&ids[2]));
        assert!(!entity_group.contains(&ids[3]));
        assert!(!entity_group.contains(&ids[4]));
    }

    #[test]
    fn entity_group_on_the_fly_adding_components_with_builder() {
        let mut entity_container = EntityContainer::new();

        let entity_group = entity_container.lock().entity_group(component_type_list![
            Position,
            Velocity,
            Orientation,
        ]);

        let entity_group_event = entity_group.event_receiver(true, &mut entity_container.lock());

        let ids = add_test_entities(&mut entity_container);

        let mut entity_added_counter: i32 = 0;
        let mut component_added_counter: i32 = 0;
        let mut id0_found = false;
        let mut id1_found = false;
        let mut id2_found = false;
        while let Some(e) = entity_group_event.pop() {
            match e {
                EntityGroupEvent::EntityAdded { entity_id } => {
                    entity_added_counter += 1;

                    if entity_id == ids[0] {
                        id0_found = true;
                    } else if entity_id == ids[1] {
                        id1_found = true;
                    } else if entity_id == ids[2] {
                        id2_found = true;
                    }
                }
                EntityGroupEvent::ComponentAdded { .. } => component_added_counter += 1,
                _ => assert!(false),
            }
        }
        assert_eq!(entity_added_counter, 3);
        assert_eq!(component_added_counter, 11);
        assert!(id0_found);
        assert!(id1_found);
        assert!(id2_found);

        let mut entity_counter: i32 = 0;
        let mut id0_found = false;
        let mut id1_found = false;
        let mut id2_found = false;
        for entity_id in entity_group.iter_entity_ids() {
            entity_counter += 1;

            if entity_id == ids[0] {
                id0_found = true;
            } else if entity_id == ids[1] {
                id1_found = true;
            } else if entity_id == ids[2] {
                id2_found = true;
            }
        }
        assert_eq!(entity_counter, 3);
        assert!(id0_found);
        assert!(id1_found);
        assert!(id2_found);

        // contains
        assert!(entity_group.contains(&ids[0]));
        assert!(entity_group.contains(&ids[1]));
        assert!(entity_group.contains(&ids[2]));
        assert!(!entity_group.contains(&ids[3]));
        assert!(!entity_group.contains(&ids[4]));
    }

    #[test]
    fn entity_group_on_the_fly_adding_components_without_builder() {
        let entity_container = EntityContainer::new();

        let entity_group = entity_container.lock().entity_group(component_type_list![
            Position,
            Velocity,
            Orientation,
        ]);

        let entity_group_event = entity_group.event_receiver(true, &mut entity_container.lock());

        let id0 = entity_container.entity_builder().build();
        let id1 = entity_container.entity_builder().build();
        let id2 = entity_container.entity_builder().build();
        let id3 = entity_container.entity_builder().build();
        let id4 = entity_container.entity_builder().build();

        let mut entity_container_guard = entity_container.lock();

        let mut entity_handler = entity_container_guard.handler_for_entity(&id0).unwrap();
        entity_handler.add_component(Position("pos0".to_string()));
        entity_handler.add_component(Velocity("vel0".to_string()));
        entity_handler.add_component(Orientation("ori0".to_string()));

        let mut entity_handler = entity_container_guard.handler_for_entity(&id1).unwrap();
        entity_handler.add_component(Position("pos1".to_string()));
        entity_handler.add_component(Velocity("vel1".to_string()));
        entity_handler.add_component(Orientation("ori1".to_string()));
        entity_handler.add_component(Camera("cam1".to_string()));

        let mut entity_handler = entity_container_guard.handler_for_entity(&id2).unwrap();
        entity_handler.add_component(Position("pos2".to_string()));
        entity_handler.add_component(Velocity("vel2".to_string()));
        entity_handler.add_component(Orientation("ori2".to_string()));
        entity_handler.add_component(Playable("pla2".to_string()));

        let mut entity_handler = entity_container_guard.handler_for_entity(&id3).unwrap();
        entity_handler.add_component(Position("pos3".to_string()));
        entity_handler.add_component(Orientation("ori3".to_string()));
        entity_handler.add_component(Playable("pla3".to_string()));

        let mut entity_handler = entity_container_guard.handler_for_entity(&id4).unwrap();
        entity_handler.add_component(Playable("pla4".to_string()));
        entity_handler.add_component(Camera("pla4".to_string()));

        let mut entity_added_counter: i32 = 0;
        let mut component_added_counter: i32 = 0;
        let mut id0_found = false;
        let mut id1_found = false;
        let mut id2_found = false;
        while let Some(e) = entity_group_event.pop() {
            match e {
                EntityGroupEvent::EntityAdded { entity_id } => {
                    entity_added_counter += 1;

                    if entity_id == id0 {
                        id0_found = true;
                    } else if entity_id == id1 {
                        id1_found = true;
                    } else if entity_id == id2 {
                        id2_found = true;
                    }
                }
                EntityGroupEvent::ComponentAdded { .. } => component_added_counter += 1,
                _ => assert!(false),
            }
        }
        assert_eq!(entity_added_counter, 3);
        assert_eq!(component_added_counter, 11);
        assert!(id0_found);
        assert!(id1_found);
        assert!(id2_found);

        let mut entity_counter: i32 = 0;
        let mut id0_found = false;
        let mut id1_found = false;
        let mut id2_found = false;
        for entity_id in entity_group.iter_entity_ids() {
            entity_counter += 1;

            if entity_id == id0 {
                id0_found = true;
            } else if entity_id == id1 {
                id1_found = true;
            } else if entity_id == id2 {
                id2_found = true;
            }
        }
        assert_eq!(entity_counter, 3);
        assert!(id0_found);
        assert!(id1_found);
        assert!(id2_found);

        // contains
        assert!(entity_group.contains(&id0));
        assert!(entity_group.contains(&id1));
        assert!(entity_group.contains(&id2));
        assert!(!entity_group.contains(&id3));
        assert!(!entity_group.contains(&id4));
    }

    #[test]
    fn entity_group_component_removed() {
        let mut entity_container = EntityContainer::new();

        let entity_group = entity_container.lock().entity_group(component_type_list![
            Position,
            Velocity,
            Orientation,
        ]);

        let ids = add_test_entities(&mut entity_container);

        // contains
        assert!(entity_group.contains(&ids[0]));
        assert!(entity_group.contains(&ids[1]));
        assert!(entity_group.contains(&ids[2]));
        assert!(!entity_group.contains(&ids[3]));
        assert!(!entity_group.contains(&ids[4]));

        let mut entity_container_guard = entity_container.lock();

        let entity_group_event = entity_group.event_receiver(false, &mut entity_container_guard);

        let mut entity_handler = entity_container_guard.handler_for_entity(&ids[2]).unwrap();
        entity_handler.remove_component::<Orientation>();
        entity_handler.remove_component::<Playable>();

        let mut entity_removed_counter: i32 = 0;
        let mut component_removed_counter: i32 = 0;
        let mut id2_found = false;
        while let Some(e) = entity_group_event.pop() {
            match e {
                EntityGroupEvent::EntityRemoved { entity_id } => {
                    entity_removed_counter += 1;

                    if entity_id == ids[2] {
                        id2_found = true;
                    }
                }
                EntityGroupEvent::ComponentRemoved { .. } => component_removed_counter += 1,
                _ => assert!(false),
            }
        }
        assert_eq!(entity_removed_counter, 1);
        assert_eq!(component_removed_counter, 4);
        assert!(id2_found);

        // contains
        assert!(entity_group.contains(&ids[0]));
        assert!(entity_group.contains(&ids[1]));
        assert!(!entity_group.contains(&ids[2]));
        assert!(!entity_group.contains(&ids[3]));
        assert!(!entity_group.contains(&ids[4]));
    }

    #[test]
    fn component_removed_event_when_deleting_an_entity() {
        let mut entity_container = EntityContainer::new();

        let entity_group = entity_container.lock().entity_group(component_type_list![
            Position,
            Velocity,
            Orientation,
        ]);

        let ids = add_test_entities(&mut entity_container);

        // contains
        assert!(entity_group.contains(&ids[0]));
        assert!(entity_group.contains(&ids[1]));
        assert!(entity_group.contains(&ids[2]));
        assert!(!entity_group.contains(&ids[3]));
        assert!(!entity_group.contains(&ids[4]));

        let mut entity_container_guard = entity_container.lock();

        let entity_group_event = entity_group.event_receiver(false, &mut entity_container_guard);

        // remove an entity
        entity_container_guard.remove_entity(&ids[2]);

        let mut entity_removed_counter: i32 = 0;
        let mut component_removed_counter: i32 = 0;
        let mut id2_found = false;
        while let Some(e) = entity_group_event.pop() {
            match e {
                EntityGroupEvent::EntityRemoved { entity_id } => {
                    entity_removed_counter += 1;

                    if entity_id == ids[2] {
                        id2_found = true;
                    }
                }
                EntityGroupEvent::ComponentRemoved { .. } => component_removed_counter += 1,
                _ => assert!(false),
            }
        }
        assert_eq!(entity_removed_counter, 1);
        assert_eq!(component_removed_counter, 4);
        assert!(id2_found);

        // contains
        assert!(entity_group.contains(&ids[0]));
        assert!(entity_group.contains(&ids[1]));
        assert!(!entity_group.contains(&ids[2]));
        assert!(!entity_group.contains(&ids[3]));
        assert!(!entity_group.contains(&ids[4]));

        // remove an entity
        assert!(entity_container_guard.remove_entity(&ids[0]));

        let mut entity_removed_counter: i32 = 0;
        let mut component_removed_counter: i32 = 0;
        let mut id0_found = false;
        while let Some(e) = entity_group_event.pop() {
            match e {
                EntityGroupEvent::EntityRemoved { entity_id } => {
                    entity_removed_counter += 1;

                    if entity_id == ids[0] {
                        id0_found = true;
                    }
                }
                EntityGroupEvent::ComponentRemoved { .. } => {
                    component_removed_counter += 1;
                }
                _ => assert!(false),
            }
        }
        assert_eq!(entity_removed_counter, 1);
        assert_eq!(component_removed_counter, 3);
        assert!(id0_found);

        // contains
        assert!(!entity_group.contains(&ids[0]));
        assert!(entity_group.contains(&ids[1]));
        assert!(!entity_group.contains(&ids[2]));
        assert!(!entity_group.contains(&ids[3]));
        assert!(!entity_group.contains(&ids[4]));

        // remove an entity
        entity_container_guard.remove_entity(&ids[1]);

        let mut entity_removed_counter: i32 = 0;
        let mut component_removed_counter: i32 = 0;
        let mut id1_found = false;
        while let Some(e) = entity_group_event.pop() {
            match e {
                EntityGroupEvent::EntityRemoved { entity_id } => {
                    entity_removed_counter += 1;

                    if entity_id == ids[1] {
                        id1_found = true;
                    }
                }
                EntityGroupEvent::ComponentRemoved { .. } => component_removed_counter += 1,
                _ => assert!(false),
            }
        }
        assert_eq!(entity_removed_counter, 1);
        assert_eq!(component_removed_counter, 4);
        assert!(id1_found);

        // contains
        assert!(!entity_group.contains(&ids[0]));
        assert!(!entity_group.contains(&ids[1]));
        assert!(!entity_group.contains(&ids[2]));
        assert!(!entity_group.contains(&ids[3]));
        assert!(!entity_group.contains(&ids[4]));
    }

    #[test]
    fn component_is_alive_after_entity_removal() {
        let mut entity_container = EntityContainer::new();

        let ids = add_test_entities(&mut entity_container);

        let mut entity_container_guard = entity_container.lock();

        // store a component id
        let entity_handler = entity_container_guard.handler_for_entity(&ids[1]).unwrap();
        let mut position_component_id = None;
        for component_id in entity_handler.iter_entity_component_ids() {
            if component_id.component_type_id == TypeId::of::<Position>() {
                position_component_id = Some(component_id.clone());
            }
        }

        // remove an entity
        entity_container_guard.remove_entity(&ids[1]);

        // component should still exist
        let position_component_id = position_component_id.unwrap();
        let position_component = entity_container_guard
            .component_storage()
            .get_component_ref::<Position>(&position_component_id);
        assert_eq!(*position_component.unwrap(), Position("pos1".to_string()));
    }

    #[test]
    fn component_changed_event() {
        let entity_container = EntityContainer::new();

        let entity_group = entity_container.lock().entity_group(component_type_list![
            Position,
            Velocity,
            Orientation,
        ]);

        let id = entity_container
            .entity_builder()
            .with_component(Position("pos0".to_string()))
            .with_component(Velocity("vel0".to_string()))
            .with_component(Orientation("ori0".to_string()))
            .with_component(Playable("pla0".to_string()))
            .build();

        // contains
        assert!(entity_group.contains(&id));

        let mut entity_container_guard = entity_container.lock();

        let entity_group_event = entity_group.event_receiver(false, &mut entity_container_guard);

        // change an interesting component
        let mut entity_handler = entity_container_guard.handler_for_entity(&id).unwrap();
        entity_handler
            .change_component(|velocity_mut: &mut Velocity| velocity_mut.0 = "vel1".to_string());

        // check events
        let mut component_changed_counter: i32 = 0;
        while let Some(e) = entity_group_event.pop() {
            match e {
                EntityGroupEvent::ComponentChanged { component_id, .. } => {
                    component_changed_counter += 1;

                    let component_opt = entity_container_guard
                        .component_storage()
                        .get_component_ref::<Velocity>(&component_id);
                    assert!(component_opt.is_some());
                    if let Some(component_ref) = component_opt {
                        assert_eq!(component_ref.0, "vel1");
                    }
                }
                _ => assert!(false),
            }
        }
        assert_eq!(component_changed_counter, 1);

        // change a non interesting component
        let mut entity_handler = entity_container_guard.handler_for_entity(&id).unwrap();
        entity_handler
            .change_component(|playable_mut: &mut Playable| playable_mut.0 = "pla1".to_string());

        // check events
        assert!(entity_group_event.pop().is_none());
    }

    #[test]
    fn change_component_in_handle_all_entities_in_entity_group() {
        let entity_container = EntityContainer::new();

        let entity_group = entity_container.lock().entity_group(component_type_list![
            Position,
            Velocity,
            Orientation,
        ]);

        entity_container
            .entity_builder()
            .with_component(Position("pos0".to_string()))
            .with_component(Velocity("vel0".to_string()))
            .with_component(Orientation("ori0".to_string()))
            .with_component(Playable("pla0".to_string()))
            .build();

        let mut entity_container_guard = entity_container.lock();

        for entity_id in entity_group.iter_entity_ids() {
            if let Some(mut entity_handler) = entity_container_guard.handler_for_entity(&entity_id)
            {
                entity_handler
                    .change_component(|pos: &mut Position| pos.0 = "changed pos".to_string());
            }
        }
    }

    #[test]
    fn remove_component_in_handle_all_entities_in_entity_group() {
        let entity_container = EntityContainer::new();

        let entity_group = entity_container.lock().entity_group(component_type_list![
            Position,
            Velocity,
            Orientation,
        ]);

        let id = entity_container
            .entity_builder()
            .with_component(Position("pos0".to_string()))
            .with_component(Velocity("vel0".to_string()))
            .with_component(Orientation("ori0".to_string()))
            .with_component(Playable("pla0".to_string()))
            .build();

        let mut entity_container_guard = entity_container.lock();

        for entity_id in entity_group.iter_entity_ids() {
            if let Some(mut entity_handler) = entity_container_guard.handler_for_entity(&entity_id)
            {
                entity_handler.remove_component::<Position>();
            }
        }

        assert!(!entity_group.contains(&id));
    }
}
