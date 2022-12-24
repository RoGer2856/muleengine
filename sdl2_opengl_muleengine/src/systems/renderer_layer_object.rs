use std::collections::BTreeMap;

use muleengine::prelude::ArcRwLock;
use vek::Mat4;

use super::{gl_camera::GLCamera, renderer_group_object::RendererGroupObject};

pub(crate) struct RendererLayerObject {
    camera: ArcRwLock<GLCamera>,
    renderer_groups: BTreeMap<*const RendererGroupObject, ArcRwLock<RendererGroupObject>>,
}

impl RendererLayerObject {
    pub fn new(camera: ArcRwLock<GLCamera>) -> Self {
        Self {
            camera,
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

    pub fn draw(&self, projection_matrix: &Mat4<f32>) {
        let camera = self.camera.read();

        let view_matrix = camera.compute_view_matrix();

        for renderer_group in self.renderer_groups.values() {
            renderer_group
                .read()
                .draw(&camera.transform.position, projection_matrix, &view_matrix);
        }
    }
}
