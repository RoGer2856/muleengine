use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};
use vek::{Transform, Vec2};

use crate::muleengine::{
    camera::Camera,
    mesh::{Material, Mesh},
};

use super::{DrawableMeshId, DrawableObjectId, RendererError};

pub enum Command {
    CreateDrawableMesh {
        mesh: Arc<Mesh>,
        result_sender: oneshot::Sender<Result<DrawableMeshId, RendererError>>,
    },
    CreateDrawableObjectFromMesh {
        mesh_id: DrawableMeshId,
        material: Option<Material>,
        shader_path: String,
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
