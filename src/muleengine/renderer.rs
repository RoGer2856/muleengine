use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Transform, Vec2};

use super::{camera::Camera, drawable_object::DrawableObject};

pub trait RendererClientClone: 'static {
    fn clone_box(&self) -> Box<dyn RendererClient>;
}

pub trait RendererClient: RendererClientClone {
    fn add_drawable_object(
        &self,
        drawable_object: Arc<RwLock<dyn DrawableObject>>,
        transform: Transform<f32, f32, f32>,
    );
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
