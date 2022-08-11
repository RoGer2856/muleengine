use gl::types::{GLenum, GLint, GLuint};

#[derive(Debug, Clone)]
pub struct ShaderInput {
    pub name: String,
    pub location: GLuint,
    pub data_type: GLenum,
    pub array_size: GLint,
}

#[derive(Debug, Clone)]
pub struct ShaderAttribute(pub(super) ShaderInput);

#[derive(Debug, Clone)]
pub struct ShaderUniform(pub(super) ShaderInput);

impl ShaderAttribute {
    pub fn new(shader_input: ShaderInput) -> Self {
        Self(shader_input)
    }
}

impl ShaderUniform {
    pub fn new(shader_input: ShaderInput) -> Self {
        Self(shader_input)
    }
}
