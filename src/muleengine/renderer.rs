use std::sync::Arc;

use vek::{Transform, Vec2};

use super::{
    camera::Camera,
    drawable_object_storage::DrawableObjectStorageIndex,
    mesh::{Material, Mesh},
};

pub trait RendererClientClone: 'static {
    fn clone_box(&self) -> Box<dyn RendererClient>;
}

pub trait RendererClient: RendererClientClone + Send {
    fn add_drawable_mesh(
        &self,
        mesh: Arc<Mesh>,
        transform: Transform<f32, f32, f32>,
        material: Option<Material>,
        shader_path: String,
    ) -> DrawableObjectStorageIndex;
    fn set_camera(&self, camera: Camera);
    fn set_window_dimensions(&self, dimensions: Vec2<usize>);
}

impl<T> RendererClientClone for T
where
    T: RendererClient + Clone,
{
    fn clone_box(&self) -> Box<dyn RendererClient> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn RendererClient> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
