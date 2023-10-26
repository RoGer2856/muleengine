use muleengine::{
    bytifex_utils::containers::object_pool::ObjectPoolIndex,
    renderer::{
        RendererCamera, RendererGroup, RendererLayer, RendererMaterial, RendererMesh,
        RendererObject, RendererShader, RendererTransform,
    },
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererLayerIndex(pub(super) ObjectPoolIndex);
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererCameraIndex(pub(super) ObjectPoolIndex);

impl RendererLayer for RendererLayerIndex {}
impl RendererGroup for RendererGroupIndex {}
impl RendererTransform for RendererTransformIndex {}
impl RendererMaterial for RendererMaterialIndex {}
impl RendererShader for RendererShaderIndex {}
impl RendererMesh for RendererMeshIndex {}
impl RendererObject for RendererObjectIndex {}
impl RendererCamera for RendererCameraIndex {}
