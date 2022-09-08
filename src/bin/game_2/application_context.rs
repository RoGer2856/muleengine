use std::sync::Arc;

use game_2::{
    muleengine::{
        assets_reader::AssetsReader,
        image_container::{ImageContainer, ImageContainerError},
        mesh::{Mesh, Scene, SceneLoadError},
        scene_container::SceneContainer,
    },
    sdl2_opengl_engine::{
        gl_mesh::GLDrawableMesh, gl_mesh_container::GLMeshContainer,
        gl_mesh_shader_program::GLMeshShaderProgramError,
        gl_shader_program_container::GLShaderProgramContainer,
        gl_texture_container::GLTextureContainer, opengl_utils::texture_2d::Texture2D,
    },
};
use parking_lot::RwLock;

#[derive(Debug)]
pub enum ApplicationMeshLoadError {
    GLMeshShaderProgramError(GLMeshShaderProgramError),
    SceneLoadError(SceneLoadError),
}

pub struct ApplicationContext {
    assets_reader: AssetsReader,
    scene_container: SceneContainer,
    image_container: ImageContainer,

    gl_mesh_container: GLMeshContainer,
    gl_shader_program_container: GLShaderProgramContainer,
    gl_texture_container: GLTextureContainer,
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self {
            assets_reader: AssetsReader::new(),
            scene_container: SceneContainer::new(),
            image_container: ImageContainer::new(),

            gl_mesh_container: GLMeshContainer::new(),
            gl_shader_program_container: GLShaderProgramContainer::new(),
            gl_texture_container: GLTextureContainer::new(),
        }
    }

    pub fn get_texture(&mut self, image_path: &str) -> Result<Arc<Texture2D>, ImageContainerError> {
        let image = self
            .image_container
            .get_image(image_path, &self.assets_reader)?;
        Ok(self.gl_texture_container.get_texture(image))
    }

    pub fn get_drawable_object_from_mesh(
        &mut self,
        shader_basepath: &str,
        mesh: Arc<Mesh>,
    ) -> Result<Arc<RwLock<GLDrawableMesh>>, GLMeshShaderProgramError> {
        let gl_mesh_shader_program = self
            .gl_shader_program_container
            .get_mesh_shader_program(shader_basepath, &mut self.assets_reader)?;

        let gl_drawable_mesh = self.gl_mesh_container.get_drawable_mesh(
            gl_mesh_shader_program.clone(),
            mesh.clone(),
            &mut self.gl_texture_container,
        );

        Ok(gl_drawable_mesh)
    }

    pub fn load_scene(&mut self, scene_path: &str) -> Result<Arc<Scene>, SceneLoadError> {
        self.scene_container.get_scene(
            scene_path,
            &mut self.assets_reader,
            &mut self.image_container,
        )
    }

    pub fn get_drawable_objects_from_scene(
        &mut self,
        shader_basepath: &str,
        scene_path: &str,
    ) -> Result<Vec<Arc<RwLock<GLDrawableMesh>>>, ApplicationMeshLoadError> {
        let mut ret = Vec::new();

        let gl_mesh_shader_program = self
            .gl_shader_program_container
            .get_mesh_shader_program(shader_basepath, &mut self.assets_reader)
            .map_err(|e| ApplicationMeshLoadError::GLMeshShaderProgramError(e))?;

        let scene = self
            .load_scene(scene_path)
            .map_err(|e| ApplicationMeshLoadError::SceneLoadError(e))?;

        let drawable_objects = self.gl_mesh_container.get_drawable_meshes_from_scene(
            gl_mesh_shader_program,
            scene,
            &mut self.gl_texture_container,
        );

        for drawable_object in drawable_objects {
            match drawable_object {
                Ok(drawable_object) => {
                    ret.push(drawable_object);
                }
                Err(e) => {
                    log::warn!(
                        "Could not load mesh, asset = \"{}\", error = \"{:?}\"",
                        scene_path,
                        e
                    );
                }
            }
        }

        Ok(ret)
    }
}
