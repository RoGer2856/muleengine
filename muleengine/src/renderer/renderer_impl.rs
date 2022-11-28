use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Transform, Vec2};

use crate::{
    camera::Camera,
    mesh::{Material, Mesh},
};

use super::{RendererMaterial, RendererMesh, RendererObject, RendererShader, RendererTransform};

pub trait RendererImpl {
    fn render(&mut self);

    fn set_window_dimensions(&mut self, dimensions: Vec2<usize>);
    fn set_camera(&mut self, camera: Camera);

    fn create_transform(
        &mut self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<Arc<RwLock<dyn RendererTransform>>, String>;
    fn release_transform(
        &mut self,
        transform: Arc<RwLock<dyn RendererTransform>>,
    ) -> Result<(), String>;

    fn create_material(
        &mut self,
        material: Material,
    ) -> Result<Arc<RwLock<dyn RendererMaterial>>, String>;
    fn release_material(
        &mut self,
        material: Arc<RwLock<dyn RendererMaterial>>,
    ) -> Result<(), String>;

    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<Arc<RwLock<dyn RendererShader>>, String>;
    fn release_shader(&mut self, shader: Arc<RwLock<dyn RendererShader>>) -> Result<(), String>;

    fn create_mesh(&mut self, mesh: Arc<Mesh>) -> Result<Arc<RwLock<dyn RendererMesh>>, String>;
    fn release_mesh(&mut self, mesh: Arc<RwLock<dyn RendererMesh>>) -> Result<(), String>;

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn RendererMesh>>,
        shader: &Arc<RwLock<dyn RendererShader>>,
        material: &Arc<RwLock<dyn RendererMaterial>>,
        transform: &Arc<RwLock<dyn RendererTransform>>,
    ) -> Result<Arc<RwLock<dyn RendererObject>>, String>;
    fn release_renderer_object(
        &mut self,
        renderer_object: Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String>;

    fn add_renderer_object(
        &mut self,
        renderer_object: &Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String>;
    fn remove_renderer_object(
        &mut self,
        renderer_object: &Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String>;
}
