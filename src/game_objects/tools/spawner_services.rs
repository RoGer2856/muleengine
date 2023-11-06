use std::sync::Arc;

use entity_component::EntityContainer;
use muleengine::{
    asset_container::AssetContainer, bytifex_utils::result_option_inspect::ResultInspector,
    renderer::renderer_system::renderer_decoupler, service_container::ServiceContainer,
};

use crate::{
    physics::Rapier3dPhysicsEngineService, systems::renderer_configuration::RendererConfiguration,
};

pub struct Spawner {
    pub service_container: ServiceContainer,
    pub asset_container: AssetContainer,

    pub renderer_configuration: Arc<RendererConfiguration>,
    pub renderer_client: renderer_decoupler::Client,

    pub physics_engine: Arc<Rapier3dPhysicsEngineService>,

    pub entity_container: Arc<EntityContainer>,
}

impl Spawner {
    pub fn new(service_container: ServiceContainer) -> Self {
        Self {
            service_container: service_container.clone(),
            renderer_configuration: service_container
                .get_service::<RendererConfiguration>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap(),
            renderer_client: service_container
                .get_service::<renderer_decoupler::Client>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            asset_container: service_container
                .get_service::<AssetContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            entity_container: service_container
                .get_service::<EntityContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap(),
            physics_engine: service_container
                .get_service::<Rapier3dPhysicsEngineService>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .clone(),
        }
    }
}
