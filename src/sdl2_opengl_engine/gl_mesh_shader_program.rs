use std::io::Read;

use crate::muleengine::assets_reader::AssetsReader;

use super::opengl_utils::{
    shader::{Shader, ShaderCreationError, ShaderType},
    shader_input::{ShaderAttribute, ShaderUniform},
    shader_program::{ShaderProgram, ShaderProgramError},
};

pub(super) struct Attributes {
    pub(super) position: Option<ShaderAttribute>,
    pub(super) normal: Option<ShaderAttribute>,
    pub(super) tangent: Option<ShaderAttribute>,
    pub(super) uv_channels: Option<ShaderAttribute>,
    pub(super) bone_ids: Option<ShaderAttribute>,
    pub(super) bone_weights: Option<ShaderAttribute>,
}

pub(super) struct Uniforms {
    pub(super) projection_matrix: Option<ShaderUniform>,
    pub(super) object_matrix: Option<ShaderUniform>,
    pub(super) view_matrix: Option<ShaderUniform>,
}

pub struct GLMeshShaderProgram {
    shader_base_path: String,
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
    UniformNotFound { uniform_name: String },
}

impl GLMeshShaderProgram {
    pub fn new(
        shader_base_path: String,
        assets_reader: &AssetsReader,
    ) -> Result<Self, GLMeshShaderProgramError> {
        let vertex_shader_path = shader_base_path.clone() + ".vert";
        let fragment_shader_path = shader_base_path.clone() + ".frag";

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
            position: shader_program.get_attribute_by_name("position"),
            normal: shader_program.get_attribute_by_name("normal"),
            tangent: shader_program.get_attribute_by_name("tangent"),
            uv_channels: shader_program.get_attribute_by_name("uvChannels"),
            bone_ids: shader_program.get_attribute_by_name("boneIds"),
            bone_weights: shader_program.get_attribute_by_name("boneWeights"),
        };

        let uniforms = Uniforms {
            projection_matrix: shader_program.get_uniform_by_name("projectionMatrix"),
            object_matrix: shader_program.get_uniform_by_name("objectMatrix"),
            view_matrix: shader_program.get_uniform_by_name("viewMatrix"),
        };

        Ok(Self {
            shader_base_path,
            shader_program,
            uniforms,
            attributes,
        })
    }

    pub fn get_shader_base_path(&self) -> &String {
        &self.shader_base_path
    }
}
