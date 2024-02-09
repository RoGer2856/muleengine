use std::sync::Arc;

use entity_component::EntityContainer;
use muleengine::{
    application_runner::{ApplicationContext, ClosureTaskSender},
    asset_container::AssetContainer,
    bytifex_utils::sync::{app_loop_state::AppLoopStateWatcher, types::ArcRwLock},
    font::HackFontContainer,
    renderer::renderer_system::RendererClient,
    service_container::ServiceContainer,
    system_container::SystemContainerClient,
    window_context::EventReceiver,
};
use parking_lot::RwLock;

use crate::{
    physics::Rapier3dPhysicsEngineService, systems::renderer_configuration::RendererConfiguration,
};

pub struct EssentialServices {
    pub event_receiver: EventReceiver,
    pub app_loop_state_watcher: AppLoopStateWatcher,

    pub system_container_client: SystemContainerClient,
    pub service_container: ServiceContainer,
    pub closure_task_sender: ClosureTaskSender,
    pub asset_container: AssetContainer,

    pub renderer_configuration: Arc<RendererConfiguration>,
    pub renderer_client: RendererClient,

    pub physics_engine: Arc<Rapier3dPhysicsEngineService>,

    pub entity_container: EntityContainer,

    pub hack_font: ArcRwLock<HackFontContainer>,
}

impl EssentialServices {
    pub fn new(app_context: &ApplicationContext) -> Self {
        Self {
            event_receiver: app_context
                .service_container_ref()
                .get_service::<EventReceiver>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            app_loop_state_watcher: app_context
                .service_container_ref()
                .get_service::<AppLoopStateWatcher>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            system_container_client: app_context.system_container_client().clone(),
            service_container: app_context.service_container_ref().clone(),
            closure_task_sender: app_context
                .service_container_ref()
                .get_service::<ClosureTaskSender>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            renderer_configuration: app_context
                .service_container_ref()
                .get_service::<RendererConfiguration>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap(),
            renderer_client: app_context
                .service_container_ref()
                .get_service::<RendererClient>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            asset_container: app_context
                .service_container_ref()
                .get_service::<AssetContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            entity_container: app_context
                .service_container_ref()
                .get_service::<EntityContainer>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .as_ref()
                .clone(),
            physics_engine: app_context
                .service_container_ref()
                .get_service::<Rapier3dPhysicsEngineService>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .clone(),
            hack_font: app_context
                .service_container_ref()
                .get_service::<RwLock<HackFontContainer>>()
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .clone(),
        }
    }
}
