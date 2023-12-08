use std::{any::TypeId, collections::HashMap};

use method_taskifier::{
    method_taskifier_impl,
    task_channel::{task_channel, TaskReceiver},
    AllWorkersDroppedError,
};
use parking_lot::RwLock;

use bytifex_utils::{
    containers::multi_type_dict::{MultiTypeDict, MultiTypeDictInsertResult, MultiTypeDictItem},
    sync::types::ArcRwLock,
};

pub trait System: 'static {
    fn tick(&mut self, delta_time_in_secs: f32);
}

pub struct SystemContainer {
    systems_multi_type_dict: MultiTypeDict,
    systems_by_type_id: HashMap<TypeId, ArcRwLock<dyn System>>,
    task_receiver: TaskReceiver<client::ChanneledTask>,
}

pub use client::Client as SystemContainerClient;

#[method_taskifier_impl(module_name = client)]
impl SystemContainer {
    pub fn new_with_client() -> (Self, SystemContainerClient) {
        let (task_sender, task_receiver) = task_channel();

        (
            Self {
                systems_multi_type_dict: MultiTypeDict::new(),
                systems_by_type_id: HashMap::new(),
                task_receiver,
            },
            SystemContainerClient::new(task_sender),
        )
    }

    pub fn tick(&mut self, delta_time_in_secs: f32) {
        while let Ok(task) = self.task_receiver.try_pop() {
            self.execute_channeled_task(task);
        }

        for system in self.systems_by_type_id.iter() {
            system.1.write().tick(delta_time_in_secs);
        }
    }

    #[method_taskifier_client_fn]
    pub fn execute_closure_async(
        &self,
        closure: impl FnOnce(&mut SystemContainer) + Send + 'static,
    ) {
        drop(self.execute_boxed_closure_async(Box::new(closure)));
    }

    #[method_taskifier_client_fn]
    pub async fn execute_closure(
        &self,
        closure: impl FnOnce(&mut SystemContainer) + Send + 'static,
    ) -> Result<(), AllWorkersDroppedError> {
        self.execute_boxed_closure_async(Box::new(closure)).await
    }

    #[method_taskifier_worker_fn]
    pub fn execute_boxed_closure_async(
        &mut self,
        closure: Box<dyn FnOnce(&mut SystemContainer) + Send>,
    ) {
        closure(self)
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
