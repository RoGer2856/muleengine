mod component;
mod component_id;
mod component_storage;
mod component_type_list;
mod entity;
mod entity_builder;
mod entity_container;
mod entity_group;
mod entity_group_event;
mod entity_handler;
mod entity_modified_event;
mod multi_type_component_storage;

pub use component_id::*;
pub use entity_builder::*;
pub use entity_container::*;
pub use entity_group::*;
pub use entity_group_event::*;
pub use entity_handler::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_remove_entity() {
        let entity_container = EntityContainer::new();

        let id = entity_container.entity_builder().build();

        let mut entity_container_guard = entity_container.lock();
        assert!(entity_container_guard.remove_entity(&id));
        assert!(!entity_container_guard.remove_entity(&id));
    }

    #[test]
    fn add_component() {
        let entity_container = EntityContainer::new();

        let id = entity_container.entity_builder().build();

        let mut entity_container_guard = entity_container.lock();

        // add
        let mut handler = entity_container_guard.handler_for_entity(&id).unwrap();
        handler.add_component("initial text".to_string());

        // read
        let handler = entity_container_guard.handler_for_entity(&id).unwrap();
        let component = handler.get_component_ref::<String>().unwrap().clone();
        assert_eq!(component, "initial text");
    }

    #[test]
    fn add_component_with_builder() {
        let entity_container = EntityContainer::new();

        let id = entity_container
            .entity_builder()
            .with_component("initial text".to_string())
            .build();

        let mut entity_container_guard = entity_container.lock();

        // read
        let handler = entity_container_guard.handler_for_entity(&id).unwrap();
        let component = handler.get_component_ref::<String>().unwrap().clone();
        assert_eq!(component, "initial text");
    }

    #[test]
    fn modify_read_remove_component() {
        let entity_container = EntityContainer::new();

        let id = entity_container
            .entity_builder()
            .with_component("initial text".to_string())
            .build();

        let mut entity_container_guard = entity_container.lock();

        // read
        let handler = entity_container_guard.handler_for_entity(&id).unwrap();
        let component = handler.get_component_ref::<String>().unwrap().clone();
        assert_eq!(component, "initial text");

        // modify
        let mut handler = entity_container_guard.handler_for_entity(&id).unwrap();
        handler.change_component(|component_mut| {
            *component_mut = "modified text".to_string();
        });

        // read
        let handler = entity_container_guard.handler_for_entity(&id).unwrap();
        let component = handler.get_component_ref::<String>().unwrap().clone();
        assert_eq!(component, "modified text");

        // remove
        let mut handler = entity_container_guard.handler_for_entity(&id).unwrap();
        handler.remove_component::<String>();

        // read
        let handler = entity_container_guard.handler_for_entity(&id).unwrap();
        assert!(handler.get_component_ref::<String>().is_none());
    }

    #[test]
    fn iterate_over_every_entities() {
        let entity_container = EntityContainer::new();

        let number_of_entities = 20;
        (0..number_of_entities).for_each(|index| {
            entity_container
                .entity_builder()
                .with_component(format!("component {}", index))
                .build();
        });

        let mut entity_container_guard = entity_container.lock();

        let components = entity_container_guard
            .iter()
            .map(|entity_handler| {
                entity_handler
                    .get_component_ref::<String>()
                    .unwrap()
                    .clone()
            })
            .collect::<Vec<String>>();

        assert_eq!(number_of_entities, components.len());
        for index in 0..number_of_entities {
            assert!(components.contains(&format!("component {}", index)));
        }
    }
}
