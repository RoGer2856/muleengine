use parking_lot::RwLock;

use crate::{containers::multi_type_dict::MultiTypeDictInsertResult, prelude::ArcRwLock};

use super::containers::multi_type_dict::{MultiTypeDict, MultiTypeDictItem};

pub trait System: 'static {
    fn tick(&mut self, delta_time_in_secs: f32);
}

pub struct SystemContainer {
    systems_by_type_id: MultiTypeDict,
    systems: Vec<ArcRwLock<dyn System>>,
}

impl Default for SystemContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemContainer {
    pub fn new() -> Self {
        Self {
            systems_by_type_id: MultiTypeDict::new(),
            systems: Vec::new(),
        }
    }

    pub fn tick(&mut self, delta_time_in_secs: f32) {
        for system in self.systems.iter() {
            system.write().tick(delta_time_in_secs);
        }
    }

    pub fn add_system<SystemType: System>(
        &mut self,
        system: SystemType,
    ) -> MultiTypeDictInsertResult<RwLock<SystemType>> {
        let result = self.systems_by_type_id.insert(RwLock::new(system));
        self.systems.push(result.new_item.as_arc_ref().clone());
        result
    }

    pub fn get_system<SystemType: 'static>(&self) -> Option<MultiTypeDictItem<RwLock<SystemType>>> {
        self.systems_by_type_id.get_item_ref()
    }

    pub fn get_or_insert_system<SystemType: 'static>(
        &mut self,
        system_constructor: impl FnOnce() -> SystemType,
    ) -> MultiTypeDictItem<RwLock<SystemType>> {
        self.systems_by_type_id
            .get_or_insert_item_ref(|| RwLock::new(system_constructor()))
    }
}
