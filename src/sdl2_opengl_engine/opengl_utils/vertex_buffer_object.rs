use std::{ffi::c_void, mem::size_of};

use gl::types::GLuint;

pub enum DataType {
    F32,
    F64,

    U8,
    U16,
    U32,

    I8,
    I16,
    I32,
}

pub enum DataCount {
    Single,

    Coords2,
    Coords3,
    Coords4,

    Rgb,
    Rgba,
}

pub struct VertexBufferObject {
    pub(super) buffer_id: GLuint,
    pub(super) size_of_element: usize,
    pub(super) data_type: DataType,
    pub(super) data_count: DataCount,
    data_pointer: *const c_void,
}

impl VertexBufferObject {
    pub fn new<ElementType>(
        data_pointer: *const ElementType,
        number_of_elements: usize,
        data_type: DataType,
        data_count: DataCount,
    ) -> Self
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
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        };
        Self {
            buffer_id,
            size_of_element,
            data_type,
            data_count,
            data_pointer,
        }
    }

    pub fn update_from_pointer(&mut self, element_offset: usize, number_of_elements: usize) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer_id);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (self.size_of_element * element_offset) as isize,
                (self.size_of_element * number_of_elements) as isize,
                self.data_pointer.add(self.size_of_element * element_offset),
            );
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
