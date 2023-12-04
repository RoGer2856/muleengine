use std::{any::TypeId, collections::HashMap};

use parking_lot::RwLock;

use bytifex_utils::{
    containers::sendable_multi_type_dict::{
        SendableMultiTypeDict, SendableMultiTypeDictInsertResult, SendableMultiTypeDictItem,
    },
    sync::types::ArcRwLock,
};

use crate::system_container::System;

pub trait SendableSystem: System + Send + Sync + 'static {}

impl<T> SendableSystem for T where T: System + Send + Sync + 'static {}

pub type SendableSystemService = RwLock<SendableSystemContainer>;

pub struct SendableSystemContainer {
    systems_multi_type_dict: SendableMultiTypeDict,
    systems_by_type_id: HashMap<TypeId, ArcRwLock<dyn SendableSystem>>,
}

impl Default for SendableSystemContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl SendableSystemContainer {
    pub fn new() -> Self {
        Self {
            systems_multi_type_dict: SendableMultiTypeDict::new(),
            systems_by_type_id: HashMap::new(),
        }
    }

    pub fn tick(&mut self, delta_time_in_secs: f32) {
        for system in self.systems_by_type_id.iter() {
            system.1.write().tick(delta_time_in_secs);
        }
    }

    pub fn add_system<SystemType: SendableSystem>(
        &mut self,
        system: SystemType,
    ) -> SendableMultiTypeDictInsertResult<RwLock<SystemType>> {
        let type_id = TypeId::of::<SystemType>();
        if self.systems_by_type_id.contains_key(&type_id) {
            self.systems_by_type_id.remove(&type_id);
        }
        let result = self.systems_multi_type_dict.insert(RwLock::new(system));
        self.systems_by_type_id
            .insert(type_id, result.new_item.as_arc_ref().clone());
        result
    }

    pub fn get_system<SystemType: SendableSystem>(
        &self,
    ) -> Option<SendableMultiTypeDictItem<RwLock<SystemType>>> {
        self.systems_multi_type_dict.get_item_ref()
    }

    pub fn get_or_insert_system<SystemType: SendableSystem>(
        &mut self,
        system_constructor: impl FnOnce() -> SystemType,
    ) -> SendableMultiTypeDictItem<RwLock<SystemType>> {
        let result = self
            .systems_multi_type_dict
            .get_or_insert_item_ref(|| RwLock::new(system_constructor()));

        let type_id = TypeId::of::<SystemType>();
        self.systems_by_type_id
            .entry(type_id)
            .or_insert_with(|| result.as_arc_ref().clone());

        result
    }

    pub fn remove<SystemType: SendableSystem>(&mut self) {
        let type_id = TypeId::of::<SystemType>();
        self.systems_by_type_id.remove(&type_id);
        self.systems_multi_type_dict.remove::<RwLock<SystemType>>();
    }
}
