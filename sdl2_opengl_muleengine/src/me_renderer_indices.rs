use muleengine::{
    containers::object_pool::ObjectPoolIndex,
    renderer::{
        RendererGroup, RendererMaterial, RendererMesh, RendererObject, RendererShader,
        RendererTransform,
    },
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererGroupIndex(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererTransformIndex(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererMaterialIndex(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererShaderIndex(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererMeshIndex(pub(super) ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RendererObjectIndex {
    Mesh(ObjectPoolIndex),
}

impl RendererGroup for RendererGroupIndex {}
impl RendererTransform for RendererTransformIndex {}
impl RendererMaterial for RendererMaterialIndex {}
impl RendererShader for RendererShaderIndex {}
impl RendererMesh for RendererMeshIndex {}
impl RendererObject for RendererObjectIndex {}
