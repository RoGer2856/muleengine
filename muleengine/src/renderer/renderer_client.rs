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
    MaterialHandler, MeshHandler, RendererError, RendererGroupHandler, RendererObjectHandler,
    ShaderHandler, TransformHandler,
};

#[derive(Clone)]
pub struct RendererClient {
    pub(super) command_sender: CommandSender,
}

impl RendererClient {
    pub async fn create_renderer_group(&self) -> Result<RendererGroupHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateRendererGroup { result_sender })
            .inspect_err(|e| log::error!("Creating renderer group, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating renderer group response, error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn create_transform(
        &self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<TransformHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateTransform {
                transform,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating transform, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating transform response, error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub async fn update_transform(
        &self,
        transform_handler: TransformHandler,
        new_transform: Transform<f32, f32, f32>,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::UpdateTransform {
                transform_handler,
                new_transform,
                result_sender,
            })
            .inspect_err(|e| log::error!("Updating transform, error = {e}"));

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Updating transform response, error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

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
        transform_handler: TransformHandler,
    ) -> Result<RendererObjectHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::CreateRendererObjectFromMesh {
                mesh_handler,
                shader_handler,
                material_handler,
                transform_handler,
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

    pub async fn add_renderer_object_to_group(
        &self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let _ = self
            .command_sender
            .send(Command::AddRendererObjectToGroup {
                renderer_object_handler,
                renderer_group_handler,
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
