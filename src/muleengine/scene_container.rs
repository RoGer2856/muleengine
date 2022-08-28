use std::collections::HashMap;
use std::sync::Arc;

use super::assets_reader::AssetsReader;
use super::image_container::ImageContainer;
use super::mesh::{Scene, SceneLoadError};

pub struct SceneContainer {
    scenes: HashMap<String, Arc<Scene>>,
}

impl SceneContainer {
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
        }
    }

    pub fn get_scene(
        &mut self,
        scene_path: &str,
        assets_reader: &AssetsReader,
        image_container: &mut ImageContainer,
    ) -> Result<Arc<Scene>, SceneLoadError> {
        if let Some(scene_mut) = self.scenes.get_mut(scene_path) {
            Ok(scene_mut.clone())
        } else {
            let scene = Arc::new(Scene::load(assets_reader, scene_path, image_container)?);
            self.scenes.insert(scene_path.to_string(), scene.clone());

            Ok(scene)
        }
    }
}
