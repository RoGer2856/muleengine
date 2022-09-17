use std::sync::Arc;

use crate::{
    muleengine::{
        assets_reader::AssetsReader,
        image_container::{ImageContainer, ImageContainerError},
        mesh::{Mesh, Scene, SceneLoadError},
        scene_container::SceneContainer,
        service_container::ServiceContainer,
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
pub enum DrawableMeshCreationError {
    GLMeshShaderProgramError(GLMeshShaderProgramError),
    SceneLoadError(SceneLoadError),
}

pub struct DrawableObjectCreator {
    assets_reader: Arc<RwLock<AssetsReader>>,
    scene_container: Arc<RwLock<SceneContainer>>,
    image_container: Arc<RwLock<ImageContainer>>,

    gl_mesh_container: GLMeshContainer,
    gl_shader_program_container: GLShaderProgramContainer,
    gl_texture_container: GLTextureContainer,
}

impl DrawableObjectCreator {
    pub fn new(service_container: &ServiceContainer) -> Self {
        Self {
            assets_reader: service_container.get_service::<AssetsReader>().unwrap(),
            scene_container: service_container.get_service::<SceneContainer>().unwrap(),
            image_container: service_container.get_service::<ImageContainer>().unwrap(),

            gl_mesh_container: GLMeshContainer::new(),
            gl_shader_program_container: GLShaderProgramContainer::new(),
            gl_texture_container: GLTextureContainer::new(),
        }
    }

    pub fn get_texture(&mut self, image_path: &str) -> Result<Arc<Texture2D>, ImageContainerError> {
        let image = self
            .image_container
            .write()
            .get_image(image_path, &self.assets_reader.write())?;
        Ok(self.gl_texture_container.get_texture(image))
    }

    pub fn get_drawable_object_from_mesh(
        &mut self,
        shader_basepath: &str,
        mesh: Arc<Mesh>,
    ) -> Result<Arc<RwLock<GLDrawableMesh>>, GLMeshShaderProgramError> {
        let gl_mesh_shader_program = self
            .gl_shader_program_container
            .get_mesh_shader_program(shader_basepath, &mut self.assets_reader.write())?;

        let gl_drawable_mesh = self.gl_mesh_container.get_drawable_mesh(
            gl_mesh_shader_program.clone(),
            mesh.clone(),
            &mut self.gl_texture_container,
        );

        Ok(gl_drawable_mesh)
    }

    pub fn load_scene(&mut self, scene_path: &str) -> Result<Arc<Scene>, SceneLoadError> {
        self.scene_container.write().get_scene(
            scene_path,
            &mut self.assets_reader.write(),
            &mut self.image_container.write(),
        )
    }

    pub fn get_drawable_objects_from_scene(
        &mut self,
        shader_basepath: &str,
        scene_path: &str,
    ) -> Result<Vec<Arc<RwLock<GLDrawableMesh>>>, DrawableMeshCreationError> {
        let mut ret = Vec::new();

        let gl_mesh_shader_program = self
            .gl_shader_program_container
            .get_mesh_shader_program(shader_basepath, &mut self.assets_reader.write())
            .map_err(|e| DrawableMeshCreationError::GLMeshShaderProgramError(e))?;

        let scene = self
            .load_scene(scene_path)
            .map_err(|e| DrawableMeshCreationError::SceneLoadError(e))?;

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
