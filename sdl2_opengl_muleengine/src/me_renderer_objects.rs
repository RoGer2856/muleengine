use muleengine::{
    containers::object_pool::ObjectPoolIndex,
    renderer::{RendererMaterial, RendererMesh, RendererShader, RendererTransform},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererTransformImpl(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererMaterialImpl(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererShaderImpl(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererMeshImpl(pub(super) ObjectPoolIndex);

impl RendererTransform for RendererTransformImpl {}
impl RendererMaterial for RendererMaterialImpl {}
impl RendererShader for RendererShaderImpl {}
impl RendererMesh for RendererMeshImpl {}
