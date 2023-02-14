use std::{any::TypeId, collections::HashMap};

use parking_lot::RwLock;

use crate::{containers::multi_type_dict::MultiTypeDictInsertResult, prelude::ArcRwLock};

use super::containers::multi_type_dict::{MultiTypeDict, MultiTypeDictItem};

pub trait System: 'static {
    fn tick(&mut self, delta_time_in_secs: f32);
}

pub struct SystemContainer {
    systems_multi_type_dict: MultiTypeDict,
    systems_by_type_id: HashMap<TypeId, ArcRwLock<dyn System>>,
}

impl Default for SystemContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemContainer {
    pub fn new() -> Self {
        Self {
            systems_multi_type_dict: MultiTypeDict::new(),
            systems_by_type_id: HashMap::new(),
        }
    }

    pub fn tick(&mut self, delta_time_in_secs: f32) {
        for system in self.systems_by_type_id.iter() {
            system.1.write().tick(delta_time_in_secs);
        }
    }

    pub fn add_system<SystemType: System>(
        &mut self,
        system: SystemType,
    ) -> MultiTypeDictInsertResult<RwLock<SystemType>> {
        let type_id = TypeId::of::<SystemType>();
        if self.systems_by_type_id.contains_key(&type_id) {
            self.systems_by_type_id.remove(&type_id);
        }
        let result = self.systems_multi_type_dict.insert(RwLock::new(system));
        self.systems_by_type_id
            .insert(type_id, result.new_item.as_arc_ref().clone());
        result
    }

    pub fn get_system<SystemType: System>(&self) -> Option<MultiTypeDictItem<RwLock<SystemType>>> {
        self.systems_multi_type_dict.get_item_ref()
    }

    pub fn get_or_insert_system<SystemType: System>(
        &mut self,
        system_constructor: impl FnOnce() -> SystemType,
    ) -> MultiTypeDictItem<RwLock<SystemType>> {
        let result = self
            .systems_multi_type_dict
            .get_or_insert_item_ref(|| RwLock::new(system_constructor()));

        let type_id = TypeId::of::<SystemType>();
        self.systems_by_type_id
            .entry(type_id)
            .or_insert_with(|| result.as_arc_ref().clone());

        result
    }

    pub fn remove<SystemType: System>(&mut self) {
        let type_id = TypeId::of::<SystemType>();
        self.systems_by_type_id.remove(&type_id);
        self.systems_multi_type_dict.remove::<RwLock<SystemType>>();
    }
}
