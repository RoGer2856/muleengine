#![allow(clippy::type_complexity)]

use std::{collections::BTreeMap, sync::Arc};

use muleengine::{
    asset_container::AssetContainer,
    camera::Camera,
    containers::object_pool::{ObjectPool, ObjectPoolIndex},
    mesh::{Material, Mesh},
    prelude::{ArcRwLock, OptionInspector},
    renderer::{
        renderer_impl::RendererImpl, RendererMaterial, RendererMesh, RendererObject,
        RendererShader, RendererTransform,
    },
    window_context::WindowContext,
};
use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2};

use crate::{
    gl_drawable_mesh::GLDrawableMesh,
    gl_material::{GLMaterial, GLRendererMaterialObject},
    gl_mesh::GLRendererMesh,
    gl_mesh_container::GLMeshContainer,
    gl_mesh_renderer_object::GLMeshRendererObject,
    gl_mesh_shader_program::GLMeshRendererShaderObject,
    gl_shader_program_container::GLShaderProgramContainer,
    gl_texture_container::GLTextureContainer,
    me_renderer_objects::{RendererMaterialImpl, RendererMeshImpl, RendererShaderImpl},
};

use super::RendererTransformImpl;

pub struct Renderer {
    renderer_transforms: BTreeMap<*const dyn RendererTransform, ArcRwLock<RendererTransformImpl>>,
    transform_update_observers: BTreeMap<
        *const dyn RendererTransform,
        BTreeMap<*const dyn RendererObject, Box<dyn Fn(&Transform<f32, f32, f32>)>>,
    >,

    renderer_materials: ObjectPool<ArcRwLock<GLRendererMaterialObject>>,
    renderer_shaders: ObjectPool<ArcRwLock<GLMeshRendererShaderObject>>,
    renderer_meshes: ObjectPool<ArcRwLock<GLRendererMesh>>,

    mesh_renderer_objects: BTreeMap<*const dyn RendererObject, ArcRwLock<GLMeshRendererObject>>,
    mesh_renderer_objects_to_draw:
        BTreeMap<*const dyn RendererObject, ArcRwLock<GLMeshRendererObject>>,

    camera: Camera,
    projection_matrix: Mat4<f32>,
    window_dimensions: Vec2<usize>,
    window_context: ArcRwLock<dyn WindowContext>,

    asset_container: AssetContainer,

    gl_mesh_container: GLMeshContainer,
    gl_shader_program_container: GLShaderProgramContainer,
    gl_texture_container: GLTextureContainer,
}

impl Renderer {
    pub fn new(
        initial_window_dimensions: Vec2<usize>,
        window_context: ArcRwLock<dyn WindowContext>,
        asset_container: AssetContainer,
    ) -> Self {
        let mut ret = Self {
            renderer_transforms: BTreeMap::new(),
            transform_update_observers: BTreeMap::new(),

            renderer_materials: ObjectPool::new(),
            renderer_shaders: ObjectPool::new(),
            renderer_meshes: ObjectPool::new(),

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

    fn get_material_index(
        &self,
        mesh: &ArcRwLock<dyn RendererMaterial>,
    ) -> Option<ObjectPoolIndex> {
        let material = mesh.read();
        material
            .as_any()
            .downcast_ref::<RendererMaterialImpl>()
            .map(|val| val.0)
    }

    fn get_shader_index(&self, mesh: &ArcRwLock<dyn RendererShader>) -> Option<ObjectPoolIndex> {
        let shader = mesh.read();
        shader
            .as_any()
            .downcast_ref::<RendererShaderImpl>()
            .map(|val| val.0)
    }

    fn get_mesh_index(&self, mesh: &ArcRwLock<dyn RendererMesh>) -> Option<ObjectPoolIndex> {
        let mesh = mesh.read();
        mesh.as_any()
            .downcast_ref::<RendererMeshImpl>()
            .map(|val| val.0)
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
    ) -> Result<ArcRwLock<dyn RendererTransform>, String> {
        let transform = Arc::new(RwLock::new(RendererTransformImpl { transform }));

        self.renderer_transforms
            .insert(transform.data_ptr(), transform.clone());

        Ok(transform)
    }

    fn update_transform(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
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
        transform: ArcRwLock<dyn RendererTransform>,
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
    ) -> Result<ArcRwLock<dyn RendererMaterial>, String> {
        let gl_material = Arc::new(GLMaterial::new(&material, &mut self.gl_texture_container));

        let renderer_material = Arc::new(RwLock::new(GLRendererMaterialObject::new(gl_material)));
        let index = self.renderer_materials.create_object(renderer_material);

        Ok(Arc::new(RwLock::new(RendererMaterialImpl(index))))
    }

    fn release_material(
        &mut self,
        material: ArcRwLock<dyn RendererMaterial>,
    ) -> Result<(), String> {
        let index = self
            .get_material_index(&material)
            .ok_or_else(|| "Releasing material, error = invalid RendererMaterial".to_string())?;

        self.renderer_materials
            .release_object(index)
            .ok_or_else(|| "Releasing material, error = could not find material".to_string())
            .map(|_| ())
    }

    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<ArcRwLock<dyn RendererShader>, String> {
        let gl_mesh_shader_program = match self
            .gl_shader_program_container
            .get_mesh_shader_program(&shader_name, &self.asset_container.asset_reader().read())
        {
            Ok(shader_program) => Ok(shader_program),
            Err(e) => Err(format!("Loading shader program, error = {e:?}")),
        }?;

        let renderer_shader = Arc::new(RwLock::new(GLMeshRendererShaderObject::new(
            gl_mesh_shader_program,
        )));
        let index = self.renderer_shaders.create_object(renderer_shader);

        Ok(Arc::new(RwLock::new(RendererShaderImpl(index))))
    }

    fn release_shader(&mut self, shader: ArcRwLock<dyn RendererShader>) -> Result<(), String> {
        let index = self
            .get_shader_index(&shader)
            .ok_or_else(|| "Releasing shader, error = invalid RendererShader".to_string())?;

        self.renderer_shaders
            .release_object(index)
            .ok_or_else(|| "Releasing shader, error = could not find shader".to_string())
            .map(|_| ())
    }

    fn create_mesh(&mut self, mesh: Arc<Mesh>) -> Result<ArcRwLock<dyn RendererMesh>, String> {
        let gl_mesh = self.gl_mesh_container.get_gl_mesh(mesh);

        let renderer_mesh = Arc::new(RwLock::new(GLRendererMesh::new(gl_mesh)));
        let index = self.renderer_meshes.create_object(renderer_mesh);

        Ok(Arc::new(RwLock::new(RendererMeshImpl(index))))
    }

    fn release_mesh(&mut self, mesh: ArcRwLock<dyn RendererMesh>) -> Result<(), String> {
        let index = self
            .get_mesh_index(&mesh)
            .ok_or_else(|| "Releasing mesh, error = invalid RendererMesh".to_string())?;

        self.renderer_meshes
            .release_object(index)
            .ok_or_else(|| "Releasing mesh, error = could not find mesh".to_string())
            .map(|_| ())
    }

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: ArcRwLock<dyn RendererMesh>,
        shader: ArcRwLock<dyn RendererShader>,
        material: ArcRwLock<dyn RendererMaterial>,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<ArcRwLock<dyn RendererObject>, String> {
        let ptr: *const dyn RendererTransform = transform.data_ptr();
        let transform = self.renderer_transforms.get(&ptr).ok_or_else(|| {
            "Creating renderer object from mesh, error = could not find transform".to_string()
        })?;

        let material = {
            let index = self.get_material_index(&material).ok_or_else(|| {
                "Creating renderer object from mesh, error = invalid RendererMaterial".to_string()
            })?;

            self.renderer_materials
                .get_ref(index)
                .ok_or_else(|| {
                    "Creating renderer object from mesh, error = could not find materialr"
                        .to_string()
                })
                .cloned()?
        };

        let shader = {
            let index = self.get_shader_index(&shader).ok_or_else(|| {
                "Creating renderer object from mesh, error = invalid RendererShader".to_string()
            })?;

            self.renderer_shaders
                .get_ref(index)
                .ok_or_else(|| {
                    "Creating renderer object from mesh, error = could not find shader".to_string()
                })
                .cloned()?
        };

        let mesh = {
            let index = self.get_mesh_index(&mesh).ok_or_else(|| {
                "Creating renderer object from mesh, error = invalid RendererMesh".to_string()
            })?;

            self.renderer_meshes
                .get_ref(index)
                .ok_or_else(|| {
                    "Creating renderer object from mesh, error = could not find mesh".to_string()
                })
                .cloned()?
        };

        let mesh_renderer_object = Arc::new(RwLock::new(GLMeshRendererObject {
            transform: transform.clone(),
            gl_drawable_mesh: GLDrawableMesh::new(
                mesh.read().gl_mesh().clone(),
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
        renderer_object: ArcRwLock<dyn RendererObject>,
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
        renderer_object: ArcRwLock<dyn RendererObject>,
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
        renderer_object: ArcRwLock<dyn RendererObject>,
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
