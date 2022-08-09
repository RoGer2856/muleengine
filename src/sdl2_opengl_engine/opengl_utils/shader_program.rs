use std::string::FromUtf8Error;

use gl::types::GLuint;

use super::shader::Shader;

#[derive(Debug)]
pub enum ShaderProgramError {
    LinkErrorToString(FromUtf8Error),
    LinkError { error_msg: String },
    ValidateError { error_msg: String },
}

pub struct ShaderProgram {
    attached_shaders: Vec<Shader>,
    program_id: GLuint,
}

impl ShaderProgram {
    pub fn new() -> Self {
        let program_id = unsafe { gl::CreateProgram() };

        Self {
            program_id,
            attached_shaders: Vec::new(),
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
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_id);
        }
    }
}
