use std::sync::Arc;

use tokio::sync::oneshot;
use vek::Transform;

use crate::{
    containers::object_pool::ObjectPoolIndex,
    mesh::{Material, Mesh},
};

use super::{
    renderer_objects::{renderer_camera::CameraHandler, renderer_layer::RendererLayerHandler},
    renderer_pipeline_step::RendererPipelineStep,
    MaterialHandler, MeshHandler, RendererError, RendererGroupHandler, RendererObjectHandler,
    ShaderHandler, TransformHandler,
};

pub enum Command {
    SetRendererPipeline {
        steps: Vec<RendererPipelineStep>,
        result_sender: oneshot::Sender<Result<(), RendererError>>,
    },
    CreateRendererLayer {
        camera_handler: CameraHandler,
        result_sender: oneshot::Sender<Result<RendererLayerHandler, RendererError>>,
    },
    ReleaseRendererLayer {
        object_pool_index: ObjectPoolIndex,
    },
    CreateRendererGroup {
        result_sender: oneshot::Sender<Result<RendererGroupHandler, RendererError>>,
    },
    ReleaseRendererGroup {
        object_pool_index: ObjectPoolIndex,
    },
    CreateTransform {
        transform: Transform<f32, f32, f32>,
        result_sender: oneshot::Sender<Result<TransformHandler, RendererError>>,
    },
    UpdateTransform {
        transform_handler: TransformHandler,
        new_transform: Transform<f32, f32, f32>,
        result_sender: oneshot::Sender<Result<(), RendererError>>,
    },
    ReleaseTransform {
        object_pool_index: ObjectPoolIndex,
    },
    CreateMaterial {
        material: Material,
        result_sender: oneshot::Sender<Result<MaterialHandler, RendererError>>,
    },
    ReleaseMaterial {
        object_pool_index: ObjectPoolIndex,
    },
    CreateShader {
        shader_name: String,
        result_sender: oneshot::Sender<Result<ShaderHandler, RendererError>>,
    },
    ReleaseShader {
        object_pool_index: ObjectPoolIndex,
    },
    CreateMesh {
        mesh: Arc<Mesh>,
        result_sender: oneshot::Sender<Result<MeshHandler, RendererError>>,
    },
    ReleaseMesh {
        object_pool_index: ObjectPoolIndex,
    },
    CreateRendererObjectFromMesh {
        mesh_handler: MeshHandler,
        shader_handler: ShaderHandler,
        material_handler: MaterialHandler,
        transform_handler: TransformHandler,
        result_sender: oneshot::Sender<Result<RendererObjectHandler, RendererError>>,
    },
    ReleaseRendererObject {
        object_pool_index: ObjectPoolIndex,
    },
    AddRendererGroupToLayer {
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
        result_sender: oneshot::Sender<Result<(), RendererError>>,
    },
    RemoveRendererGroupFromLayer {
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
        result_sender: oneshot::Sender<Result<(), RendererError>>,
    },
    AddRendererObjectToGroup {
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
        result_sender: oneshot::Sender<Result<(), RendererError>>,
    },
    RemoveRendererObjectFromGroup {
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
        result_sender: oneshot::Sender<Result<(), RendererError>>,
    },
    CreateCamera {
        transform_handler: TransformHandler,
        result_sender: oneshot::Sender<Result<CameraHandler, RendererError>>,
    },
    ReleaseCamera {
        object_pool_index: ObjectPoolIndex,
    },
}
