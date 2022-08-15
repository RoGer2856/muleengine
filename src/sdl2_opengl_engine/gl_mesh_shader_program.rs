use std::io::Read;

use crate::muleengine::assets_reader::AssetsReader;

use super::opengl_utils::{
    shader::{Shader, ShaderCreationError, ShaderType},
    shader_input::{ShaderAttribute, ShaderUniform},
    shader_program::{ShaderProgram, ShaderProgramError},
};

pub(super) struct Attributes {
    pub(super) position: ShaderAttribute,
}

pub(super) struct Uniforms {
    pub(super) projection_matrix: ShaderUniform,
    pub(super) object_matrix: ShaderUniform,
    pub(super) view_matrix: ShaderUniform,
}

pub struct GLMeshShaderProgram {
    pub(super) shader_program: ShaderProgram,
    pub(super) uniforms: Uniforms,
    pub(super) attributes: Attributes,
}

#[derive(Debug)]
pub enum GLMeshShaderProgramError {
    AssetNotFoundError { path: String },
    AssetReadError { error: std::io::Error, path: String },
    ShaderCreationError(ShaderCreationError),
    ShaderProgramError(ShaderProgramError),
    AttributeNotFound { attribute_name: String },
    UniformNotFound { uniform_name: String },
}

impl GLMeshShaderProgram {
    pub fn new(
        shader_base_path: String,
        assets_reader: &mut AssetsReader,
    ) -> Result<Self, GLMeshShaderProgramError> {
        let vertex_shader_path = shader_base_path.clone() + ".vert";
        let fragment_shader_path = shader_base_path + ".frag";

        let mut vertex_shader_source = String::new();
        assets_reader
            .get_reader(&vertex_shader_path)
            .ok_or(GLMeshShaderProgramError::AssetNotFoundError {
                path: vertex_shader_path,
            })?
            .read_to_string(&mut vertex_shader_source)
            .map_err(|e| GLMeshShaderProgramError::AssetReadError {
                error: e,
                path: vertex_shader_source.clone(),
            })?;
        let mut fragment_shader_source = String::new();
        assets_reader
            .get_reader(&fragment_shader_path)
            .ok_or(GLMeshShaderProgramError::AssetNotFoundError {
                path: fragment_shader_path,
            })?
            .read_to_string(&mut fragment_shader_source)
            .map_err(|e| GLMeshShaderProgramError::AssetReadError {
                error: e,
                path: fragment_shader_source.clone(),
            })?;

        let vertex_shader = Shader::new(ShaderType::Vertex, &vertex_shader_source)
            .map_err(|e| GLMeshShaderProgramError::ShaderCreationError(e))?;

        let fragment_shader = Shader::new(ShaderType::Fragment, &fragment_shader_source)
            .map_err(|e| GLMeshShaderProgramError::ShaderCreationError(e))?;

        let mut shader_program = ShaderProgram::new();
        shader_program.attach_shader(vertex_shader);
        shader_program.attach_shader(fragment_shader);
        shader_program
            .link_program()
            .map_err(|e| GLMeshShaderProgramError::ShaderProgramError(e))?;

        let attributes = Attributes {
            position: shader_program.get_attribute_by_name("position").ok_or(
                GLMeshShaderProgramError::AttributeNotFound {
                    attribute_name: "position".to_string(),
                },
            )?,
        };

        let uniforms = Uniforms {
            projection_matrix: shader_program
                .get_uniform_by_name("projectionMatrix")
                .ok_or(GLMeshShaderProgramError::UniformNotFound {
                    uniform_name: "projectionMatrix".to_string(),
                })?,
            object_matrix: shader_program.get_uniform_by_name("objectMatrix").ok_or(
                GLMeshShaderProgramError::UniformNotFound {
                    uniform_name: "objectMatrix".to_string(),
                },
            )?,
            view_matrix: shader_program.get_uniform_by_name("viewMatrix").ok_or(
                GLMeshShaderProgramError::UniformNotFound {
                    uniform_name: "viewMatrix".to_string(),
                },
            )?,
        };

        Ok(Self {
            shader_program,
            uniforms,
            attributes,
        })
    }
}
