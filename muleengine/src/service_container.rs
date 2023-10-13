use std::{any::type_name, sync::Arc};

use crate::prelude::{arc_rw_lock_new, ArcRwLock};
use tokio::sync::Notify;

use super::containers::sendable_multi_type_dict::{
    SendableMultiTypeDict, SendableMultiTypeDictInsertResult,
};

#[derive(Debug, Clone)]
pub struct ServiceMissingError {
    service_type_name: String,
}

pub trait Service: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Service for T {}

#[derive(Clone)]
pub struct ServiceContainer {
    service_dict: ArcRwLock<SendableMultiTypeDict>,
    // todo!("use an async condvar")
    service_dict_notify: Arc<Notify>,
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
            service_dict: arc_rw_lock_new(SendableMultiTypeDict::new()),
            service_dict_notify: Arc::new(Notify::new()),
        }
    }

    pub fn insert<ServiceType: Service>(
        &self,
        item: ServiceType,
    ) -> SendableMultiTypeDictInsertResult<ServiceType> {
        let ret = self.service_dict.write().insert(item);
        self.service_dict_notify.notify_waiters();
        ret
    }

    pub async fn wait_for_service<ServiceType: Service>(&self) -> Arc<ServiceType> {
        #![allow(clippy::await_holding_lock)]

        loop {
            let service_dict_guard = self.service_dict.read();

            if let Some(service) = service_dict_guard
                .get_item_ref::<ServiceType>()
                .map(|service| service.as_arc_ref().clone())
            {
                break service;
            }

            let service_dict_notify_task = self.service_dict_notify.notified();
            drop(service_dict_guard);
            service_dict_notify_task.await;
        }
    }

    pub fn get_service<ServiceType: Service>(
        &self,
    ) -> Result<Arc<ServiceType>, ServiceMissingError> {
        self.service_dict
            .read()
            .get_item_ref::<ServiceType>()
            .map(|service| service.as_arc_ref().clone())
            .ok_or_else(ServiceMissingError::new::<ServiceType>)
    }

    pub fn get_or_insert_service<ServiceType: Service>(
        &self,
        service_constructor: impl FnOnce() -> ServiceType,
    ) -> Arc<ServiceType> {
        let service = self.service_dict.read().get_item_ref::<ServiceType>();

        let service = if let Some(service) = service {
            service.as_arc_ref().clone()
        } else {
            let service = service_constructor();
            let ret = self
                .service_dict
                .write()
                .get_or_insert_item_ref(|| service)
                .as_arc_ref()
                .clone();
            self.service_dict_notify.notify_waiters();
            ret
        };

        service
    }

    pub fn remove<ServiceType: Service>(&self) -> Option<Arc<ServiceType>> {
        self.service_dict.write().remove::<ServiceType>()
    }
}
