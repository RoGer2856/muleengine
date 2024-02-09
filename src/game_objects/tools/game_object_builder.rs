use std::sync::Arc;

use entity_component::EntityBuilder;
use muleengine::{
    mesh::{Material, Mesh},
    renderer::{
        RendererGroupHandler, RendererMaterialHandler, RendererMeshHandler, RendererShaderHandler,
    },
};
use vek::{Transform, Vec3};

use crate::{
    essential_services::EssentialServices,
    physics::{
        collider::ColliderShape,
        rigid_body::{RigidBodyBuilder, RigidBodyType},
    },
};

#[derive(Clone)]
pub struct GameObjectBuilder<'a> {
    essentials: &'a Arc<EssentialServices>,
    shader_handler: Option<RendererShaderHandler>,
    transform: Option<Transform<f32, f32, f32>>,
    mesh_default_material: Option<Material>,
    mesh_handler: Option<RendererMeshHandler>,
    material_handler: Option<RendererMaterialHandler>,
    renderer_group_handler: Option<RendererGroupHandler>,
    rigid_body_builder: Option<RigidBodyBuilder>,
}

impl<'a> GameObjectBuilder<'a> {
    pub fn new(essentials: &'a Arc<EssentialServices>) -> Self {
        Self {
            essentials,
            shader_handler: None,
            transform: None,
            mesh_default_material: None,
            mesh_handler: None,
            material_handler: None,
            renderer_group_handler: None,
            rigid_body_builder: None,
        }
    }

    pub fn reset_material(mut self) -> GameObjectBuilder<'a> {
        self.material_handler = None;
        self
    }

    pub async fn material(mut self, material: Material) -> GameObjectBuilder<'a> {
        self.material_handler = Some(
            self.essentials
                .renderer_client
                .create_material(material)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap(),
        );
        self
    }

    pub fn material_handler(mut self, material_handler: RendererMaterialHandler) -> Self {
        self.material_handler = Some(material_handler);
        self
    }

    pub fn reset_mesh(mut self) -> GameObjectBuilder<'a> {
        self.mesh_handler = None;
        self.mesh_default_material = None;
        self
    }

    pub async fn mesh(mut self, mesh: Arc<Mesh>) -> GameObjectBuilder<'a> {
        self.mesh_default_material = Some(mesh.get_material().clone());

        self.mesh_handler = Some(
            self.essentials
                .renderer_client
                .create_mesh(mesh)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap(),
        );
        self
    }

    pub fn mesh_handler(mut self, mesh_handler: RendererMeshHandler) -> Self {
        self.mesh_handler = Some(mesh_handler);
        self
    }

    pub fn reset_transform(mut self) -> GameObjectBuilder<'a> {
        self.transform = None;
        self
    }

    pub async fn transform(mut self, transform: Transform<f32, f32, f32>) -> GameObjectBuilder<'a> {
        self.transform = Some(transform);
        self
    }

    pub fn reset_shader(mut self) -> GameObjectBuilder<'a> {
        self.shader_handler = None;
        self
    }

    pub async fn shader(mut self, shader_name: impl Into<String>) -> GameObjectBuilder<'a> {
        self.shader_handler = Some(
            self.essentials
                .renderer_client
                .create_shader(shader_name.into())
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap(),
        );
        self
    }

    pub fn shader_handler(mut self, shader_handler: RendererShaderHandler) -> Self {
        self.shader_handler = Some(shader_handler);
        self
    }

    pub fn reset_renderer_group_handler(mut self) -> GameObjectBuilder<'a> {
        self.renderer_group_handler = None;
        self
    }

    pub fn renderer_group_handler(mut self, group_handler: RendererGroupHandler) -> Self {
        self.renderer_group_handler = Some(group_handler);
        self
    }

    pub fn simple_rigid_body(
        mut self,
        position: Vec3<f32>,
        collider_shape: ColliderShape,
        rigid_body_type: RigidBodyType,
    ) -> Self {
        let physics_engine = self.essentials.physics_engine.write();

        let collider = physics_engine.collider_builder(collider_shape).build();

        let rigid_body_builder = physics_engine
            .rigid_body_builder(collider, rigid_body_type)
            .position(position);

        self.rigid_body_builder = Some(rigid_body_builder);

        self
    }

    pub async fn build(&self) -> EntityBuilder {
        let entity_builder = self.essentials.entity_container.entity_builder();

        let entity_builder = if let Some(objects) = self
            .mesh_handler
            .as_ref()
            .zip(self.renderer_group_handler.as_ref())
            .zip(self.shader_handler.as_ref())
        {
            let mesh_handler_ref = objects.0 .0;
            let shader_handler_ref = objects.1;
            let renderer_group_handler_ref = objects.0 .1;

            let transform = self.transform.unwrap_or_default();
            let transform_handler = self
                .essentials
                .renderer_client
                .create_transform(transform)
                .await
                .inspect_err(|e| log::error!("{e:?}"))
                .unwrap()
                .unwrap();

            let material_handler =
                if let Some(material_handler_ref) = self.material_handler.as_ref() {
                    material_handler_ref.clone()
                } else {
                    let material = if let Some(material) = &self.mesh_default_material {
                        material.clone()
                    } else {
                        Material::default()
                    };

                    self.essentials
                        .renderer_client
                        .create_material(material)
                        .await
                        .inspect_err(|e| log::error!("{e:?}"))
                        .unwrap()
                        .unwrap()
                };

            let renderer_object_handler = self
                .essentials
                .renderer_client
                .create_renderer_object_from_mesh(
                    mesh_handler_ref.clone(),
                    shader_handler_ref.clone(),
                    material_handler,
                    transform_handler.clone(),
                )
                .await
                .unwrap()
                .unwrap();

            self.essentials
                .renderer_client
                .add_renderer_object_to_group(
                    renderer_object_handler.clone(),
                    renderer_group_handler_ref.clone(),
                )
                .await
                .unwrap()
                .unwrap();

            entity_builder
                .with_component(renderer_object_handler.clone())
                .with_component(transform_handler)
                .with_component(transform)
        } else {
            entity_builder
        };

        let entity_builder = if let Some(rigid_body_builder) = self.rigid_body_builder.clone() {
            let rigid_body_handler =
                rigid_body_builder.build(&mut self.essentials.physics_engine.write());
            entity_builder.with_component(rigid_body_handler)
        } else {
            entity_builder
        };

        entity_builder
    }
}
