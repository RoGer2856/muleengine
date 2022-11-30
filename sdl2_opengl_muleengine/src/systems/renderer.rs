#![allow(clippy::type_complexity)]

use std::{collections::BTreeMap, sync::Arc};

use muleengine::{
    asset_container::AssetContainer,
    camera::Camera,
    mesh::{Material, Mesh},
    renderer::{
        renderer_impl::RendererImpl, RendererMaterial, RendererMesh, RendererObject,
        RendererShader, RendererTransform,
    },
    result_option_inspect::OptionInspector,
    window_context::WindowContext,
};
use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2};

use crate::{
    gl_drawable_mesh::GLDrawableMesh,
    gl_material::{GLMaterial, RendererMaterialImpl},
    gl_mesh::RendererMeshImpl,
    gl_mesh_container::GLMeshContainer,
    gl_mesh_renderer_object::GLMeshRendererObject,
    gl_mesh_shader_program::RendererShaderImpl,
    gl_shader_program_container::GLShaderProgramContainer,
    gl_texture_container::GLTextureContainer,
};

use super::RendererTransformImpl;

pub struct Renderer {
    renderer_transforms: BTreeMap<*const dyn RendererTransform, Arc<RwLock<RendererTransformImpl>>>,
    transform_update_observers: BTreeMap<
        *const dyn RendererTransform,
        BTreeMap<*const dyn RendererObject, Box<dyn Fn(&Transform<f32, f32, f32>)>>,
    >,

    renderer_materials: BTreeMap<*const dyn RendererMaterial, Arc<RwLock<RendererMaterialImpl>>>,
    renderer_shaders: BTreeMap<*const dyn RendererShader, Arc<RwLock<RendererShaderImpl>>>,
    renderer_meshes: BTreeMap<*const dyn RendererMesh, Arc<RwLock<RendererMeshImpl>>>,

    mesh_renderer_objects: BTreeMap<*const dyn RendererObject, Arc<RwLock<GLMeshRendererObject>>>,
    mesh_renderer_objects_to_draw:
        BTreeMap<*const dyn RendererObject, Arc<RwLock<GLMeshRendererObject>>>,

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
            renderer_transforms: BTreeMap::new(),
            transform_update_observers: BTreeMap::new(),

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

    fn add_transform_update_observer(
        &mut self,
        renderer_transform: *const dyn RendererTransform,
        renderer_object: *const dyn RendererObject,
        observer_fn: impl Fn(&Transform<f32, f32, f32>) + 'static,
    ) {
        self.transform_update_observers
            .entry(renderer_transform)
            .or_insert_with(|| {
                let mut map: BTreeMap<
                    *const dyn RendererObject,
                    Box<dyn Fn(&Transform<f32, f32, f32>)>,
                > = BTreeMap::new();
                map.insert(renderer_object, Box::new(observer_fn));
                map
            });
    }

    fn remove_transform_update_observers(
        &mut self,
        renderer_transform: *const dyn RendererTransform,
    ) {
        self.transform_update_observers.remove(&renderer_transform);
    }

    fn remove_transform_update_observer_of_renderer_object(
        &mut self,
        renderer_transform: *const dyn RendererTransform,
        renderer_object: *const dyn RendererObject,
    ) {
        self.transform_update_observers
            .get_mut(&renderer_transform)
            .map(|observers| {
                observers.remove(&renderer_object);
                Some(())
            });
    }

    fn trigger_transform_update_observer(
        &self,
        renderer_transform: *const dyn RendererTransform,
        transform: &Transform<f32, f32, f32>,
    ) {
        self.transform_update_observers
            .get(&renderer_transform)
            .inspect(|observers| {
                for observer in observers.values() {
                    observer(transform);
                }
            });
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
            renderer_object.read().gl_drawable_mesh.render(
                &self.camera.transform.position,
                &self.projection_matrix,
                &view_matrix,
            );
        }

        self.window_context.read().swap_buffers();
    }

    fn create_transform(
        &mut self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<Arc<RwLock<dyn RendererTransform>>, String> {
        let transform = Arc::new(RwLock::new(RendererTransformImpl { transform }));

        self.renderer_transforms
            .insert(transform.data_ptr(), transform.clone());

        Ok(transform)
    }

    fn update_transform(
        &mut self,
        transform: &Arc<RwLock<dyn RendererTransform>>,
        new_transform: Transform<f32, f32, f32>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererTransform = transform.data_ptr();
        let transform = self
            .renderer_transforms
            .get(&ptr)
            .ok_or_else(|| "Updating transform, error = could not find transform".to_string())?;

        transform.write().transform = new_transform;

        self.trigger_transform_update_observer(transform.data_ptr(), &new_transform);

        Ok(())
    }

    fn release_transform(
        &mut self,
        transform: Arc<RwLock<dyn RendererTransform>>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererTransform = transform.data_ptr();

        self.remove_transform_update_observers(ptr);

        self.renderer_transforms
            .remove(&ptr)
            .ok_or_else(|| "Releasing transform, error = could not find transform".to_string())
            .map(|_| ())
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
            .ok_or_else(|| "Releasing material, error = could not find material".to_string())
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
            .ok_or_else(|| "Releasing shader, error = could not find shader".to_string())
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
            .ok_or_else(|| "Releasing mesh, error = could not find mesh".to_string())
            .map(|_| ())
    }

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn RendererMesh>>,
        shader: &Arc<RwLock<dyn RendererShader>>,
        material: &Arc<RwLock<dyn RendererMaterial>>,
        transform: &Arc<RwLock<dyn RendererTransform>>,
    ) -> Result<Arc<RwLock<dyn RendererObject>>, String> {
        let ptr: *const dyn RendererShader = shader.data_ptr();
        let shader = self.renderer_shaders.get(&ptr).ok_or_else(|| {
            "Creating renderer object from mesh, error = could not find shader".to_string()
        })?;

        let ptr: *const dyn RendererMaterial = material.data_ptr();
        let material = self.renderer_materials.get(&ptr).ok_or_else(|| {
            "Creating renderer object from mesh, error = could not find material".to_string()
        })?;

        let ptr: *const dyn RendererTransform = transform.data_ptr();
        let transform = self.renderer_transforms.get(&ptr).ok_or_else(|| {
            "Creating renderer object from mesh, error = could not find transform".to_string()
        })?;

        let ptr: *const dyn RendererMesh = mesh.data_ptr();
        let gl_mesh = self.renderer_meshes.get(&ptr).ok_or_else(|| {
            "Creating renderer object from mesh, error = could not find mesh".to_string()
        })?;

        let mesh_renderer_object = Arc::new(RwLock::new(GLMeshRendererObject {
            transform: transform.clone(),
            gl_drawable_mesh: GLDrawableMesh::new(
                gl_mesh.read().gl_mesh().clone(),
                material.read().gl_material().clone(),
                transform.read().transform,
                shader.read().gl_mesh_shader_program().clone(),
            ),
        }));

        {
            let mesh_renderer_object = mesh_renderer_object.clone();
            self.add_transform_update_observer(
                transform.data_ptr(),
                mesh_renderer_object.data_ptr(),
                move |transform| {
                    mesh_renderer_object
                        .write()
                        .gl_drawable_mesh
                        .set_transform(transform)
                },
            );
        }

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
                "Releasing renderer object, error = could not find renderer object".to_string()
            })
            .map(|renderer_object| {
                self.remove_transform_update_observer_of_renderer_object(
                    renderer_object.read().transform.data_ptr(),
                    ptr,
                );
                Some(())
            })?;

        self.mesh_renderer_objects_to_draw.remove(&ptr);

        Ok(())
    }

    fn add_renderer_object(
        &mut self,
        renderer_object: &Arc<RwLock<dyn RendererObject>>,
    ) -> Result<(), String> {
        let ptr: *const dyn RendererObject = renderer_object.data_ptr();
        let mesh = self.mesh_renderer_objects.get(&ptr).ok_or_else(|| {
            "Adding renderer object, error = could not find renderer object".to_string()
        })?;

        self.mesh_renderer_objects_to_draw.insert(ptr, mesh.clone());

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
                "Removing renderer object from renderer, error = could not find renderer object"
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
