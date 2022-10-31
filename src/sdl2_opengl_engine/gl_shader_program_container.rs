use std::collections::HashMap;
use std::sync::Arc;

use crate::muleengine::asset_reader::AssetReader;

use super::gl_mesh_shader_program::{GLMeshShaderProgram, GLMeshShaderProgramError};

#[derive(Clone)]
pub struct GLShaderProgramContainer {
    mesh_shader_programs: HashMap<String, Arc<GLMeshShaderProgram>>,
}

impl GLShaderProgramContainer {
    pub fn new() -> Self {
        Self {
            mesh_shader_programs: HashMap::new(),
        }
    }

    pub fn get_mesh_shader_program(
        &mut self,
        shader_basepath: &str,
        asset_reader: &AssetReader,
    ) -> Result<Arc<GLMeshShaderProgram>, GLMeshShaderProgramError> {
        if let Some(scene_mut) = self.mesh_shader_programs.get_mut(shader_basepath) {
            Ok(scene_mut.clone())
        } else {
            let gl_mesh_shader_program = Arc::new(GLMeshShaderProgram::new(
                shader_basepath.to_string(),
                asset_reader,
            )?);
            self.mesh_shader_programs
                .insert(shader_basepath.to_string(), gl_mesh_shader_program.clone());

            Ok(gl_mesh_shader_program)
        }
    }
}
