use std::collections::BTreeMap;

use muleengine::prelude::ArcRwLock;
use vek::{Mat4, Vec3};

use crate::mesh_renderer_object::MeshRendererObject;

pub struct RendererGroup {
    mesh_renderer_objects: BTreeMap<*const MeshRendererObject, ArcRwLock<MeshRendererObject>>,
}

pub type RendererGroupObject = RendererGroup;

impl Default for RendererGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl RendererGroup {
    pub fn new() -> Self {
        Self {
            mesh_renderer_objects: BTreeMap::new(),
        }
    }

    pub fn add_mesh_renderer_object(
        &mut self,
        renderer_object: ArcRwLock<MeshRendererObject>,
    ) -> Option<ArcRwLock<MeshRendererObject>> {
        self.mesh_renderer_objects
            .insert(renderer_object.data_ptr(), renderer_object)
    }

    pub fn remove_mesh_renderer_object(
        &mut self,
        renderer_object: &ArcRwLock<MeshRendererObject>,
    ) -> Option<ArcRwLock<MeshRendererObject>> {
        let ptr: *const MeshRendererObject = renderer_object.data_ptr();
        self.mesh_renderer_objects.remove(&ptr)
    }

    pub fn draw(
        &self,
        eye_position: &Vec3<f32>,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
    ) {
        for renderer_object in self.mesh_renderer_objects.values() {
            renderer_object.read().gl_drawable_mesh.draw(
                eye_position,
                projection_matrix,
                view_matrix,
            );
        }
    }
}
