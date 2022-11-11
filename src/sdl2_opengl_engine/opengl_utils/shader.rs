use std::string::FromUtf8Error;

use gl::types::GLuint;

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Geometry,
    Fragment,
}

impl ShaderType {
    pub fn to_gl_shader_type(&self) -> u32 {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

#[derive(Debug)]
pub enum ShaderCreationError {
    CompilationErrorToString(FromUtf8Error),
    CompilationError { error_msg: String },
}

pub struct Shader {
    pub(super) shader_id: GLuint,
}

impl Shader {
    pub fn new(shader_type: ShaderType, source_code: &str) -> Result<Self, ShaderCreationError> {
        let shader_id = unsafe {
            let shader_id = gl::CreateShader(shader_type.to_gl_shader_type());
            let lengths = [source_code.len()];
            gl::ShaderSource(
                shader_id,
                1,
                &(source_code.as_ptr() as *const i8),
                lengths.as_ptr() as *const i32,
            );

            gl::CompileShader(shader_id);

            let mut compile_status = 0;
            gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut compile_status);

            let mut info_log_length = 0;
            let mut actual_info_log_length: i32 = 0;

            gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut info_log_length);

            if info_log_length > 0 {
                let mut error_log = vec![0u8; info_log_length as usize + 1];
                gl::GetShaderInfoLog(
                    shader_id,
                    info_log_length,
                    &mut actual_info_log_length,
                    error_log.as_mut_ptr() as *mut i8,
                );
                error_log.resize(actual_info_log_length as usize, 0);

                let error_msg = String::from_utf8(error_log)
                    .map_err(ShaderCreationError::CompilationErrorToString)?;

                Err(ShaderCreationError::CompilationError { error_msg })?
            }

            shader_id
        };
        Ok(Self { shader_id })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.shader_id);
        }
    }
}
