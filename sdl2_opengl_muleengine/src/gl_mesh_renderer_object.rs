use std::sync::Arc;

use muleengine::renderer::{RendererObject, RendererTransform};
use parking_lot::RwLock;

use crate::gl_drawable_mesh::GLDrawableMesh;

pub struct GLMeshRendererObject {
    pub(super) transform: Arc<RwLock<dyn RendererTransform>>,
    pub(super) gl_drawable_mesh: GLDrawableMesh,
}

impl RendererObject for GLMeshRendererObject {}
