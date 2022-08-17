use std::collections::HashMap;
use std::sync::Arc;

use crate::muleengine::assets_reader::AssetsReader;
use crate::muleengine::mesh::SceneLoadError;
use crate::muleengine::scene_container::SceneContainer;

use super::gl_mesh::GLMesh;
use super::gl_scene::GLScene;

pub struct GLSceneContainer {
    mesh_container: SceneContainer,
    scenes: HashMap<String, Arc<GLScene>>,
}

impl GLSceneContainer {
    pub fn new(mesh_container: SceneContainer) -> Self {
        Self {
            mesh_container,
            scenes: HashMap::new(),
        }
    }

    pub fn get_scene(
        &mut self,
        scene_path: &str,
        assets_reader: &AssetsReader,
    ) -> Result<Arc<GLScene>, SceneLoadError> {
        if let Some(scene_mut) = self.scenes.get_mut(scene_path) {
            Ok(scene_mut.clone())
        } else {
            let scene = self.mesh_container.get_scene(scene_path, assets_reader)?;

            let mut gl_scene = GLScene::new();

            for mesh in scene.meshes_ref().iter() {
                match mesh {
                    Ok(mesh) => {
                        let gl_mesh = Arc::new(GLMesh::new(mesh.clone()));

                        gl_scene.meshes_mut().push(Ok(gl_mesh));
                    }
                    Err(e) => {
                        gl_scene.meshes_mut().push(Err(e.clone()));
                    }
                }
            }

            let gl_scene = Arc::new(gl_scene);
            self.scenes.insert(scene_path.to_string(), gl_scene.clone());

            Ok(gl_scene)
        }
    }
}
