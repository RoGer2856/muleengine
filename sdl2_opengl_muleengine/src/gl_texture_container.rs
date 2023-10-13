use std::collections::HashMap;
use std::sync::Arc;

use muleengine::image::Image;

use super::opengl_utils::texture_2d::Texture2D;

pub struct GLTextureContainer {
    textures_2d: HashMap<*const Image, (Arc<Image>, Arc<Texture2D>)>,
}

impl Default for GLTextureContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl GLTextureContainer {
    pub fn new() -> Self {
        Self {
            textures_2d: HashMap::new(),
        }
    }

    pub fn get_texture(&mut self, image: Arc<Image>) -> Arc<Texture2D> {
        self.textures_2d
            .entry(&*image)
            .or_insert_with(|| (image.clone(), Arc::new(Texture2D::new(image))))
            .1
            .clone()
    }
}
