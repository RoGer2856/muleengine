#[cfg(test)]
mod tests;

pub mod renderer_client;
mod renderer_command;
pub mod renderer_impl;
mod renderer_object;
pub mod renderer_system;

pub use renderer_object::renderer_group::*;
pub use renderer_object::renderer_material::*;
pub use renderer_object::renderer_mesh::*;
pub use renderer_object::renderer_object::*;
pub use renderer_object::renderer_shader::*;
pub use renderer_object::renderer_transform::*;

#[derive(Debug)]
pub enum RendererError {
    InvalidRendererTransformHandler(TransformHandler),
    InvalidRendererMaterialHandler(MaterialHandler),
    InvalidRendererShaderHandler(ShaderHandler),
    InvalidRendererMeshHandler(MeshHandler),
    InvalidRendererObjectHandler(RendererObjectHandler),
    InvalidRendererGroupHandler(RendererGroupHandler),
    RendererImplError(String),
}
