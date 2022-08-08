use std::{ffi::c_void, mem::size_of};

use gl::types::GLuint;

pub struct VertexBufferObject {
    buffer_id: GLuint,
    size_of_element: usize,
    data_pointer: *const c_void,
}

impl VertexBufferObject {
    pub fn new<ElementType>(data_pointer: *const ElementType, number_of_elements: usize) -> Self
    where
        ElementType: Sized,
    {
        let size_of_element = size_of::<ElementType>();
        let data_pointer = data_pointer as *const c_void;
        let mut buffer_id = 0;
        unsafe {
            gl::GenBuffers(1, &mut buffer_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of_element * number_of_elements) as isize,
                data_pointer,
                gl::STATIC_DRAW,
            );
        };
        Self {
            buffer_id,
            size_of_element,
            data_pointer,
        }
    }

    pub fn update_from_pointer(&mut self, element_offset: usize, number_of_elements: usize) {
        self.use_buffer();
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (self.size_of_element * element_offset) as isize,
                (self.size_of_element * number_of_elements) as isize,
                self.data_pointer.add(self.size_of_element * element_offset),
            );
        }
    }

    pub fn use_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer_id);
        }
    }

    pub fn unuse() {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

impl Drop for VertexBufferObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.buffer_id);
        }
    }
}
