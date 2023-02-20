use std::sync::Arc;

use renderer_client_fn::renderer_client_fn;
use tokio::sync::oneshot;
use vek::Transform;

use crate::{
    mesh::{Material, Mesh},
    prelude::ResultInspector,
    sync::command_channel::CommandSender,
};

use super::{
    renderer_command::Command, renderer_objects::renderer_camera::RendererCameraHandler,
    renderer_pipeline_step::RendererPipelineStep, RendererError, RendererGroupHandler,
    RendererLayerHandler, RendererMaterialHandler, RendererMeshHandler, RendererObjectHandler,
    RendererShaderHandler, RendererTransformHandler,
};

#[derive(Clone)]
pub struct RendererClient {
    pub(super) command_sender: CommandSender<Command>,
}

impl RendererClient {
    #[renderer_client_fn(Command::SetRendererPipeline)]
    pub async fn set_renderer_pipeline(
        &self,
        steps: Vec<RendererPipelineStep>,
    ) -> Result<(), RendererError>;

    #[renderer_client_fn(Command::CreateRendererLayer)]
    pub async fn create_renderer_layer(
        &self,
        camera_handler: RendererCameraHandler,
    ) -> Result<RendererLayerHandler, RendererError>;

    #[renderer_client_fn(Command::CreateRendererGroup)]
    pub async fn create_renderer_group(&self) -> Result<RendererGroupHandler, RendererError>;

    #[renderer_client_fn(Command::CreateTransform)]
    pub async fn create_transform(
        &self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<RendererTransformHandler, RendererError>;

    #[renderer_client_fn(Command::UpdateTransform)]
    pub async fn update_transform(
        &self,
        transform_handler: RendererTransformHandler,
        new_transform: Transform<f32, f32, f32>,
    ) -> Result<(), RendererError>;

    #[renderer_client_fn(Command::CreateMaterial)]
    pub async fn create_material(
        &self,
        material: Material,
    ) -> Result<RendererMaterialHandler, RendererError>;

    #[renderer_client_fn(Command::CreateShader)]
    pub async fn create_shader(
        &self,
        shader_name: String,
    ) -> Result<RendererShaderHandler, RendererError>;

    #[renderer_client_fn(Command::CreateMesh)]
    pub async fn create_mesh(&self, mesh: Arc<Mesh>) -> Result<RendererMeshHandler, RendererError>;

    #[renderer_client_fn(Command::CreateRendererObjectFromMesh)]
    pub async fn create_renderer_object_from_mesh(
        &self,
        mesh_handler: RendererMeshHandler,
        shader_handler: RendererShaderHandler,
        material_handler: RendererMaterialHandler,
        transform_handler: RendererTransformHandler,
    ) -> Result<RendererObjectHandler, RendererError>;

    #[renderer_client_fn(Command::AddRendererGroupToLayer)]
    pub async fn add_renderer_group_to_layer(
        &self,
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
    ) -> Result<(), RendererError>;

    #[renderer_client_fn(Command::RemoveRendererGroupFromLayer)]
    pub async fn remove_renderer_group_from_layer(
        &self,
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
    ) -> Result<(), RendererError>;

    #[renderer_client_fn(Command::AddRendererObjectToGroup)]
    pub async fn add_renderer_object_to_group(
        &self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
    ) -> Result<(), RendererError>;

    #[renderer_client_fn(Command::RemoveRendererObjectFromGroup)]
    pub async fn remove_renderer_object_from_group(
        &self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
    ) -> Result<(), RendererError>;

    #[renderer_client_fn(Command::CreateCamera)]
    pub async fn create_camera(
        &self,
        transform_handler: RendererTransformHandler,
    ) -> Result<RendererCameraHandler, RendererError>;
}
