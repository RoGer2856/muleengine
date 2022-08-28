use std::collections::HashMap;
use std::sync::Arc;

use crate::muleengine::mesh::Mesh;

use super::gl_mesh::{GLDrawableMesh, GLMesh};
use super::gl_mesh_shader_program::GLMeshShaderProgram;
use super::gl_texture_container::GLTextureContainer;

pub struct GLMeshContainer {
    drawable_meshes: HashMap<
        *const Mesh,
        (
            Arc<GLMesh>,
            HashMap<*const GLMeshShaderProgram, Arc<GLDrawableMesh>>,
        ),
    >,
}

impl GLMeshContainer {
    pub fn new() -> Self {
        Self {
            drawable_meshes: HashMap::new(),
        }
    }

    pub fn get_drawable_mesh(
        &mut self,
        gl_mesh_shader_program: Arc<GLMeshShaderProgram>,
        mesh: Arc<Mesh>,
        gl_texture_container: &mut GLTextureContainer,
    ) -> Arc<GLDrawableMesh> {
        let inner_container = self.drawable_meshes.entry(&*mesh).or_insert_with(|| {
            (
                Arc::new(GLMesh::new(mesh, gl_texture_container)),
                HashMap::new(),
            )
        });

        let gl_mesh = inner_container.0.clone();

        inner_container
            .1
            .entry(&*gl_mesh_shader_program)
            .or_insert_with(|| {
                Arc::new(GLDrawableMesh::new(
                    gl_mesh.clone(),
                    gl_mesh_shader_program.clone(),
                ))
            })
            .clone()
    }

    pub fn release_mesh(&mut self, mesh: Arc<Mesh>) {
        let key: *const Mesh = &*mesh;
        self.drawable_meshes.remove(&key);
    }
}
