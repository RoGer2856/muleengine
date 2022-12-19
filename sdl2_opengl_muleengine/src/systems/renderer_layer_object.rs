use std::collections::BTreeMap;

use muleengine::prelude::ArcRwLock;
use vek::{Mat4, Vec3};

use super::renderer_group_object::RendererGroupObject;

pub struct RendererLayerObject {
    renderer_groups: BTreeMap<*const RendererGroupObject, ArcRwLock<RendererGroupObject>>,
}

impl Default for RendererLayerObject {
    fn default() -> Self {
        Self::new()
    }
}

impl RendererLayerObject {
    pub fn new() -> Self {
        Self {
            renderer_groups: BTreeMap::new(),
        }
    }

    pub fn add_renderer_group(
        &mut self,
        renderer_group: ArcRwLock<RendererGroupObject>,
    ) -> Option<ArcRwLock<RendererGroupObject>> {
        self.renderer_groups
            .insert(renderer_group.data_ptr(), renderer_group)
    }

    pub fn remove_renderer_group(
        &mut self,
        renderer_group: &ArcRwLock<RendererGroupObject>,
    ) -> Option<ArcRwLock<RendererGroupObject>> {
        let ptr: *const RendererGroupObject = renderer_group.data_ptr();
        self.renderer_groups.remove(&ptr)
    }

    pub fn draw(
        &self,
        eye_position: &Vec3<f32>,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
    ) {
        for renderer_group in self.renderer_groups.values() {
            renderer_group
                .read()
                .draw(eye_position, projection_matrix, view_matrix);
        }
    }
}
