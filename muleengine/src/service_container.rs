use std::{
    any::{type_name, TypeId},
    collections::BTreeMap,
    sync::Arc,
};

use parking_lot::{Mutex, RwLock};

use crate::prelude::ArcRwLock;

use super::containers::sendable_multi_type_dict::{
    SendableMultiTypeDict, SendableMultiTypeDictInsertResult,
};

#[derive(Debug, Clone)]
pub struct ServiceMissingError {
    service_type_name: String,
}

#[derive(Clone)]
pub struct ServiceContainer {
    service_dict: ArcRwLock<SendableMultiTypeDict>,
    service_type_locks: Arc<Mutex<BTreeMap<TypeId, Arc<Mutex<()>>>>>,
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceMissingError {
    pub fn new<ServiceType>() -> Self {
        Self {
            service_type_name: type_name::<ServiceType>().to_string(),
        }
    }

    pub fn service_type_name(&self) -> &String {
        &self.service_type_name
    }
}

impl ServiceContainer {
    pub fn new() -> Self {
        Self {
            service_dict: Arc::new(RwLock::new(SendableMultiTypeDict::new())),
            service_type_locks: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn insert<ServiceType: Send + Sync + 'static>(
        &mut self,
        item: ServiceType,
    ) -> SendableMultiTypeDictInsertResult<RwLock<ServiceType>> {
        self.service_dict.write().insert(RwLock::new(item))
    }

    pub fn get_service<ServiceType: 'static>(
        &self,
    ) -> Result<ArcRwLock<ServiceType>, ServiceMissingError> {
        self.service_dict
            .read()
            .get_item_ref::<RwLock<ServiceType>>()
            .map(|service| service.as_arc_ref().clone())
            .ok_or_else(ServiceMissingError::new::<ServiceType>)
    }

    pub fn get_or_insert_service<ServiceType: Send + Sync + 'static>(
        &self,
        service_constructor: impl FnOnce() -> ServiceType,
    ) -> ArcRwLock<ServiceType> {
        self.lock_service_type::<ServiceType>();

        let service = self
            .service_dict
            .read()
            .get_item_ref::<RwLock<ServiceType>>();

        let service = if let Some(service) = service {
            service.as_arc_ref().clone()
        } else {
            let service = RwLock::new(service_constructor());
            self.service_dict
                .write()
                .get_or_insert_item_ref::<RwLock<ServiceType>>(|| service)
                .as_arc_ref()
                .clone()
        };

        self.unlock_service_type::<ServiceType>();

        service
    }

    fn lock_service_type<ServiceType: 'static>(&self) {
        let type_id = TypeId::of::<ServiceType>();
        let mut service_type_locks = self.service_type_locks.lock();
        let entry = service_type_locks
            .entry(type_id)
            .or_insert_with(|| Arc::new(Mutex::new(())));
        let _ = entry.clone().lock();
    }

    fn unlock_service_type<ServiceType: 'static>(&self) {
        let type_id = TypeId::of::<ServiceType>();
        self.service_type_locks.lock().remove(&type_id);
    }
}
