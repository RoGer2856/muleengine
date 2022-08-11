use gl::types::{GLenum, GLint};

#[derive(Debug, Clone)]
pub struct ShaderInput {
    pub name: String,
    pub location: GLint,
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

    pub fn send_uniform_1f(&self, v0: f32) {
        unsafe {
            gl::Uniform1f(self.0.location, v0);
        }
    }

    pub fn send_uniform_2f(&self, v0: f32, v1: f32) {
        unsafe {
            gl::Uniform2f(self.0.location, v0, v1);
        }
    }

    pub fn send_uniform_3f(&self, v0: f32, v1: f32, v2: f32) {
        unsafe {
            gl::Uniform3f(self.0.location, v0, v1, v2);
        }
    }

    pub fn send_uniform_4f(&self, v0: f32, v1: f32, v2: f32, v3: f32) {
        unsafe {
            gl::Uniform4f(self.0.location, v0, v1, v2, v3);
        }
    }

    pub fn send_uniform_1i(&self, v0: i32) {
        unsafe {
            gl::Uniform1i(self.0.location, v0);
        }
    }

    pub fn send_uniform_2i(&self, v0: i32, v1: i32) {
        unsafe {
            gl::Uniform2i(self.0.location, v0, v1);
        }
    }

    pub fn send_uniform_3i(&self, v0: i32, v1: i32, v2: i32) {
        unsafe {
            gl::Uniform3i(self.0.location, v0, v1, v2);
        }
    }

    pub fn send_uniform_4i(&self, v0: i32, v1: i32, v2: i32, v3: i32) {
        unsafe {
            gl::Uniform4i(self.0.location, v0, v1, v2, v3);
        }
    }

    pub fn send_uniform_1ui(&self, v0: u32) {
        unsafe {
            gl::Uniform1ui(self.0.location, v0);
        }
    }

    pub fn send_uniform_2ui(&self, v0: u32, v1: u32) {
        unsafe {
            gl::Uniform2ui(self.0.location, v0, v1);
        }
    }

    pub fn send_uniform_3ui(&self, v0: u32, v1: u32, v2: u32) {
        unsafe {
            gl::Uniform3ui(self.0.location, v0, v1, v2);
        }
    }

    pub fn send_uniform_4ui(&self, v0: u32, v1: u32, v2: u32, v3: u32) {
        unsafe {
            gl::Uniform4ui(self.0.location, v0, v1, v2, v3);
        }
    }

    pub fn send_uniform_1fv(&self, array: *const f32, count: usize) {
        unsafe {
            gl::Uniform1fv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_2fv(&self, array: *const f32, count: usize) {
        unsafe {
            gl::Uniform2fv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_3fv(&self, array: *const f32, count: usize) {
        unsafe {
            gl::Uniform3fv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_4fv(&self, array: *const f32, count: usize) {
        unsafe {
            gl::Uniform4fv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_1iv(&self, array: *const i32, count: usize) {
        unsafe {
            gl::Uniform1iv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_2iv(&self, array: *const i32, count: usize) {
        unsafe {
            gl::Uniform2iv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_3iv(&self, array: *const i32, count: usize) {
        unsafe {
            gl::Uniform3iv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_4iv(&self, array: *const i32, count: usize) {
        unsafe {
            gl::Uniform4iv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_1uiv(&self, array: *const u32, count: usize) {
        unsafe {
            gl::Uniform1uiv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_2uiv(&self, array: *const u32, count: usize) {
        unsafe {
            gl::Uniform2uiv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_3uiv(&self, array: *const u32, count: usize) {
        unsafe {
            gl::Uniform3uiv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_4uiv(&self, array: *const u32, count: usize) {
        unsafe {
            gl::Uniform4uiv(self.0.location, count as i32, array);
        }
    }

    pub fn send_uniform_matrix_2fv(&self, array: *const f32, count: usize) {
        unsafe {
            gl::UniformMatrix2fv(self.0.location, count as i32, gl::FALSE, array);
        }
    }

    pub fn send_uniform_matrix_3fv(&self, array: *const f32, count: usize) {
        unsafe {
            gl::UniformMatrix3fv(self.0.location, count as i32, gl::FALSE, array);
        }
    }

    pub fn send_uniform_matrix_4fv(&self, array: *const f32, count: usize) {
        unsafe {
            gl::UniformMatrix4fv(self.0.location, count as i32, gl::FALSE, array);
        }
    }
}
