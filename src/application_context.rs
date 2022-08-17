use std::sync::Arc;

use game_2::{
    muleengine::{
        assets_reader::AssetsReader, camera::Camera,
        drawable_object_storage::DrawableObjectStorage, image_container::ImageContainer,
        mesh::SceneLoadError, mesh_creator, scene_container::SceneContainer,
    },
    sdl2_opengl_engine::{
        gl_mesh::{GLDrawableMesh, GLMesh},
        gl_mesh_shader_program::GLMeshShaderProgramError,
        gl_scene_container::GLSceneContainer,
        gl_shader_program_container::GLShaderProgramContainer,
    },
};
use vek::{Mat4, Transform, Vec3};

#[derive(Debug)]
pub enum ApplicationMeshLoadError {
    GLMeshShaderProgramError(GLMeshShaderProgramError),
    SceneLoadError(SceneLoadError),
}

pub struct ApplicationContext {
    assets_reader: AssetsReader,
    gl_scene_container: GLSceneContainer,
    image_container: ImageContainer,
    shader_program_container: GLShaderProgramContainer,

    drawable_object_storage: DrawableObjectStorage,

    camera: Camera,
    moving_direction: Vec3<f32>,

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

        let mut ret = Self {
            assets_reader: AssetsReader::new(),
            gl_scene_container: GLSceneContainer::new(SceneContainer::new()),
            image_container: ImageContainer::new(),
            shader_program_container: GLShaderProgramContainer::new(),

            drawable_object_storage: DrawableObjectStorage::new(),

            camera: Camera::new(),
            moving_direction: Vec3::zero(),

            // projection
            window_dimensions: initial_window_dimensions,
            projection_matrix,
            fov_y_degrees,
            near_plane,
            far_plane,
        };

        let gl_mesh_shader_program = ret
            .shader_program_container
            .get_mesh_shader_program("src/shaders/unlit", &ret.assets_reader)
            .unwrap();
        let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));
        let gl_mesh = Arc::new(GLMesh::new(mesh));
        let gl_drawable_mesh = GLDrawableMesh::new(gl_mesh, gl_mesh_shader_program);

        let mut transform = Transform::<f32, f32, f32>::default();
        transform.position.x = -2.0;
        transform.position.z = -5.0;
        ret.drawable_object_storage
            .add_drawable_object(Box::new(gl_drawable_mesh), transform);

        ret
    }

    pub fn add_scene_from_asset(
        &mut self,
        shader_basepath: &str,
        scene_path: &str,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), ApplicationMeshLoadError> {
        let gl_mesh_shader_program = self
            .shader_program_container
            .get_mesh_shader_program(shader_basepath, &mut self.assets_reader)
            .map_err(|e| ApplicationMeshLoadError::GLMeshShaderProgramError(e))?;

        let scene = self
            .gl_scene_container
            .get_scene(scene_path, &mut self.assets_reader)
            .map_err(|e| ApplicationMeshLoadError::SceneLoadError(e))?;

        for gl_mesh in scene.meshes_ref().iter() {
            match gl_mesh {
                Ok(gl_mesh) => {
                    let gl_drawable_mesh =
                        GLDrawableMesh::new(gl_mesh.clone(), gl_mesh_shader_program.clone());

                    self.drawable_object_storage
                        .add_drawable_object(Box::new(gl_drawable_mesh), transform);
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

        Ok(())
    }

    pub fn tick(&mut self, delta_time: f32) {
        let velocity = 0.5;

        self.camera
            .move_by(self.moving_direction * velocity * delta_time);
    }

    pub fn set_moving_direction(&mut self, mut direction: Vec3<f32>) {
        if direction != Vec3::zero() {
            direction.normalize();
        }

        self.moving_direction = direction
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

    pub fn render(&mut self) {
        let view_matrix = self.camera.compute_view_matrix();
        self.drawable_object_storage
            .render_all(&self.projection_matrix, &view_matrix);
    }
}
