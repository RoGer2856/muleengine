use muleengine::{
    containers::object_pool::ObjectPoolIndex,
    renderer::{RendererMaterial, RendererMesh, RendererShader},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererMaterialImpl(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererShaderImpl(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererMeshImpl(pub(super) ObjectPoolIndex);

impl RendererMaterial for RendererMaterialImpl {}
impl RendererShader for RendererShaderImpl {}
impl RendererMesh for RendererMeshImpl {}
