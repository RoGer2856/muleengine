use std::sync::Arc;

use game_2::{
    muleengine::{
        assets_reader::AssetsReader,
        camera::Camera,
        drawable_object_storage::DrawableObjectStorage,
        image_container::{ImageContainer, ImageContainerError},
        mesh::{Mesh, SceneLoadError},
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
use vek::{Mat4, Transform};

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

    drawable_object_storage: DrawableObjectStorage<GLDrawableMesh>,

    // projection
    window_dimensions: (usize, usize),
    projection_matrix: Mat4<f32>,
    fov_y_degrees: f32,
    near_plane: f32,
    far_plane: f32,
}

impl ApplicationContext {
    pub fn new(initial_window_dimensions: (usize, usize)) -> Self {
        // projection
        let fov_y_degrees = 45.0f32;
        let near_plane = 0.01;
        let far_plane = 1000.0;
        let projection_matrix = Mat4::perspective_fov_rh_zo(
            fov_y_degrees.to_radians(),
            initial_window_dimensions.0 as f32,
            initial_window_dimensions.1 as f32,
            near_plane,
            far_plane,
        );

        Self {
            assets_reader: AssetsReader::new(),
            scene_container: SceneContainer::new(),
            image_container: ImageContainer::new(),

            gl_mesh_container: GLMeshContainer::new(),
            gl_shader_program_container: GLShaderProgramContainer::new(),
            gl_texture_container: GLTextureContainer::new(),

            drawable_object_storage: DrawableObjectStorage::new(),

            // projection
            window_dimensions: initial_window_dimensions,
            projection_matrix,
            fov_y_degrees,
            near_plane,
            far_plane,
        }
    }

    pub fn get_texture(&mut self, image_path: &str) -> Result<Arc<Texture2D>, ImageContainerError> {
        let image = self
            .image_container
            .get_image(image_path, &self.assets_reader)?;
        Ok(self.gl_texture_container.get_texture(image))
    }

    pub fn add_mesh(
        &mut self,
        shader_basepath: &str,
        mesh: Arc<Mesh>,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), GLMeshShaderProgramError> {
        let gl_mesh_shader_program = self
            .gl_shader_program_container
            .get_mesh_shader_program(shader_basepath, &mut self.assets_reader)?;

        let gl_drawable_mesh = self.gl_mesh_container.get_drawable_mesh(
            gl_mesh_shader_program.clone(),
            mesh.clone(),
            &mut self.gl_texture_container,
        );
        self.drawable_object_storage
            .add_drawable_object(gl_drawable_mesh, transform);

        Ok(())
    }

    pub fn add_drawable_object(
        &mut self,
        drawable_object: Arc<RwLock<GLDrawableMesh>>,
        transform: Transform<f32, f32, f32>,
    ) {
        self.drawable_object_storage
            .add_drawable_object(drawable_object, transform);
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
            .scene_container
            .get_scene(
                scene_path,
                &mut self.assets_reader,
                &mut self.image_container,
            )
            .map_err(|e| ApplicationMeshLoadError::SceneLoadError(e))?;

        for mesh in scene.meshes_ref().iter() {
            match mesh {
                Ok(mesh) => {
                    let gl_drawable_mesh = self.gl_mesh_container.get_drawable_mesh(
                        gl_mesh_shader_program.clone(),
                        mesh.clone(),
                        &mut self.gl_texture_container,
                    );
                    ret.push(gl_drawable_mesh.clone());
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

    pub fn window_resized(&mut self, width: usize, height: usize) {
        self.window_dimensions = (width, height);

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }

        self.projection_matrix = Mat4::perspective_fov_rh_zo(
            self.fov_y_degrees.to_radians(),
            width as f32,
            height as f32,
            self.near_plane,
            self.far_plane,
        );
    }

    pub fn render(&mut self, camera: &Camera) {
        let view_matrix = camera.compute_view_matrix();
        self.drawable_object_storage.render_all(
            &camera.transform.position,
            &self.projection_matrix,
            &view_matrix,
        );
    }
}
