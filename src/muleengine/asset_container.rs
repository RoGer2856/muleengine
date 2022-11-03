use std::sync::Arc;

use crate::muleengine::{
    asset_reader::AssetReader, image_container::ImageContainer, scene_container::SceneContainer,
    service_container::ServiceContainer,
};
use parking_lot::RwLock;

#[derive(Clone)]
pub struct AssetContainer {
    asset_reader: Arc<RwLock<AssetReader>>,

    image_container: Arc<RwLock<ImageContainer>>,
    scene_container: Arc<RwLock<SceneContainer>>,
}

impl AssetContainer {
    pub fn new(service_container: &ServiceContainer) -> Self {
        Self {
            asset_reader: service_container.get_service::<AssetReader>().unwrap(),
            scene_container: service_container.get_service::<SceneContainer>().unwrap(),
            image_container: service_container.get_service::<ImageContainer>().unwrap(),
        }
    }

    pub fn asset_reader(&self) -> &Arc<RwLock<AssetReader>> {
        &self.asset_reader
    }

    pub fn image_container(&self) -> &Arc<RwLock<ImageContainer>> {
        &self.image_container
    }

    pub fn scene_container(&self) -> &Arc<RwLock<SceneContainer>> {
        &self.scene_container
    }
}
