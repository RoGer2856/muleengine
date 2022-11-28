use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};
use vek::{Transform, Vec2};

use crate::{
    camera::Camera,
    containers::object_pool::ObjectPoolIndex,
    mesh::{Material, Mesh},
};

use super::{
    MaterialHandler, MeshHandler, RendererError, RendererObjectHandler, ShaderHandler,
    TransformHandler,
};

pub enum Command {
    CreateTransform {
        transform: Transform<f32, f32, f32>,
        result_sender: oneshot::Sender<Result<TransformHandler, RendererError>>,
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
    AddRendererObject {
        renderer_object_handler: RendererObjectHandler,
        result_sender: oneshot::Sender<Result<(), RendererError>>,
    },

    SetCamera {
        camera: Camera,
    },
    SetWindowDimensions {
        dimensions: Vec2<usize>,
    },
}

pub type CommandSender = mpsc::UnboundedSender<Command>;
pub type CommandReceiver = mpsc::UnboundedReceiver<Command>;
