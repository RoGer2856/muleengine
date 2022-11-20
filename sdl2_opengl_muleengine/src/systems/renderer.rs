use std::{collections::BTreeMap, sync::Arc};

use muleengine::{
    asset_container::AssetContainer,
    camera::Camera,
    mesh::{Material, Mesh},
    renderer::{
        renderer_impl::RendererImpl, RendererMaterial, RendererMesh, RendererObject, RendererShader,
    },
    window_context::WindowContext,
};
use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2};

use crate::{
    gl_material::{GLMaterial, RendererMaterialImpl},
    gl_mesh::RendererMeshImpl,
    gl_mesh_container::GLMeshContainer,
    gl_mesh_drawable_object::GLMeshDrawableObject,
    gl_mesh_shader_program::RendererShaderImpl,
    gl_shader_program_container::GLShaderProgramContainer,
    gl_texture_container::GLTextureContainer,
};

pub struct Renderer {
    materials: BTreeMap<*const dyn RendererMaterial, Arc<RwLock<RendererMaterialImpl>>>,
    shaders: BTreeMap<*const dyn RendererShader, Arc<RwLock<RendererShaderImpl>>>,
    meshes: BTreeMap<*const dyn RendererMesh, Arc<RwLock<RendererMeshImpl>>>,

    drawable_meshes: BTreeMap<*const dyn RendererObject, Arc<RwLock<GLMeshDrawableObject>>>,
    #[allow(clippy::type_complexity)]
    drawable_meshes_to_draw:
        BTreeMap<*const dyn RendererObject, (Mat4<f32>, Arc<RwLock<GLMeshDrawableObject>>)>,

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
            materials: BTreeMap::new(),
            shaders: BTreeMap::new(),
            meshes: BTreeMap::new(),

            drawable_meshes: BTreeMap::new(),
            drawable_meshes_to_draw: BTreeMap::new(),

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
        for drawable_object in self.drawable_meshes_to_draw.values() {
            drawable_object.1.read().render(
                &self.camera.transform.position,
                &self.projection_matrix,
                &view_matrix,
                &drawable_object.0,
            );
        }

        self.window_context.read().swap_buffers();
    }

    fn create_material(
        &mut self,
        material: Material,
    ) -> Result<Arc<RwLock<dyn RendererMaterial>>, String> {
        let gl_material = Arc::new(GLMaterial::new(&material, &mut self.gl_texture_container));

        let material = Arc::new(RwLock::new(RendererMaterialImpl::new(gl_material)));

        self.materials.insert(material.data_ptr(), material.clone());

        Ok(material)
    }

    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<Arc<RwLock<dyn RendererShader>>, String> {
        let gl_mesh_shader_program = match self
            .gl_shader_program_container
            .get_mesh_shader_program(&shader_name, &self.asset_container.asset_reader().read())
        {
            Ok(shader_program) => Ok(shader_program),
            Err(e) => Err(format!("Loading shader program, error = {e:?}")),
        }?;

        let shader = Arc::new(RwLock::new(RendererShaderImpl::new(gl_mesh_shader_program)));

        self.shaders.insert(shader.data_ptr(), shader.clone());

        Ok(shader)
    }

    fn create_drawable_mesh(
        &mut self,
        mesh: Arc<Mesh>,
    ) -> Result<Arc<RwLock<dyn RendererMesh>>, String> {
        let gl_mesh = self.gl_mesh_container.get_drawable_mesh(mesh);

        let drawable_mesh = Arc::new(RwLock::new(RendererMeshImpl::new(gl_mesh)));

        self.meshes
            .insert(drawable_mesh.data_ptr(), drawable_mesh.clone());

        Ok(drawable_mesh)
    }

    fn create_drawable_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn RendererMesh>>,
        shader: &Arc<RwLock<dyn RendererShader>>,
        material: &Arc<RwLock<dyn RendererMaterial>>,
    ) -> Result<Arc<RwLock<dyn RendererObject>>, String> {
        let ptr: *const dyn RendererShader = shader.data_ptr();
        let shader = self.shaders.get(&ptr).ok_or_else(|| {
            "Creating drawable object from mesh, error = could not found shader".to_string()
        })?;

        let ptr: *const dyn RendererMaterial = material.data_ptr();
        let material = self.materials.get(&ptr).ok_or_else(|| {
            "Creating drawable object from mesh, error = could not found material".to_string()
        })?;

        let ptr: *const dyn RendererMesh = mesh.data_ptr();
        let gl_mesh = self.meshes.get(&ptr).ok_or_else(|| {
            "Creating drawable object from mesh, error = could not found mesh".to_string()
        })?;

        let drawable_mesh = Arc::new(RwLock::new(GLMeshDrawableObject::new(
            gl_mesh.read().gl_mesh().clone(),
            material.read().gl_material().clone(),
            shader.read().gl_mesh_shader_program().clone(),
        )));

        self.drawable_meshes
            .insert(&*drawable_mesh.read(), drawable_mesh.clone());

        Ok(drawable_mesh)
    }

    fn add_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn RendererObject>>,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererObject = drawable_object.data_ptr();
        let mesh = self.drawable_meshes.get(&ptr).ok_or_else(|| {
            "Adding drawable object, error = could not found drawable object".to_string()
        })?;

        self.drawable_meshes_to_draw
            .insert(ptr, (Mat4::from(transform), mesh.clone()));

        Ok(())
    }

    fn remove_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererObject = drawable_object.data_ptr();
        if self.drawable_meshes_to_draw.remove(&ptr).is_some() {
            Ok(())
        } else {
            Err(
                "Removing drawable object from renderer, error = could not found drawable object"
                    .to_string(),
            )
        }
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
