use std::{any::type_name, sync::Arc};

use parking_lot::RwLock;

use super::containers::multi_type_dict::{MultiTypeDict, MultiTypeDictInsertResult};

#[derive(Debug, Clone)]
pub struct ServiceMissingError {
    service_type_name: String,
}

#[derive(Clone)]
pub struct ServiceContainer {
    service_dict: Arc<RwLock<MultiTypeDict>>,
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
            service_dict: Arc::new(RwLock::new(MultiTypeDict::new())),
        }
    }

    pub fn insert<ServiceType: 'static>(
        &mut self,
        item: ServiceType,
    ) -> MultiTypeDictInsertResult<RwLock<ServiceType>> {
        self.service_dict.write().insert(RwLock::new(item))
    }

    pub fn get_service<ServiceType: 'static>(
        &self,
    ) -> Result<Arc<RwLock<ServiceType>>, ServiceMissingError> {
        self.service_dict
            .read()
            .get_item_ref::<RwLock<ServiceType>>()
            .map(|service| service.as_arc_ref().clone())
            .ok_or_else(|| ServiceMissingError::new::<ServiceType>())
    }

    pub fn get_or_insert_service<ServiceType: 'static>(
        &self,
        service_constructor: impl FnOnce() -> ServiceType,
    ) -> Arc<RwLock<ServiceType>> {
        self.service_dict
            .write()
            .get_or_insert_item_ref::<RwLock<ServiceType>>(|| RwLock::new(service_constructor()))
            .as_arc_ref()
            .clone()
    }
}
