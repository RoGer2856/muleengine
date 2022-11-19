use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};
use vek::{Transform, Vec2};

use crate::muleengine::{
    camera::Camera,
    mesh::{Material, Mesh},
};

use super::{DrawableMeshId, DrawableObjectId, RendererError, ShaderId};

pub enum Command {
    CreateShader {
        shader_name: String,
        result_sender: oneshot::Sender<Result<ShaderId, RendererError>>,
    },
    CreateDrawableMesh {
        mesh: Arc<Mesh>,
        result_sender: oneshot::Sender<Result<DrawableMeshId, RendererError>>,
    },
    CreateDrawableObjectFromMesh {
        mesh_id: DrawableMeshId,
        shader_id: ShaderId,
        material: Option<Material>,
        result_sender: oneshot::Sender<Result<DrawableObjectId, RendererError>>,
    },

    AddDrawableObject {
        drawable_object_id: DrawableObjectId,
        transform: Transform<f32, f32, f32>,
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
