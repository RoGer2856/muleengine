use std::collections::HashMap;
use std::sync::Arc;

use muleengine::asset_reader::AssetReader;

use crate::gl_shader_program::{GLShaderProgram, GLShaderProgramError};

use super::gl_mesh_shader_program::GLMeshShaderProgram;

#[derive(Clone)]
pub struct GLShaderProgramContainer {
    shader_programs: HashMap<String, Arc<GLShaderProgram>>,
    mesh_shader_programs: HashMap<String, Arc<GLMeshShaderProgram>>,
}

impl Default for GLShaderProgramContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl GLShaderProgramContainer {
    pub fn new() -> Self {
        Self {
            shader_programs: HashMap::new(),
            mesh_shader_programs: HashMap::new(),
        }
    }

    pub fn get_shader_program(
        &mut self,
        shader_basepath: &str,
        asset_reader: &AssetReader,
    ) -> Result<Arc<GLShaderProgram>, GLShaderProgramError> {
        if let Some(shader_program_mut) = self.shader_programs.get_mut(shader_basepath) {
            Ok(shader_program_mut.clone())
        } else {
            let gl_shader_program = Arc::new(GLShaderProgram::new(
                shader_basepath.to_string(),
                asset_reader,
            )?);
            self.shader_programs
                .insert(shader_basepath.to_string(), gl_shader_program.clone());

            Ok(gl_shader_program)
        }
    }

    pub fn get_mesh_shader_program(
        &mut self,
        shader_basepath: &str,
        asset_reader: &AssetReader,
    ) -> Result<Arc<GLMeshShaderProgram>, GLShaderProgramError> {
        if let Some(shader_program_mut) = self.mesh_shader_programs.get_mut(shader_basepath) {
            Ok(shader_program_mut.clone())
        } else {
            let gl_mesh_shader_program = Arc::new(GLMeshShaderProgram::new(
                self.get_shader_program(shader_basepath, asset_reader)?,
            )?);
            self.mesh_shader_programs
                .insert(shader_basepath.to_string(), gl_mesh_shader_program.clone());

            Ok(gl_mesh_shader_program)
        }
    }
}
