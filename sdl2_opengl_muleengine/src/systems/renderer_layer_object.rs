use std::collections::BTreeMap;

use muleengine::prelude::ArcRwLock;

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
}
