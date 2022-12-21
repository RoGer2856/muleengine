use std::sync::Arc;

use vek::{Transform, Vec2};

use crate::{
    camera::Camera,
    mesh::{Material, Mesh},
    prelude::ArcRwLock,
};

use super::{
    renderer_objects::{renderer_camera::RendererCamera, renderer_layer::RendererLayer},
    renderer_pipeline_step_impl::RendererPipelineStepImpl,
    RendererGroup, RendererMaterial, RendererMesh, RendererObject, RendererShader,
    RendererTransform,
};

pub trait RendererImpl {
    fn render(&mut self);

    fn set_window_dimensions(&mut self, dimensions: Vec2<usize>);
    fn set_camera(&mut self, camera: Camera);

    fn set_renderer_pipeline(&mut self, steps: Vec<RendererPipelineStepImpl>)
        -> Result<(), String>;

    fn create_renderer_layer(&mut self) -> Result<ArcRwLock<dyn RendererLayer>, String>;
    fn release_renderer_layer(
        &mut self,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String>;

    fn add_renderer_group_to_layer(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String>;
    fn remove_renderer_group_from_layer(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String>;

    fn create_renderer_group(&mut self) -> Result<ArcRwLock<dyn RendererGroup>, String>;
    fn release_renderer_group(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String>;

    fn create_transform(
        &mut self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<ArcRwLock<dyn RendererTransform>, String>;
    fn update_transform(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
        new_transform: Transform<f32, f32, f32>,
    ) -> Result<(), String>;
    fn release_transform(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<(), String>;

    fn create_material(
        &mut self,
        material: Material,
    ) -> Result<ArcRwLock<dyn RendererMaterial>, String>;
    fn release_material(&mut self, material: ArcRwLock<dyn RendererMaterial>)
        -> Result<(), String>;

    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<ArcRwLock<dyn RendererShader>, String>;
    fn release_shader(&mut self, shader: ArcRwLock<dyn RendererShader>) -> Result<(), String>;

    fn create_mesh(&mut self, mesh: Arc<Mesh>) -> Result<ArcRwLock<dyn RendererMesh>, String>;
    fn release_mesh(&mut self, mesh: ArcRwLock<dyn RendererMesh>) -> Result<(), String>;

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: ArcRwLock<dyn RendererMesh>,
        shader: ArcRwLock<dyn RendererShader>,
        material: ArcRwLock<dyn RendererMaterial>,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<ArcRwLock<dyn RendererObject>, String>;
    fn release_renderer_object(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
    ) -> Result<(), String>;

    fn add_renderer_object_to_group(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String>;
    fn remove_renderer_object_from_group(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String>;

    fn create_camera(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<ArcRwLock<dyn RendererCamera>, String>;
    fn release_camera(&mut self, camera: ArcRwLock<dyn RendererCamera>) -> Result<(), String>;
}

pub trait AsRendererImpl {
    fn as_renderer_impl_ref(&self) -> &dyn RendererImpl;
    fn as_renderer_impl_mut(&mut self) -> &mut dyn RendererImpl;
}

pub trait RendererImplAsync: AsRendererImpl + RendererImpl + Send + 'static {
    fn box_clone(&self) -> Box<dyn RendererImplAsync + 'static>;
}

impl<T: RendererImpl + AsRendererImpl + Clone + Send + 'static> RendererImplAsync for T {
    fn box_clone(&self) -> Box<dyn RendererImplAsync> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn RendererImplAsync> {
    fn clone(&self) -> Box<dyn RendererImplAsync> {
        self.box_clone()
    }
}

impl<T: RendererImpl + Clone + Send + 'static> AsRendererImpl for T {
    fn as_renderer_impl_ref(&self) -> &dyn RendererImpl {
        self
    }

    fn as_renderer_impl_mut(&mut self) -> &mut dyn RendererImpl {
        self
    }
}
