use std::{
    any::{type_name, TypeId},
    collections::BTreeMap,
    sync::Arc,
};

use parking_lot::{Condvar, Mutex, RwLock};

use crate::prelude::ArcRwLock;

use super::containers::sendable_multi_type_dict::{
    SendableMultiTypeDict, SendableMultiTypeDictInsertResult,
};

#[derive(Debug, Clone)]
pub struct ServiceMissingError {
    service_type_name: String,
}

type ServiceTypeLock = Arc<(Mutex<bool>, Condvar)>;

pub trait Service: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Service for T {}

#[derive(Clone)]
pub struct ServiceContainer {
    service_dict: ArcRwLock<SendableMultiTypeDict>,
    service_type_locks: Arc<Mutex<BTreeMap<TypeId, ServiceTypeLock>>>,
}

pub struct ServiceTypeGuard {
    service_type_locks: Arc<Mutex<BTreeMap<TypeId, ServiceTypeLock>>>,
    type_id: TypeId,
    lock: Arc<(Mutex<bool>, Condvar)>,
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

    pub fn insert<ServiceType: Service>(
        &mut self,
        item: ServiceType,
    ) -> SendableMultiTypeDictInsertResult<RwLock<ServiceType>> {
        self.service_dict.write().insert(RwLock::new(item))
    }

    pub fn get_service<ServiceType: Service>(
        &self,
    ) -> Result<ArcRwLock<ServiceType>, ServiceMissingError> {
        self.service_dict
            .read()
            .get_item_ref::<RwLock<ServiceType>>()
            .map(|service| service.as_arc_ref().clone())
            .ok_or_else(ServiceMissingError::new::<ServiceType>)
    }

    pub fn get_or_insert_service<ServiceType: Service>(
        &self,
        service_constructor: impl FnOnce() -> ServiceType,
    ) -> ArcRwLock<ServiceType> {
        let _service_type_guard = self.lock_service_type::<ServiceType>();

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

        service
    }

    fn lock_service_type<ServiceType: Service>(&self) -> ServiceTypeGuard {
        let type_id = TypeId::of::<ServiceType>();
        let mut service_type_locks = self.service_type_locks.lock();
        let entry = service_type_locks
            .entry(type_id)
            .or_insert_with(|| Arc::new((Mutex::new(false), Condvar::new())));

        let entry = entry.clone();

        drop(service_type_locks);

        let mut service_type_locked = entry.0.lock();
        while *service_type_locked {
            entry.1.wait(&mut service_type_locked);
        }
        *service_type_locked = true;

        ServiceTypeGuard {
            service_type_locks: self.service_type_locks.clone(),
            type_id,
            lock: entry.clone(),
        }
    }
}

impl Drop for ServiceTypeGuard {
    fn drop(&mut self) {
        let mut service_type_locks = self.service_type_locks.lock();
        // check that only service_type_locks and self contains this lock
        if Arc::strong_count(&self.lock) == 2 {
            // nobody tries to lock the service type
            service_type_locks.remove(&self.type_id);
        } else {
            // somebody tries to lock the service type
            let mut service_type_locked = self.lock.0.lock();
            *service_type_locked = false;
            self.lock.1.notify_one();
        }
    }
}
