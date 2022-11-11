#![allow(clippy::type_complexity)]

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::muleengine::mesh::{Mesh, MeshConvertError, Scene};

use super::gl_mesh::{GLDrawableMesh, GLMesh};
use super::gl_mesh_shader_program::GLMeshShaderProgram;
use super::gl_texture_container::GLTextureContainer;

pub struct GLMeshContainer {
    drawable_meshes: HashMap<
        *const Mesh,
        (
            Arc<GLMesh>,
            HashMap<*const GLMeshShaderProgram, Arc<RwLock<GLDrawableMesh>>>,
        ),
    >,
}

impl Default for GLMeshContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl GLMeshContainer {
    pub fn new() -> Self {
        Self {
            drawable_meshes: HashMap::new(),
        }
    }

    pub fn get_drawable_meshes_from_scene(
        &mut self,
        gl_mesh_shader_program: Arc<GLMeshShaderProgram>,
        scene: Arc<Scene>,
        gl_texture_container: &mut GLTextureContainer,
    ) -> Vec<Result<Arc<RwLock<GLDrawableMesh>>, MeshConvertError>> {
        let mut ret = Vec::new();

        for mesh in scene.meshes_ref().iter() {
            match mesh {
                Ok(mesh) => {
                    let drawable_object = self.get_drawable_mesh(
                        gl_mesh_shader_program.clone(),
                        mesh.clone(),
                        gl_texture_container,
                    );
                    ret.push(Ok(drawable_object));
                }
                Err(e) => {
                    ret.push(Err(e.clone()));
                }
            }
        }

        ret
    }

    pub fn get_drawable_mesh(
        &mut self,
        gl_mesh_shader_program: Arc<GLMeshShaderProgram>,
        mesh: Arc<Mesh>,
        gl_texture_container: &mut GLTextureContainer,
    ) -> Arc<RwLock<GLDrawableMesh>> {
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
                Arc::new(RwLock::new(GLDrawableMesh::new(
                    gl_mesh.clone(),
                    gl_mesh_shader_program.clone(),
                )))
            })
            .clone()
    }

    pub fn release_mesh(&mut self, mesh: Arc<Mesh>) {
        let key: *const Mesh = &*mesh;
        self.drawable_meshes.remove(&key);
    }
}
