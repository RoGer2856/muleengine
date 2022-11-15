use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2};

use crate::{
    muleengine::{
        asset_container::AssetContainer,
        camera::Camera,
        drawable_object_storage::{DrawableObjectStorage, DrawableObjectStorageIndex},
        mesh::{Material, Mesh},
        renderer::RendererImpl,
        window_context::WindowContext,
    },
    sdl2_opengl_engine::{
        gl_material::GLMaterial, gl_mesh::GLDrawableMesh, gl_mesh_container::GLMeshContainer,
        gl_shader_program_container::GLShaderProgramContainer,
        gl_texture_container::GLTextureContainer,
    },
};

pub struct Renderer {
    drawable_object_storage: DrawableObjectStorage,

    camera: Camera,
    projection_matrix: Mat4<f32>,
    window_dimensions: Vec2<usize>,
    window_context: Arc<RwLock<dyn WindowContext>>,

    asset_container: AssetContainer,

    gl_mesh_container: GLMeshContainer,
    gl_shader_program_container: GLShaderProgramContainer,
    gl_texture_container: GLTextureContainer,
}

impl Renderer {
    pub fn new(
        initial_window_dimensions: Vec2<usize>,
        window_context: Arc<RwLock<dyn WindowContext>>,
        asset_container: AssetContainer,
    ) -> Self {
        let mut ret = Self {
            drawable_object_storage: DrawableObjectStorage::new(),

            camera: Camera::new(),
            projection_matrix: Mat4::identity(),
            window_dimensions: Vec2::zero(),
            window_context,

            asset_container,

            gl_mesh_container: GLMeshContainer::new(),
            gl_shader_program_container: GLShaderProgramContainer::new(),
            gl_texture_container: GLTextureContainer::new(),
        };

        ret.set_window_dimensions(initial_window_dimensions);

        ret
    }
}

impl RendererImpl for Renderer {
    fn render(&mut self) {
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Enable(gl::DEPTH_TEST);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let view_matrix = self.camera.compute_view_matrix();
        for (transform, drawable_object) in self.drawable_object_storage.iter() {
            let guard = drawable_object.read();
            let any_obj = guard.as_any();
            if let Some(drawable_object) = any_obj.downcast_ref::<GLDrawableMesh>() {
                drawable_object.render(
                    &self.camera.transform.position,
                    &self.projection_matrix,
                    &view_matrix,
                    transform,
                );
            }
        }

        self.window_context.read().swap_buffers();
    }

    fn add_drawable_mesh(
        &mut self,
        mesh: Arc<Mesh>,
        transform: Transform<f32, f32, f32>,
        material: Option<Material>,
        shader_path: String,
    ) -> DrawableObjectStorageIndex {
        let gl_mesh_shader_program = match self
            .gl_shader_program_container
            .get_mesh_shader_program(&shader_path, &self.asset_container.asset_reader().read())
        {
            Ok(shader_program) => shader_program,
            Err(e) => {
                log::error!("Error loading shader program, error = {e:?}");
                todo!();
            }
        };

        let drawable_mesh = self.gl_mesh_container.get_drawable_mesh(
            gl_mesh_shader_program,
            mesh,
            &mut self.gl_texture_container,
        );

        if let Some(material) = material {
            let material = GLMaterial::new(&material, &mut self.gl_texture_container);
            drawable_mesh.write().material = Some(material);
        }

        let index = self
            .drawable_object_storage
            .add_drawable_object(drawable_mesh, transform);

        index
    }

    fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    fn set_window_dimensions(&mut self, window_dimensions: Vec2<usize>) {
        self.window_dimensions = window_dimensions;

        let fov_y_degrees = 45.0f32;
        let near_plane = 0.01;
        let far_plane = 1000.0;
        self.projection_matrix = Mat4::perspective_fov_rh_zo(
            fov_y_degrees.to_radians(),
            window_dimensions.x as f32,
            window_dimensions.y as f32,
            near_plane,
            far_plane,
        );

        unsafe {
            gl::Viewport(0, 0, window_dimensions.x as i32, window_dimensions.y as i32);
        }
    }
}
