use std::{io::Read, sync::Arc};

use muleengine::asset_reader::AssetReader;

use super::opengl_utils::{
    shader::{Shader, ShaderCreationError, ShaderType},
    shader_program::{ShaderProgram, ShaderProgramError},
};

pub struct GLShaderProgram {
    shader_base_path: String,
    pub(super) shader_program: ShaderProgram,
}

pub struct RendererShaderObject {
    gl_shader_program: Arc<GLShaderProgram>,
}

#[derive(Debug)]
pub enum GLShaderProgramError {
    AssetNotFoundError {
        path: String,
    },
    AssetReadError {
        error: std::io::Error,
        path: String,
    },
    ShaderCreationError {
        shader_type: ShaderType,
        shader_path: String,
        shader_creation_error: ShaderCreationError,
    },
    ShaderProgramError(ShaderProgramError),
}

impl GLShaderProgram {
    pub fn new(
        shader_base_path: String,
        asset_reader: &AssetReader,
    ) -> Result<Self, GLShaderProgramError> {
        let vertex_shader_path = shader_base_path.clone() + ".vert";
        let fragment_shader_path = shader_base_path.clone() + ".frag";

        let mut vertex_shader_source = String::new();
        asset_reader
            .get_reader(&vertex_shader_path)
            .ok_or(GLShaderProgramError::AssetNotFoundError {
                path: vertex_shader_path.clone(),
            })?
            .read_to_string(&mut vertex_shader_source)
            .map_err(|e| GLShaderProgramError::AssetReadError {
                error: e,
                path: vertex_shader_source.clone(),
            })?;
        let mut fragment_shader_source = String::new();
        asset_reader
            .get_reader(&fragment_shader_path)
            .ok_or(GLShaderProgramError::AssetNotFoundError {
                path: fragment_shader_path.clone(),
            })?
            .read_to_string(&mut fragment_shader_source)
            .map_err(|e| GLShaderProgramError::AssetReadError {
                error: e,
                path: fragment_shader_source.clone(),
            })?;

        let vertex_shader =
            Shader::new(ShaderType::Vertex, &vertex_shader_source).map_err(|e| {
                GLShaderProgramError::ShaderCreationError {
                    shader_type: ShaderType::Vertex,
                    shader_path: vertex_shader_path,
                    shader_creation_error: e,
                }
            })?;

        let fragment_shader =
            Shader::new(ShaderType::Fragment, &fragment_shader_source).map_err(|e| {
                GLShaderProgramError::ShaderCreationError {
                    shader_type: ShaderType::Fragment,
                    shader_path: fragment_shader_path,
                    shader_creation_error: e,
                }
            })?;

        let mut shader_program = ShaderProgram::new();
        shader_program.attach_shader(vertex_shader);
        shader_program.attach_shader(fragment_shader);
        shader_program
            .link_program()
            .map_err(GLShaderProgramError::ShaderProgramError)?;

        Ok(Self {
            shader_base_path,
            shader_program,
        })
    }

    pub fn get_shader_base_path(&self) -> &String {
        &self.shader_base_path
    }
}

impl RendererShaderObject {
    pub fn new(gl_shader_program: Arc<GLShaderProgram>) -> Self {
        Self { gl_shader_program }
    }

    pub fn gl_shader_program(&self) -> &Arc<GLShaderProgram> {
        &self.gl_shader_program
    }
}
