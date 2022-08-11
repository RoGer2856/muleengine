use std::{ffi::c_void, mem::size_of, ptr::null};

use gl::types::{GLenum, GLuint};

pub enum PrimitiveMode {
    Points,
    LineStrip,
    LineLoop,
    Lines,
    LineStripAdjacency,
    LinesAdjacency,
    TriangleStrip,
    TriangleFan,
    Triangles,
    TriangleStripAdjacency,
    TrianglesAdjacency,
    Patches,
}

impl PrimitiveMode {
    fn to_gl_enum(&self) -> GLenum {
        match self {
            PrimitiveMode::Points => gl::POINTS,
            PrimitiveMode::LineStrip => gl::LINE_STRIP,
            PrimitiveMode::LineLoop => gl::LINE_LOOP,
            PrimitiveMode::Lines => gl::LINES,
            PrimitiveMode::LineStripAdjacency => gl::LINE_STRIP_ADJACENCY,
            PrimitiveMode::LinesAdjacency => gl::LINES_ADJACENCY,
            PrimitiveMode::TriangleStrip => gl::TRIANGLE_STRIP,
            PrimitiveMode::TriangleFan => gl::TRIANGLE_FAN,
            PrimitiveMode::Triangles => gl::TRIANGLES,
            PrimitiveMode::TriangleStripAdjacency => gl::TRIANGLE_STRIP_ADJACENCY,
            PrimitiveMode::TrianglesAdjacency => gl::TRIANGLES_ADJACENCY,
            PrimitiveMode::Patches => gl::PATCHES,
        }
    }
}

pub struct IndexBufferObject {
    pub(super) buffer_id: GLuint,
    number_of_elements: usize,
    primitive_mode: GLenum,
}

impl IndexBufferObject {
    pub fn new(
        data_pointer: *const u32,
        number_of_elements: usize,
        primitive_mode: PrimitiveMode,
    ) -> Self {
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
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        };
        Self {
            buffer_id,
            number_of_elements,
            primitive_mode: primitive_mode.to_gl_enum(),
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::DrawElements(
                self.primitive_mode,
                self.number_of_elements as i32,
                gl::UNSIGNED_INT,
                null(),
            );
        }
    }

    pub fn draw_elements(&self, number_of_elements: usize) {
        unsafe {
            gl::DrawElements(
                self.primitive_mode,
                number_of_elements as i32,
                gl::UNSIGNED_INT,
                null(),
            );
        }
    }

    pub fn draw_instances(&self, number_of_instances: usize) {
        unsafe {
            gl::DrawElementsInstanced(
                self.primitive_mode,
                self.number_of_elements as i32,
                gl::UNSIGNED_INT,
                null(),
                number_of_instances as i32,
            );
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
