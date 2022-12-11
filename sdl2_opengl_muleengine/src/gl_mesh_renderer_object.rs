use muleengine::{prelude::ArcRwLock, renderer::RendererTransform};

use crate::gl_drawable_mesh::GLDrawableMesh;

pub struct GLMeshRendererObject {
    pub(super) transform: ArcRwLock<dyn RendererTransform>,
    pub(super) gl_drawable_mesh: GLDrawableMesh,
}
