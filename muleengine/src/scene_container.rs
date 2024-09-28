use std::collections::HashMap;
use std::sync::Arc;

use super::asset_reader::AssetReader;
use super::image_container::ImageContainer;
use super::mesh::{Scene, SceneLoadError};

pub struct SceneContainer {
    scenes: HashMap<String, Arc<Scene>>,
}

impl Default for SceneContainer {
    fn default() -> Self {
        Self::new()
    }
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
        asset_reader: &AssetReader,
        image_container: &mut ImageContainer,
    ) -> Result<Arc<Scene>, SceneLoadError> {
        if let Some(scene_mut) = self.scenes.get_mut(scene_path) {
            Ok(scene_mut.clone())
        } else {
            let scene = Arc::new(Scene::from_reader(
                asset_reader,
                scene_path,
                image_container,
            )?);
            self.scenes.insert(scene_path.to_string(), scene.clone());

            Ok(scene)
        }
    }
}
