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
    gl_mesh_renderer_object::GLMeshRendererObject,
    gl_mesh_shader_program::RendererShaderImpl,
    gl_shader_program_container::GLShaderProgramContainer,
    gl_texture_container::GLTextureContainer,
};

pub struct Renderer {
    renderer_materials: BTreeMap<*const dyn RendererMaterial, Arc<RwLock<RendererMaterialImpl>>>,
    renderer_shaders: BTreeMap<*const dyn RendererShader, Arc<RwLock<RendererShaderImpl>>>,
    renderer_meshes: BTreeMap<*const dyn RendererMesh, Arc<RwLock<RendererMeshImpl>>>,

    mesh_renderer_objects: BTreeMap<*const dyn RendererObject, Arc<RwLock<GLMeshRendererObject>>>,
    #[allow(clippy::type_complexity)]
    mesh_renderer_objects_to_draw:
        BTreeMap<*const dyn RendererObject, (Mat4<f32>, Arc<RwLock<GLMeshRendererObject>>)>,

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
            renderer_materials: BTreeMap::new(),
            renderer_shaders: BTreeMap::new(),
            renderer_meshes: BTreeMap::new(),

            mesh_renderer_objects: BTreeMap::new(),
            mesh_renderer_objects_to_draw: BTreeMap::new(),

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
        for renderer_object in self.mesh_renderer_objects_to_draw.values() {
            renderer_object.1.read().render(
                &self.camera.transform.position,
                &self.projection_matrix,
                &view_matrix,
                &renderer_object.0,
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

        self.renderer_materials
            .insert(material.data_ptr(), material.clone());

        Ok(material)
    }

    fn release_material(
        &mut self,
        material: Arc<RwLock<dyn RendererMaterial>>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererMaterial = material.data_ptr();
        self.renderer_materials
            .remove(&ptr)
            .ok_or_else(|| "Releasing material, error = could not found material".to_string())
            .map(|_| ())
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

        self.renderer_shaders
            .insert(shader.data_ptr(), shader.clone());

        Ok(shader)
    }

    fn release_shader(&mut self, shader: Arc<RwLock<dyn RendererShader>>) -> Result<(), String> {
        let ptr: *const dyn RendererShader = shader.data_ptr();
        self.renderer_shaders
            .remove(&ptr)
            .ok_or_else(|| "Releasing shader, error = could not found shader".to_string())
            .map(|_| ())
    }

    fn create_mesh(&mut self, mesh: Arc<Mesh>) -> Result<Arc<RwLock<dyn RendererMesh>>, String> {
        let gl_mesh = self.gl_mesh_container.get_gl_mesh(mesh);

        let renderer_mesh = Arc::new(RwLock::new(RendererMeshImpl::new(gl_mesh)));

        self.renderer_meshes
            .insert(renderer_mesh.data_ptr(), renderer_mesh.clone());

        Ok(renderer_mesh)
    }

    fn release_mesh(&mut self, mesh: Arc<RwLock<dyn RendererMesh>>) -> Result<(), String> {
        let ptr: *const dyn RendererMesh = mesh.data_ptr();
        self.renderer_meshes
            .remove(&ptr)
            .ok_or_else(|| "Releasing mesh, error = could not found mesh".to_string())
            .map(|_| ())
    }

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn RendererMesh>>,
        shader: &Arc<RwLock<dyn RendererShader>>,
        material: &Arc<RwLock<dyn RendererMaterial>>,
    ) -> Result<Arc<RwLock<dyn RendererObject>>, String> {
        let ptr: *const dyn RendererShader = shader.data_ptr();
        let shader = self.renderer_shaders.get(&ptr).ok_or_else(|| {
            "Creating renderer object from mesh, error = could not found shader".to_string()
        })?;

        let ptr: *const dyn RendererMaterial = material.data_ptr();
        let material = self.renderer_materials.get(&ptr).ok_or_else(|| {
            "Creating renderer object from mesh, error = could not found material".to_string()
        })?;

        let ptr: *const dyn RendererMesh = mesh.data_ptr();
        let gl_mesh = self.renderer_meshes.get(&ptr).ok_or_else(|| {
            "Creating renderer object from mesh, error = could not found mesh".to_string()
        })?;

        let mesh_renderer_object = Arc::new(RwLock::new(GLMeshRendererObject::new(
            gl_mesh.read().gl_mesh().clone(),
            material.read().gl_material().clone(),
            shader.read().gl_mesh_shader_program().clone(),
        )));

        self.mesh_renderer_objects
            .insert(&*mesh_renderer_object.read(), mesh_renderer_object.clone());

        Ok(mesh_renderer_object)
    }

    fn release_renderer_object(
        &mut self,
        renderer_object: Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererObject = renderer_object.data_ptr();
        self.mesh_renderer_objects
            .remove(&ptr)
            .ok_or_else(|| {
                "Releasing renderer object, error = could not found renderer object".to_string()
            })
            .map(|_| ())?;

        self.mesh_renderer_objects_to_draw.remove(&ptr);

        Ok(())
    }

    fn add_renderer_object(
        &mut self,
        renderer_object: &Arc<RwLock<dyn RendererObject>>,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererObject = renderer_object.data_ptr();
        let mesh = self.mesh_renderer_objects.get(&ptr).ok_or_else(|| {
            "Adding renderer object, error = could not found renderer object".to_string()
        })?;

        self.mesh_renderer_objects_to_draw
            .insert(ptr, (Mat4::from(transform), mesh.clone()));

        Ok(())
    }

    fn remove_renderer_object(
        &mut self,
        renderer_object: &Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererObject = renderer_object.data_ptr();
        if self.mesh_renderer_objects_to_draw.remove(&ptr).is_some() {
            Ok(())
        } else {
            Err(
                "Removing renderer object from renderer, error = could not found renderer object"
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
