use std::sync::Arc;

use crate::muleengine::mesh::MeshConvertError;

use super::gl_mesh::GLMesh;

pub struct GLScene {
    meshes: Vec<Result<Arc<GLMesh>, MeshConvertError>>,
}

impl Default for GLScene {
    fn default() -> Self {
        Self::new()
    }
}

impl GLScene {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn meshes_ref(&self) -> &Vec<Result<Arc<GLMesh>, MeshConvertError>> {
        &self.meshes
    }

    pub fn meshes_mut(&mut self) -> &mut Vec<Result<Arc<GLMesh>, MeshConvertError>> {
        &mut self.meshes
    }
}
