use std::sync::Arc;

use tokio::sync::oneshot;
use vek::Transform;

use crate::{
    mesh::{Material, Mesh},
    prelude::ResultInspector,
    sync::command_channel::CommandSender,
};

use super::{
    renderer_command::Command, renderer_objects::renderer_camera::CameraHandler,
    renderer_pipeline_step::RendererPipelineStep, MaterialHandler, MeshHandler, RendererError,
    RendererGroupHandler, RendererLayerHandler, RendererObjectHandler, ShaderHandler,
    TransformHandler,
};

#[derive(Clone)]
pub struct RendererClient {
    pub(super) command_sender: CommandSender<Command>,
}

impl RendererClient {
    pub async fn set_renderer_pipeline(
        &self,
        steps: Vec<RendererPipelineStep>,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::SetRendererPipeline {
                steps,
                result_sender,
            })
            .inspect_err(|e| log::error!("Setting renderer pipeline, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Setting renderer pipeline response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn create_renderer_layer(
        &self,
        camera_handler: CameraHandler,
    ) -> Result<RendererLayerHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::CreateRendererLayer {
                camera_handler,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating renderer group, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating renderer group response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn create_renderer_group(&self) -> Result<RendererGroupHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::CreateRendererGroup { result_sender })
            .inspect_err(|e| log::error!("Creating renderer group, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating renderer group response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn create_transform(
        &self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<TransformHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::CreateTransform {
                transform,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating transform, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating transform response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn update_transform(
        &self,
        transform_handler: TransformHandler,
        new_transform: Transform<f32, f32, f32>,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::UpdateTransform {
                transform_handler,
                new_transform,
                result_sender,
            })
            .inspect_err(|e| log::error!("Updating transform, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Updating transform response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn create_material(
        &self,
        material: Material,
    ) -> Result<MaterialHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::CreateMaterial {
                material,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating material, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating material response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn create_shader(&self, shader_name: String) -> Result<ShaderHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::CreateShader {
                shader_name,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating shader, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating shader response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn create_mesh(&self, mesh: Arc<Mesh>) -> Result<MeshHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::CreateMesh {
                mesh,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating renderer mesh, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating renderer mesh response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
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
        self.command_sender
            .send(Command::CreateRendererObjectFromMesh {
                mesh_handler,
                shader_handler,
                material_handler,
                transform_handler,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating renderer object from mesh, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Creating renderer object from mesh response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn add_renderer_group_to_layer(
        &self,
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::AddRendererGroupToLayer {
                renderer_group_handler,
                renderer_layer_handler,
                result_sender,
            })
            .inspect_err(|e| log::error!("Adding renderer group to layer, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Adding renderer group to layer response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn remove_renderer_group_from_layer(
        &self,
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::RemoveRendererGroupFromLayer {
                renderer_group_handler,
                renderer_layer_handler,
                result_sender,
            })
            .inspect_err(|e| log::error!("Removing renderer group from layer, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Removing renderer group from layer response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn add_renderer_object_to_group(
        &self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::AddRendererObjectToGroup {
                renderer_object_handler,
                renderer_group_handler,
                result_sender,
            })
            .inspect_err(|e| log::error!("Adding renderer object to group, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Adding renderer object to group response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn remove_renderer_object_from_group(
        &self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::RemoveRendererObjectFromGroup {
                renderer_object_handler,
                renderer_group_handler,
                result_sender,
            })
            .inspect_err(|e| log::error!("Removing renderer object from group, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver
            .await
            .inspect_err(|e| log::error!("Removing renderer object from group response, msg = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }

    pub async fn create_camera(
        &self,
        transform_handler: TransformHandler,
    ) -> Result<CameraHandler, RendererError> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.command_sender
            .send(Command::CreateCamera {
                transform_handler,
                result_sender,
            })
            .inspect_err(|e| log::error!("Removing camera from renderer, msg = {e:?}"))
            .map_err(|_| RendererError::RendererSystemDropped)?;

        match result_receiver.await.inspect_err(|e| {
            log::error!("Removing renderer object from renderer response, msg = {e}")
        }) {
            Ok(ret) => ret,
            Err(_) => Err(RendererError::RendererSystemDropped),
        }
    }
}
