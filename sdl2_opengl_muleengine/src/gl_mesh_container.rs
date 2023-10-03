use std::sync::Arc;
use std::{collections::BTreeMap, rc::Rc};

use muleengine::mesh::{Mesh, MeshConvertError, Scene};

use super::gl_mesh::GLMesh;

pub struct GLMeshContainer {
    meshes: BTreeMap<*const Mesh, Rc<GLMesh>>,
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
        scene: Rc<Scene>,
    ) -> Vec<Result<Rc<GLMesh>, MeshConvertError>> {
        let mut ret = Vec::new();

        for mesh in scene.meshes_ref().iter() {
            match mesh {
                Ok(mesh) => {
                    let gl_mesh = self.get_gl_mesh(mesh.clone());
                    ret.push(Ok(gl_mesh));
                }
                Err(e) => {
                    ret.push(Err(e.clone()));
                }
            }
        }

        ret
    }

    pub fn get_gl_mesh(&mut self, mesh: Arc<Mesh>) -> Rc<GLMesh> {
        let mesh = self
            .meshes
            .entry(&*mesh)
            .or_insert_with(|| Rc::new(GLMesh::new(mesh)));

        mesh.clone()
    }

    pub fn release_mesh(&mut self, mesh: Arc<Mesh>) {
        let key: *const Mesh = &*mesh;
        self.meshes.remove(&key);
    }
}
