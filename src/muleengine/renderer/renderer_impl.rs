use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Transform, Vec2};

use crate::muleengine::{
    camera::Camera,
    mesh::{Material, Mesh},
};

use super::{DrawableMesh, DrawableObject};

pub trait RendererImpl {
    fn render(&mut self);

    fn set_window_dimensions(&mut self, dimensions: Vec2<usize>);
    fn set_camera(&mut self, camera: Camera);

    fn create_drawable_mesh(
        &mut self,
        mesh: Arc<Mesh>,
    ) -> Result<Arc<RwLock<dyn DrawableMesh>>, String>;

    fn create_drawable_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn DrawableMesh>>,
        material: Option<Material>,
        shader_path: String,
    ) -> Result<Arc<RwLock<dyn DrawableObject>>, String>;

    fn add_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn DrawableObject>>,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), String>;
    fn remove_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn DrawableObject>>,
    ) -> Result<(), String>;
}
