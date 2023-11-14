use std::sync::Arc;

use entity_component::EntityContainer;
use muleengine::{
    asset_container::AssetContainer,
    bytifex_utils::{
        result_option_inspect::ResultInspector, sync::app_loop_state::AppLoopStateWatcher,
    },
    renderer::renderer_system::renderer_decoupler,
    service_container::ServiceContainer,
};

use crate::{
    physics::Rapier3dPhysicsEngineService, systems::renderer_configuration::RendererConfiguration,
};

pub struct EssentialServices {
    pub app_loop_state_watcher: AppLoopStateWatcher,

    pub service_container: ServiceContainer,
    pub asset_container: AssetContainer,

    pub renderer_configuration: Arc<RendererConfiguration>,
    pub renderer_client: renderer_decoupler::Client,

    pub physics_engine: Arc<Rapier3dPhysicsEngineService>,

    pub entity_container: EntityContainer,
}

impl EssentialServices {
    pub fn new(service_container: ServiceContainer) -> Self {
        Self {
            app_loop_state_watcher: service_container
                .get_service::<AppLoopStateWatcher>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
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
                .unwrap()
                .as_ref()
                .clone(),
            physics_engine: service_container
                .get_service::<Rapier3dPhysicsEngineService>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .clone(),
        }
    }
}
