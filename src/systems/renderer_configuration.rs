use muleengine::{
    prelude::ResultInspector,
    renderer::{
        renderer_client::RendererClient, renderer_pipeline_step::RendererPipelineStep,
        CameraHandler, RendererGroupHandler, RendererLayerHandler, TransformHandler,
    },
    service_container::ServiceContainer,
};
use vek::{Transform, Vec2};

pub struct RendererConfiguration {
    skydome_camera_transform_handler: TransformHandler,
    skydome_camera_handler: CameraHandler,

    main_camera_transform_handler: TransformHandler,
    main_camera_handler: CameraHandler,

    skydome_renderer_layer_handler: RendererLayerHandler,
    main_renderer_layer_handler: RendererLayerHandler,

    skydome_renderer_group_handler: RendererGroupHandler,
    main_renderer_group_handler: RendererGroupHandler,
}

impl RendererConfiguration {
    pub async fn new(service_container: ServiceContainer) -> Self {
        let renderer_client = service_container
            .get_service::<RendererClient>()
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .read()
            .clone();

        let skydome_camera_transform_handler = renderer_client
            .create_transform(Transform::default())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();
        let skydome_camera_handler = renderer_client
            .create_camera(skydome_camera_transform_handler.clone())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let main_camera_transform_handler = renderer_client
            .create_transform(Transform::default())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();
        let main_camera_handler = renderer_client
            .create_camera(main_camera_transform_handler.clone())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let skydome_renderer_layer_handler = renderer_client
            .create_renderer_layer(skydome_camera_handler.clone())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();
        let main_renderer_layer_handler = renderer_client
            .create_renderer_layer(main_camera_handler.clone())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let skydome_renderer_group_handler = renderer_client
            .create_renderer_group()
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();
        renderer_client
            .add_renderer_group_to_layer(
                skydome_renderer_group_handler.clone(),
                skydome_renderer_layer_handler.clone(),
            )
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        let main_renderer_group_handler = renderer_client
            .create_renderer_group()
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();
        renderer_client
            .add_renderer_group_to_layer(
                main_renderer_group_handler.clone(),
                main_renderer_layer_handler.clone(),
            )
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        renderer_client
            .set_renderer_pipeline(vec![
                RendererPipelineStep::Clear {
                    depth: true,
                    color: true,

                    viewport_start_ndc: Vec2::broadcast(0.0),
                    viewport_end_ndc: Vec2::broadcast(1.0),
                },
                RendererPipelineStep::Draw {
                    renderer_layer_handler: skydome_renderer_layer_handler.clone(),

                    viewport_start_ndc: Vec2::broadcast(0.0),
                    viewport_end_ndc: Vec2::broadcast(1.0),
                },
                RendererPipelineStep::Clear {
                    viewport_start_ndc: Vec2::broadcast(0.0),
                    viewport_end_ndc: Vec2::broadcast(1.0),
                    depth: true,
                    color: false,
                },
                RendererPipelineStep::Draw {
                    renderer_layer_handler: main_renderer_layer_handler.clone(),

                    viewport_start_ndc: Vec2::broadcast(0.0),
                    viewport_end_ndc: Vec2::broadcast(1.0),
                },
            ])
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap();

        Self {
            skydome_camera_transform_handler,
            skydome_camera_handler,

            main_camera_handler,
            main_camera_transform_handler,

            skydome_renderer_layer_handler,
            main_renderer_layer_handler,

            skydome_renderer_group_handler,
            main_renderer_group_handler,
        }
    }

    pub fn skydome_camera_transform_handler(&self) -> TransformHandler {
        self.skydome_camera_transform_handler.clone()
    }

    pub fn skydome_camera_handler(&self) -> CameraHandler {
        self.skydome_camera_handler.clone()
    }

    pub fn main_camera_transform_handler(&self) -> TransformHandler {
        self.main_camera_transform_handler.clone()
    }

    pub fn main_camera_handler(&self) -> CameraHandler {
        self.main_camera_handler.clone()
    }

    pub fn skydome_renderer_layer_handler(&self) -> RendererLayerHandler {
        self.skydome_renderer_layer_handler.clone()
    }

    pub fn main_renderer_layer_handler(&self) -> RendererLayerHandler {
        self.main_renderer_layer_handler.clone()
    }

    pub fn skydome_renderer_group_handler(&self) -> RendererGroupHandler {
        self.skydome_renderer_group_handler.clone()
    }

    pub fn main_renderer_group_handler(&self) -> RendererGroupHandler {
        self.main_renderer_group_handler.clone()
    }
}
