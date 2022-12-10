use muleengine::{containers::object_pool::ObjectPoolIndex, renderer::RendererMesh};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererMeshImpl(pub(super) ObjectPoolIndex);

impl RendererMesh for RendererMeshImpl {}
