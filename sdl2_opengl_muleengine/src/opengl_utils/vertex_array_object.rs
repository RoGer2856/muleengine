use std::marker::PhantomData;
use std::ptr::null;

use gl::types::GLuint;

use super::shader_input::ShaderAttribute;
use super::vertex_buffer_object::{DataCount, DataType};
use super::{index_buffer_object::IndexBufferObject, vertex_buffer_object::VertexBufferObject};

pub struct VertexArrayObjectInterface {
    _phantom: PhantomData<()>,
}

impl VertexArrayObjectInterface {
    pub fn bind_vbo_to_shader_attrib(&self, vbo: &VertexBufferObject, attrib: &ShaderAttribute) {
        self.bind_vbo_to_shader_attrib_array(vbo, attrib, 0);
    }

    pub fn bind_vbo_to_shader_attrib_array(
        &self,
        vbo: &VertexBufferObject,
        attrib: &ShaderAttribute,
        attrib_array_index: usize,
    ) {
        let data_count = match vbo.data_count {
            DataCount::Coords2 => 2,
            DataCount::Coords3 => 3,
            DataCount::Coords4 => 4,
            DataCount::Rgb => 3,
            DataCount::Rgba => 4,
            DataCount::Single => 1,
        };

        let data_type = match vbo.data_type {
            DataType::F32 => gl::FLOAT,
            DataType::F64 => gl::DOUBLE,

            DataType::I16 => gl::SHORT,
            DataType::I32 => gl::INT,
            DataType::I8 => gl::BYTE,

            DataType::U16 => gl::UNSIGNED_SHORT,
            DataType::U32 => gl::UNSIGNED_INT,
            DataType::U8 => gl::UNSIGNED_BYTE,
        };

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo.buffer_id);
            gl::EnableVertexAttribArray(attrib.0.location as u32 + attrib_array_index as u32);

            gl::VertexAttribPointer(
                attrib.0.location as u32 + attrib_array_index as u32,
                data_count,
                data_type,
                gl::FALSE,
                vbo.size_of_element as i32,
                null(),
            );
        }
    }

    pub fn use_index_buffer_object(&self, ibo: &IndexBufferObject) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo.buffer_id);
        }
    }
}

pub struct VertexArrayObject {
    vao_id: GLuint,
}

impl VertexArrayObject {
    pub fn new(setup_fn: impl FnOnce(VertexArrayObjectInterface)) -> Self {
        let vao_id = unsafe {
            let mut vao_id = 0;
            gl::GenVertexArrays(1, &mut vao_id);
            gl::BindVertexArray(vao_id);

            setup_fn(VertexArrayObjectInterface {
                _phantom: PhantomData,
            });

            gl::BindVertexArray(0);

            vao_id
        };

        Self { vao_id }
    }

    pub fn use_vao(&self, use_fn: impl FnOnce()) {
        unsafe {
            gl::BindVertexArray(self.vao_id);
        }

        use_fn();

        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao_id);
        }
    }
}
