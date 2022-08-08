use std::{ffi::c_void, mem::size_of, ptr::null};

use gl::types::{GLenum, GLuint};

pub struct IndexBufferObject {
    buffer_id: GLuint,
    number_of_elements: usize,
}

impl IndexBufferObject {
    pub fn new(data_pointer: *const u32, number_of_elements: usize) -> Self {
        let mut buffer_id = 0;
        unsafe {
            gl::GenBuffers(1, &mut buffer_id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer_id);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (size_of::<u32>() * number_of_elements) as isize,
                data_pointer as *const c_void,
                gl::STATIC_DRAW,
            );
        };
        Self {
            buffer_id,
            number_of_elements,
        }
    }

    pub fn use_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.buffer_id);
        }
    }

    pub fn draw(&self, primitive_mode: GLenum) {
        unsafe {
            gl::DrawElements(
                primitive_mode,
                self.number_of_elements as i32,
                gl::UNSIGNED_INT,
                null::<c_void>(),
            );
        }
    }

    pub fn draw_elements(&self, primitive_mode: GLenum, number_of_elements: usize) {
        unsafe {
            gl::DrawElements(
                primitive_mode,
                number_of_elements as i32,
                gl::UNSIGNED_INT,
                null::<c_void>(),
            );
        }
    }

    pub fn draw_instances(&self, primitive_mode: GLenum, number_of_instances: usize) {
        unsafe {
            gl::DrawElementsInstanced(
                primitive_mode,
                self.number_of_elements as i32,
                gl::UNSIGNED_INT,
                null::<c_void>(),
                number_of_instances as i32,
            );
        }
    }

    pub fn unuse() {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }
}

impl Drop for IndexBufferObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.buffer_id);
        }
    }
}
