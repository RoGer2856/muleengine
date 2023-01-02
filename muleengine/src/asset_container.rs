use crate::{
    asset_reader::AssetReader,
    image_container::ImageContainer,
    prelude::{ArcRwLock, ResultInspector},
    scene_container::SceneContainer,
    service_container::ServiceContainer,
};

#[derive(Clone)]
pub struct AssetContainer {
    asset_reader: ArcRwLock<AssetReader>,

    image_container: ArcRwLock<ImageContainer>,
    scene_container: ArcRwLock<SceneContainer>,
}

impl AssetContainer {
    pub fn new(service_container: &ServiceContainer) -> Self {
        Self {
            asset_reader: service_container
                .get_service::<AssetReader>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap(),
            scene_container: service_container
                .get_service::<SceneContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap(),
            image_container: service_container
                .get_service::<ImageContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap(),
        }
    }

    pub fn asset_reader(&self) -> &ArcRwLock<AssetReader> {
        &self.asset_reader
    }

    pub fn image_container(&self) -> &ArcRwLock<ImageContainer> {
        &self.image_container
    }

    pub fn scene_container(&self) -> &ArcRwLock<SceneContainer> {
        &self.scene_container
    }
}
