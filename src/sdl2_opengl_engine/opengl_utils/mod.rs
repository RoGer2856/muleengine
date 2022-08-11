pub mod index_buffer_object;
pub mod shader;
pub mod shader_input;
pub mod shader_program;
pub mod vertex_array_object;
pub mod vertex_buffer_object;

#[derive(Debug)]
pub enum GlError {
    InvalidEnum,
    InvalidValue,
    InvalidOperation,
    StackOverflow,
    StackUnderflow,
    OutOfMemory,
    InvalidFramebufferOperation,
    Unknown(u32),
}

pub fn gl_get_error() -> Result<(), GlError> {
    let error = unsafe { gl::GetError() };

    match error {
        gl::NO_ERROR => Ok(()),
        gl::INVALID_ENUM => Err(GlError::InvalidEnum),
        gl::INVALID_VALUE => Err(GlError::InvalidValue),
        gl::INVALID_OPERATION => Err(GlError::InvalidOperation),
        gl::STACK_OVERFLOW => Err(GlError::StackOverflow),
        gl::STACK_UNDERFLOW => Err(GlError::StackUnderflow),
        gl::OUT_OF_MEMORY => Err(GlError::OutOfMemory),
        gl::INVALID_FRAMEBUFFER_OPERATION => Err(GlError::InvalidFramebufferOperation),
        e => Err(GlError::Unknown(e)),
    }
}
