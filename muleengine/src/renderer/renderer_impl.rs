use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Transform, Vec2};

use crate::{
    camera::Camera,
    mesh::{Material, Mesh},
};

use super::{RendererMaterial, RendererMesh, RendererObject, RendererShader};

pub trait RendererImpl {
    fn render(&mut self);

    fn set_window_dimensions(&mut self, dimensions: Vec2<usize>);
    fn set_camera(&mut self, camera: Camera);

    fn create_material(
        &mut self,
        material: Material,
    ) -> Result<Arc<RwLock<dyn RendererMaterial>>, String>;

    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<Arc<RwLock<dyn RendererShader>>, String>;

    fn create_drawable_mesh(
        &mut self,
        mesh: Arc<Mesh>,
    ) -> Result<Arc<RwLock<dyn RendererMesh>>, String>;

    fn create_drawable_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn RendererMesh>>,
        shader: &Arc<RwLock<dyn RendererShader>>,
        material: &Arc<RwLock<dyn RendererMaterial>>,
    ) -> Result<Arc<RwLock<dyn RendererObject>>, String>;

    fn add_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn RendererObject>>,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), String>;
    fn remove_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String>;
}
