use std::sync::Arc;

use vek::Mat4;

use crate::muleengine::{drawable_object::DrawableObject, mesh::Mesh};

use super::{
    gl_mesh_shader_program::GLMeshShaderProgram,
    opengl_utils::{
        index_buffer_object::{IndexBufferObject, PrimitiveMode},
        vertex_array_object::VertexArrayObject,
        vertex_buffer_object::{DataCount, DataType, VertexBufferObject},
    },
};

pub struct GLMesh {
    _mesh: Arc<Mesh>,

    index_buffer_object: IndexBufferObject,
    positions_vbo: VertexBufferObject,
}

pub struct GLDrawableMesh {
    gl_mesh: Arc<GLMesh>,
    vertex_array_object: VertexArrayObject,
    gl_mesh_shader_program: Arc<GLMeshShaderProgram>,
}

impl DrawableObject for GLDrawableMesh {
    fn render(
        &self,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
        object_matrix: &Mat4<f32>,
    ) {
        GLDrawableMesh::render(&self, projection_matrix, view_matrix, object_matrix);
    }
}

impl GLMesh {
    pub fn new(mesh: Arc<Mesh>) -> Self {
        let index_buffer_object = IndexBufferObject::new(
            mesh.get_faces().as_ptr(),
            mesh.get_faces().len(),
            PrimitiveMode::Triangles,
        );
        let positions_vbo = VertexBufferObject::new(
            mesh.get_positions().as_ptr(),
            mesh.get_positions().len(),
            DataType::F32,
            DataCount::Coords3,
        );

        Self {
            _mesh: mesh,

            index_buffer_object,
            positions_vbo,
        }
    }
}

impl GLDrawableMesh {
    pub fn new(gl_mesh: Arc<GLMesh>, gl_mesh_shader_program: Arc<GLMeshShaderProgram>) -> Self {
        let vertex_array_object = VertexArrayObject::new(|vao_interface| {
            vao_interface.use_index_buffer_object(&gl_mesh.index_buffer_object);

            vao_interface.use_vertex_buffer_object(
                &gl_mesh.positions_vbo,
                &gl_mesh_shader_program.attributes.position,
            );
        });

        Self {
            gl_mesh,
            vertex_array_object,
            gl_mesh_shader_program,
        }
    }

    pub fn render(
        &self,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
        object_matrix: &Mat4<f32>,
    ) {
        self.gl_mesh_shader_program.shader_program.use_program();

        self.gl_mesh_shader_program
            .uniforms
            .projection_matrix
            .send_uniform_matrix_4fv(projection_matrix.as_col_ptr(), 1);

        self.gl_mesh_shader_program
            .uniforms
            .object_matrix
            .send_uniform_matrix_4fv(object_matrix.as_col_ptr(), 1);
        self.gl_mesh_shader_program
            .uniforms
            .view_matrix
            .send_uniform_matrix_4fv(view_matrix.as_col_ptr(), 1);

        self.vertex_array_object.use_vao();
        self.gl_mesh.index_buffer_object.draw();
    }
}
