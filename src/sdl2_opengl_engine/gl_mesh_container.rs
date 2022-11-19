use std::collections::BTreeMap;
use std::sync::Arc;

use muleengine::mesh::{Mesh, MeshConvertError, Scene};

use super::gl_mesh::GLMesh;
use super::gl_texture_container::GLTextureContainer;

pub struct GLMeshContainer {
    meshes: BTreeMap<*const Mesh, Arc<GLMesh>>,
}

impl Default for GLMeshContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl GLMeshContainer {
    pub fn new() -> Self {
        Self {
            meshes: BTreeMap::new(),
        }
    }

    pub fn get_gl_meshes_from_scene(
        &mut self,
        scene: Arc<Scene>,
        gl_texture_container: &mut GLTextureContainer,
    ) -> Vec<Result<Arc<GLMesh>, MeshConvertError>> {
        let mut ret = Vec::new();

        for mesh in scene.meshes_ref().iter() {
            match mesh {
                Ok(mesh) => {
                    let drawable_object =
                        self.get_drawable_mesh(mesh.clone(), gl_texture_container);
                    ret.push(Ok(drawable_object));
                }
                Err(e) => {
                    ret.push(Err(e.clone()));
                }
            }
        }

        ret
    }

    pub fn get_drawable_mesh(
        &mut self,
        mesh: Arc<Mesh>,
        gl_texture_container: &mut GLTextureContainer,
    ) -> Arc<GLMesh> {
        let inner_container = self
            .meshes
            .entry(&*mesh)
            .or_insert_with(|| Arc::new(GLMesh::new(mesh, gl_texture_container)));

        inner_container.clone()
    }

    pub fn release_mesh(&mut self, mesh: Arc<Mesh>) {
        let key: *const Mesh = &*mesh;
        self.meshes.remove(&key);
    }
}
