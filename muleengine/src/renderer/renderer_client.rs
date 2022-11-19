use std::sync::Arc;

use tokio::sync::oneshot;
use vek::{Transform, Vec2};

use crate::{
    camera::Camera,
    mesh::{Material, Mesh},
    result_option_inspect::ResultInspector,
};

use super::{
    renderer_command::{Command, CommandSender},
    DrawableMeshId, DrawableObjectId, RendererError, ShaderId,
};

#[derive(Clone)]
pub struct RendererClient {
    pub(super) command_sender: CommandSender,
}

impl RendererClient {
    pub async fn create_shader(&self, shader_name: String) -> Result<ShaderId, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateShader {
                shader_name,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating shader, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating shader response error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn create_drawable_mesh(
        &self,
        mesh: Arc<Mesh>,
    ) -> Result<DrawableMeshId, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateDrawableMesh {
                mesh,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating drawable mesh, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating drawable mesh response error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn create_drawable_object_from_mesh(
        &self,
        mesh_id: DrawableMeshId,
        shader_id: ShaderId,
        material: Option<Material>,
    ) -> Result<DrawableObjectId, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateDrawableObjectFromMesh {
                mesh_id,
                shader_id,
                material,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating drawable object from mesh, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating drawable object from mesh response error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn add_drawable_object(
        &self,
        drawable_object_id: DrawableObjectId,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::AddDrawableObject {
                drawable_object_id,
                transform,
                result_sender,
            })
            .inspect_err(|e| log::error!("Adding drawable object to renderer, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Adding drawable object to renderer response error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub fn set_camera(&self, camera: Camera) {
        let _ = self
            .command_sender
            .send(Command::SetCamera { camera })
            .inspect_err(|e| log::error!("Setting camera of renderer, error = {e}"));
    }

    pub fn set_window_dimensions(&self, dimensions: Vec2<usize>) {
        let _ = self
            .command_sender
            .send(Command::SetWindowDimensions { dimensions })
            .inspect_err(|e| log::error!("Setting window dimensions of renderer, error = {e}"));
    }
}
