#[cfg(test)]
mod tests;

pub mod renderer_impl;
mod renderer_objects;
pub mod renderer_pipeline_step;
pub mod renderer_pipeline_step_impl;
pub mod renderer_system;

pub use renderer_objects::renderer_camera::*;
pub use renderer_objects::renderer_group::*;
pub use renderer_objects::renderer_layer::*;
pub use renderer_objects::renderer_material::*;
pub use renderer_objects::renderer_mesh::*;
pub use renderer_objects::renderer_object::*;
pub use renderer_objects::renderer_shader::*;
pub use renderer_objects::renderer_transform::*;

#[derive(Debug)]
pub enum RendererError {
    InvalidRendererCameraHandler(RendererCameraHandler),
    InvalidRendererTransformHandler(RendererTransformHandler),
    InvalidRendererMaterialHandler(RendererMaterialHandler),
    InvalidRendererShaderHandler(RendererShaderHandler),
    InvalidRendererMeshHandler(RendererMeshHandler),
    InvalidRendererObjectHandler(RendererObjectHandler),
    InvalidRendererLayerHandler(RendererLayerHandler),
    InvalidRendererGroupHandler(RendererGroupHandler),
    RendererImplError(String),
    RendererSystemDropped,
}
