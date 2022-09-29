use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Transform, Vec2};

use super::{camera::Camera, drawable_object::DrawableObject};

pub trait RendererClient {
    fn add_drawable_object(
        &self,
        drawable_object: Arc<RwLock<dyn DrawableObject>>,
        transform: Transform<f32, f32, f32>,
    );
    fn set_camera(&self, camera: Camera);
    fn set_window_dimensions(&self, dimensions: Vec2<usize>);
}
