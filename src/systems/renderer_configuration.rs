use std::sync::Arc;

use muleengine::{
    bytifex_utils::{result_option_inspect::ResultInspector, sync::async_item::AsyncItem},
    renderer::{
        renderer_pipeline_step::RendererPipelineStep, renderer_system::RendererClient,
        RendererCameraHandler, RendererGroupHandler, RendererLayerHandler,
        RendererTransformHandler,
    },
    service_container::ServiceContainer,
};
use vek::{Mat4, Transform, Vec2};

pub struct RendererConfigurationData {
    skydome_camera_transform_handler: RendererTransformHandler,
    skydome_camera_handler: RendererCameraHandler,

    main_camera_transform_handler: RendererTransformHandler,
    main_camera_handler: RendererCameraHandler,

    skydome_renderer_layer_handler: RendererLayerHandler,
    main_renderer_layer_handler: RendererLayerHandler,

    skydome_renderer_group_handler: RendererGroupHandler,
    main_renderer_group_handler: RendererGroupHandler,
}

#[derive(Clone)]
pub struct RendererConfiguration {
    data: AsyncItem<RendererConfigurationData>,
}

impl RendererConfigurationData {
    pub async fn new(renderer_client: RendererClient) -> Self {
        let skydome_camera_transform_handler = renderer_client
            .create_transform(Transform::default())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();
        let skydome_camera_handler = renderer_client
            .create_camera(skydome_camera_transform_handler.clone())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();

        let main_camera_transform_handler = renderer_client
            .create_transform(Transform::default())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();
        let main_camera_handler = renderer_client
            .create_camera(main_camera_transform_handler.clone())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();

        let skydome_renderer_layer_handler = renderer_client
            .create_renderer_layer(skydome_camera_handler.clone())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();
        let skydome_renderer_group_handler = renderer_client
            .create_renderer_group()
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();
        renderer_client
            .add_renderer_group_to_layer(
                skydome_renderer_group_handler.clone(),
                skydome_renderer_layer_handler.clone(),
            )
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();

        let main_renderer_layer_handler = renderer_client
            .create_renderer_layer(main_camera_handler.clone())
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();
        let main_renderer_group_handler = renderer_client
            .create_renderer_group()
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();
        renderer_client
            .add_renderer_group_to_layer(
                main_renderer_group_handler.clone(),
                main_renderer_layer_handler.clone(),
            )
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();

        let fov_y_degrees = 45.0f32;
        let near_plane = 0.01;
        let far_plane = 1000.0;

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

                    compute_projection_matrix: Arc::new(move |window_width, window_height| {
                        Mat4::perspective_fov_rh_zo(
                            fov_y_degrees.to_radians(),
                            window_width as f32,
                            window_height as f32,
                            near_plane,
                            far_plane,
                        )
                    }),
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

                    compute_projection_matrix: Arc::new(move |window_width, window_height| {
                        Mat4::perspective_fov_rh_zo(
                            fov_y_degrees.to_radians(),
                            window_width as f32,
                            window_height as f32,
                            near_plane,
                            far_plane,
                        )
                    }),
                },
            ])
            .await
            .inspect_err(|e| log::error!("{e:?}"))
            .unwrap()
            .unwrap();

        Self {
            skydome_camera_transform_handler,
            skydome_camera_handler,

            main_camera_transform_handler,
            main_camera_handler,

            skydome_renderer_layer_handler,
            main_renderer_layer_handler,

            skydome_renderer_group_handler,
            main_renderer_group_handler,
        }
    }
}

impl RendererConfiguration {
    pub fn new(service_container: ServiceContainer) -> Self {
        let data = AsyncItem::new();

        {
            let data = data.clone();
            tokio::spawn(async move {
                let renderer_client = service_container
                    .get_service::<RendererClient>()
                    .inspect_err(|e| log::error!("{e:?}"))
                    .unwrap()
                    .as_ref()
                    .clone();

                data.set(RendererConfigurationData::new(renderer_client).await)
                    .await;
            });
        }

        Self { data }
    }

    pub async fn skydome_camera_transform_handler(&self) -> RendererTransformHandler {
        self.data
            .read()
            .await
            .skydome_camera_transform_handler
            .clone()
    }

    pub async fn skydome_camera_handler(&self) -> RendererCameraHandler {
        self.data.read().await.skydome_camera_handler.clone()
    }

    pub async fn main_camera_transform_handler(&self) -> RendererTransformHandler {
        self.data.read().await.main_camera_transform_handler.clone()
    }

    pub async fn main_camera_handler(&self) -> RendererCameraHandler {
        self.data.read().await.main_camera_handler.clone()
    }

    pub async fn skydome_renderer_layer_handler(&self) -> RendererLayerHandler {
        self.data
            .read()
            .await
            .skydome_renderer_layer_handler
            .clone()
    }

    pub async fn main_renderer_layer_handler(&self) -> RendererLayerHandler {
        self.data.read().await.main_renderer_layer_handler.clone()
    }

    pub async fn skydome_renderer_group_handler(&self) -> RendererGroupHandler {
        self.data
            .read()
            .await
            .skydome_renderer_group_handler
            .clone()
    }

    pub async fn main_renderer_group_handler(&self) -> RendererGroupHandler {
        self.data.read().await.main_renderer_group_handler.clone()
    }
}
