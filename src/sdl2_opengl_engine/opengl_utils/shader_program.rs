use std::string::FromUtf8Error;

use gl::types::GLuint;

use super::{
    shader::Shader,
    shader_input::{ShaderAttribute, ShaderInput, ShaderUniform},
};

#[derive(Debug)]
pub enum ShaderProgramError {
    LinkErrorToString(FromUtf8Error),
    LinkError { error_msg: String },
    ValidateError { error_msg: String },
}

pub struct ShaderProgram {
    attached_shaders: Vec<Shader>,
    program_id: GLuint,
    attributes: Vec<Result<ShaderAttribute, FromUtf8Error>>,
    uniforms: Vec<Result<ShaderUniform, FromUtf8Error>>,
}

impl ShaderProgram {
    pub fn new() -> Self {
        let program_id = unsafe { gl::CreateProgram() };

        Self {
            program_id,
            attached_shaders: Vec::new(),
            attributes: Vec::new(),
            uniforms: Vec::new(),
        }
    }

    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.program_id) }
    }

    pub fn attach_shader(&mut self, shader: Shader) {
        unsafe { gl::AttachShader(self.program_id, shader.shader_id) }
        self.attached_shaders.push(shader);
    }

    unsafe fn get_program_info_log(&self) -> Result<String, ShaderProgramError> {
        let mut info_log_length = 0;
        let mut actual_info_log_length: i32 = 0;

        gl::GetProgramiv(self.program_id, gl::INFO_LOG_LENGTH, &mut info_log_length);

        if info_log_length > 0 {
            let mut error_log = vec![0u8; info_log_length as usize + 1];
            gl::GetProgramInfoLog(
                self.program_id,
                info_log_length,
                &mut actual_info_log_length,
                error_log.as_mut_ptr() as *mut i8,
            );
            error_log.resize(actual_info_log_length as usize, 0);

            let error_msg = String::from_utf8(error_log)
                .map_err(|e| ShaderProgramError::LinkErrorToString(e))?;

            Ok(error_msg)
        } else {
            Ok("".to_string())
        }
    }

    pub fn link_program(&mut self) -> Result<(), ShaderProgramError> {
        unsafe {
            gl::LinkProgram(self.program_id);

            let mut link_status = 0;
            gl::GetProgramiv(self.program_id, gl::LINK_STATUS, &mut link_status);

            let info_log = self.get_program_info_log()?;

            if info_log.len() > 0 {
                Err(ShaderProgramError::LinkError {
                    error_msg: info_log,
                })
            } else {
                self.gather_attributes();
                self.gather_uniforms();

                Ok(())
            }
        }
    }

    pub fn validate_program(&self) -> Result<(), ShaderProgramError> {
        unsafe {
            gl::ValidateProgram(self.program_id);

            let mut validate_status = 0;
            gl::GetProgramiv(self.program_id, gl::VALIDATE_STATUS, &mut validate_status);

            if validate_status == 0 {
                let info_log = self.get_program_info_log()?;

                if info_log.len() > 0 {
                    Err(ShaderProgramError::ValidateError {
                        error_msg: info_log,
                    })
                } else {
                    Err(ShaderProgramError::ValidateError {
                        error_msg: "Unknown validation error".to_string(),
                    })
                }
            } else {
                Ok(())
            }
        }
    }

    pub fn get_attribute_by_name(&self, name: &str) -> Option<ShaderAttribute> {
        let mut ret = None;

        for attribute in self.attributes.iter() {
            match attribute {
                Ok(attribute) => {
                    if attribute.0.name == name {
                        ret = Some(attribute.clone())
                    }
                }
                Err(_) => {}
            }
        }

        ret
    }

    pub fn get_uniform_by_name(&self, name: &str) -> Option<ShaderUniform> {
        let mut ret = None;

        for uniform in self.uniforms.iter() {
            match uniform {
                Ok(uniform) => {
                    if uniform.0.name == name {
                        ret = Some(uniform.clone())
                    }
                }
                Err(_) => {}
            }
        }

        ret
    }

    unsafe fn gather_attributes(&mut self) {
        let mut attributes_count = 0;

        gl::GetProgramiv(
            self.program_id,
            gl::ACTIVE_ATTRIBUTES,
            &mut attributes_count,
        );

        let mut max_length = 0;
        gl::GetProgramiv(
            self.program_id,
            gl::ACTIVE_ATTRIBUTE_MAX_LENGTH,
            &mut max_length,
        );

        let mut data_type = 0;
        let mut array_size = 0;

        let mut length = 0;

        for i in 0..attributes_count as GLuint {
            let mut attribute_name = vec![0u8; max_length as usize + 1];
            gl::GetActiveAttrib(
                self.program_id,
                i,
                max_length,
                &mut length,
                &mut array_size,
                &mut data_type,
                attribute_name.as_mut_ptr() as *mut i8,
            );
            attribute_name.resize(length as usize, 0);

            let location =
                gl::GetAttribLocation(self.program_id, attribute_name.as_ptr() as *const i8);

            match String::from_utf8(attribute_name) {
                Ok(attribute_name) => self.attributes.push(Ok(ShaderAttribute::new(ShaderInput {
                    name: attribute_name,
                    location,
                    data_type,
                    array_size,
                }))),
                Err(e) => self.attributes.push(Err(e)),
            }
        }
    }

    unsafe fn gather_uniforms(&mut self) {
        let mut uniforms_count = 0;

        gl::GetProgramiv(self.program_id, gl::ACTIVE_UNIFORMS, &mut uniforms_count);

        let mut max_length = 0;
        gl::GetProgramiv(
            self.program_id,
            gl::ACTIVE_UNIFORM_MAX_LENGTH,
            &mut max_length,
        );

        let mut data_type = 0;
        let mut array_size = 0;

        let mut length = 0;

        for i in 0..uniforms_count as GLuint {
            let mut uniform_name = vec![0u8; max_length as usize + 1];

            gl::GetActiveUniform(
                self.program_id,
                i,
                max_length,
                &mut length,
                &mut array_size,
                &mut data_type,
                uniform_name.as_mut_ptr() as *mut i8,
            );
            uniform_name.resize(length as usize, 0);

            let location =
                gl::GetUniformLocation(self.program_id, uniform_name.as_ptr() as *const i8);

            match String::from_utf8(uniform_name) {
                Ok(uniform_name) => self.uniforms.push(Ok(ShaderUniform::new(ShaderInput {
                    name: uniform_name,
                    location,
                    data_type,
                    array_size,
                }))),
                Err(e) => self.uniforms.push(Err(e)),
            }
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_id);
        }
    }
}
