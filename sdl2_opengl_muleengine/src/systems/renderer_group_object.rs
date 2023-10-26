use std::collections::BTreeMap;

use muleengine::bytifex_utils::sync::types::RcRwLock;
use vek::{Mat4, Vec3};

use crate::gl_drawable_mesh::GLDrawableMesh;

pub(crate) struct RendererGroupObject {
    mesh_renderer_objects: BTreeMap<*const GLDrawableMesh, RcRwLock<GLDrawableMesh>>,
}

impl Default for RendererGroupObject {
    fn default() -> Self {
        Self::new()
    }
}

impl RendererGroupObject {
    pub fn new() -> Self {
        Self {
            mesh_renderer_objects: BTreeMap::new(),
        }
    }

    pub fn add_mesh_renderer_object(
        &mut self,
        renderer_object: RcRwLock<GLDrawableMesh>,
    ) -> Option<RcRwLock<GLDrawableMesh>> {
        self.mesh_renderer_objects
            .insert(renderer_object.data_ptr(), renderer_object)
    }

    pub fn remove_mesh_renderer_object(
        &mut self,
        renderer_object: &RcRwLock<GLDrawableMesh>,
    ) -> Option<RcRwLock<GLDrawableMesh>> {
        let ptr: *const GLDrawableMesh = renderer_object.data_ptr();
        self.mesh_renderer_objects.remove(&ptr)
    }

    pub fn draw(
        &self,
        eye_position: &Vec3<f32>,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
    ) {
        for renderer_object in self.mesh_renderer_objects.values() {
            renderer_object
                .read()
                .draw(eye_position, projection_matrix, view_matrix);
        }
    }
}
