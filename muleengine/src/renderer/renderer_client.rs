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
    MaterialHandler, MeshHandler, RendererError, RendererObjectHandler, ShaderHandler,
};

#[derive(Clone)]
pub struct RendererClient {
    pub(super) command_sender: CommandSender,
}

impl RendererClient {
    pub async fn create_material(
        &self,
        material: Material,
    ) -> Result<MaterialHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateMaterial {
                material,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating material, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating material response, error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn create_shader(&self, shader_name: String) -> Result<ShaderHandler, RendererError> {
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
            .inspect_err(|e| log::error!("Creating shader response, error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn create_mesh(&self, mesh: Arc<Mesh>) -> Result<MeshHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateMesh {
                mesh,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating renderer mesh, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating renderer mesh response, error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn create_renderer_object_from_mesh(
        &self,
        mesh_handler: MeshHandler,
        shader_handler: ShaderHandler,
        material_handler: MaterialHandler,
    ) -> Result<RendererObjectHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateRendererObjectFromMesh {
                mesh_handler,
                shader_handler,
                material_handler,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating renderer object from mesh, error = {e}"));

        match result_receiver.await.inspect_err(|e| {
            log::error!("Creating renderer object from mesh response, error = {e}")
        }) {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn add_renderer_object(
        &self,
        renderer_object_handler: RendererObjectHandler,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::AddRendererObject {
                renderer_object_handler,
                transform,
                result_sender,
            })
            .inspect_err(|e| log::error!("Adding renderer object to renderer, error = {e}"));

        match result_receiver.await.inspect_err(|e| {
            log::error!("Adding renderer object to renderer response, error = {e}")
        }) {
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
