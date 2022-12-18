use std::{
    collections::BTreeMap,
    sync::Arc,
};

use muleengine::{
    asset_container::AssetContainer,
    camera::Camera,
    containers::object_pool::ObjectPool,
    mesh::{Material, Mesh},
    prelude::{ArcRwLock, ResultInspector},
    renderer::{
        renderer_impl::RendererImpl, RendererGroup, RendererLayer, RendererMaterial, RendererMesh,
        RendererObject, RendererShader, RendererTransform,
    },
    window_context::WindowContext,
};
use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2};

use crate::{
    gl_drawable_mesh::GLDrawableMesh,
    gl_material::{GLMaterial, RendererMaterialObject},
    gl_mesh::RendererMeshObject,
    gl_mesh_container::GLMeshContainer,
    gl_shader_program::RendererShaderObject,
    gl_shader_program_container::GLShaderProgramContainer,
    gl_texture_container::GLTextureContainer,
    me_renderer_indices::{
        RendererGroupIndex, RendererMaterialIndex, RendererMeshIndex, RendererObjectIndex,
        RendererShaderIndex, RendererTransformIndex,
    },
    mesh_renderer_object::MeshRendererObject,
};

use super::{renderer_group_object::RendererGroupObject, RendererTransformObject};

type TransformObservers =
    BTreeMap<*const dyn RendererObject, Box<dyn Fn(&Transform<f32, f32, f32>)>>;

pub struct Renderer {
    renderer_groups: ObjectPool<ArcRwLock<RendererGroupObject>>,
    renderer_transforms: ObjectPool<(ArcRwLock<RendererTransformObject>, TransformObservers)>,
    renderer_materials: ObjectPool<ArcRwLock<RendererMaterialObject>>,
    renderer_shaders: ObjectPool<ArcRwLock<RendererShaderObject>>,
    renderer_meshes: ObjectPool<ArcRwLock<RendererMeshObject>>,

    mesh_renderer_objects: ObjectPool<ArcRwLock<MeshRendererObject>>,

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
            renderer_groups: ObjectPool::new(),
            renderer_transforms: ObjectPool::new(),
            renderer_materials: ObjectPool::new(),
            renderer_shaders: ObjectPool::new(),
            renderer_meshes: ObjectPool::new(),

            mesh_renderer_objects: ObjectPool::new(),

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

    fn add_transform_observer(
        &mut self,
        renderer_transform: &ArcRwLock<dyn RendererTransform>,
        renderer_object: *const dyn RendererObject,
        observer_fn: impl Fn(&Transform<f32, f32, f32>) + 'static,
    ) -> Result<(), String> {
        let index = self
            .get_transform_index(renderer_transform)
            .map_err(|e| format!("Adding transform observer, msg = {e}"))?;

        let (_transform, observers) =
            self.renderer_transforms.get_mut(index.0).ok_or_else(|| {
                "Adding transform observer, msg = could not find RendererTransform".to_string()
            })?;

        observers.insert(renderer_object, Box::new(observer_fn));

        Ok(())
    }

    fn remove_transform_observer_of_renderer_object(
        &mut self,
        renderer_transform: &ArcRwLock<dyn RendererTransform>,
        renderer_object: *const dyn RendererObject,
    ) -> Result<(), String> {
        let index = self
            .get_transform_index(renderer_transform)
            .map_err(|e| format!("Removing transform observer of renderer object, msg = {e}"))?;

        let (_transform, observers) =
            self.renderer_transforms.get_mut(index.0).ok_or_else(|| {
                "Removing transform observer of renderer object, msg = could not find RendererTransform"
                    .to_string()
            })?;

        observers
            .remove(&renderer_object)
            .ok_or_else(|| {
                "Removing transform observer of renderer object, msg = could not find observer for RendererObject".to_string()
            })
            .map(|_| ())
    }

    fn get_renderer_group_index(
        &self,
        renderer_group: &ArcRwLock<dyn RendererGroup>,
    ) -> Result<RendererGroupIndex, String> {
        let renderer_group = renderer_group.read();
        renderer_group
            .as_any()
            .downcast_ref::<RendererGroupIndex>()
            .ok_or_else(|| "invalid RendererGroup provided".to_string())
            .cloned()
    }

    fn get_transform_index(
        &self,
        renderer_transform: &ArcRwLock<dyn RendererTransform>,
    ) -> Result<RendererTransformIndex, String> {
        let renderer_transform = renderer_transform.read();
        renderer_transform
            .as_any()
            .downcast_ref::<RendererTransformIndex>()
            .ok_or_else(|| "invalid RendererTransform provided".to_string())
            .cloned()
    }

    fn get_material_index(
        &self,
        renderer_material: &ArcRwLock<dyn RendererMaterial>,
    ) -> Result<RendererMaterialIndex, String> {
        let renderer_material = renderer_material.read();
        renderer_material
            .as_any()
            .downcast_ref::<RendererMaterialIndex>()
            .ok_or_else(|| "invalid RendererMaterial provided".to_string())
            .cloned()
    }

    fn get_shader_index(
        &self,
        renderer_shader: &ArcRwLock<dyn RendererShader>,
    ) -> Result<RendererShaderIndex, String> {
        let renderer_shader = renderer_shader.read();
        renderer_shader
            .as_any()
            .downcast_ref::<RendererShaderIndex>()
            .ok_or_else(|| "invalid RendererShader provided".to_string())
            .cloned()
    }

    fn get_mesh_index(
        &self,
        renderer_mesh: &ArcRwLock<dyn RendererMesh>,
    ) -> Result<RendererMeshIndex, String> {
        let renderer_mesh = renderer_mesh.read();
        renderer_mesh
            .as_any()
            .downcast_ref::<RendererMeshIndex>()
            .ok_or_else(|| "invalid RendererMesh provided".to_string())
            .cloned()
    }

    fn get_renderer_object_index(
        &self,
        renderer_object: &ArcRwLock<dyn RendererObject>,
    ) -> Result<RendererObjectIndex, String> {
        let renderer_object = renderer_object.read();
        renderer_object
            .as_any()
            .downcast_ref::<RendererObjectIndex>()
            .ok_or_else(|| "invalid RendererObject provided".to_string())
            .cloned()
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
        for renderer_group in self.renderer_groups.iter() {
            renderer_group.read().draw(
                &self.camera.transform.position,
                &self.projection_matrix,
                &view_matrix,
            );
        }

        self.window_context.read().swap_buffers();
    }

    fn create_renderer_layer(&mut self) -> Result<ArcRwLock<dyn RendererLayer>, String> {
        todo!();
        // let renderer_layer = Arc::new(RwLock::new(RendererLayerObject::new()));
        // let index = self.renderer_layers.create_object(renderer_layer);

        // Ok(Arc::new(RwLock::new(RendererLayerIndex(index))))
    }

    fn release_renderer_layer(
        &mut self,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        todo!();
    }

    fn add_renderer_group_to_layer(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        todo!();
    }

    fn remove_renderer_group_from_layer(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        todo!();
    }

    fn create_renderer_group(&mut self) -> Result<ArcRwLock<dyn RendererGroup>, String> {
        let renderer_group = Arc::new(RwLock::new(RendererGroupObject::new()));
        let index = self.renderer_groups.create_object(renderer_group);

        Ok(Arc::new(RwLock::new(RendererGroupIndex(index))))
    }

    fn release_renderer_group(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String> {
        let index = self
            .get_renderer_group_index(&renderer_group)
            .map_err(|e| format!("Releasing renderer group, msg = {e}"))?;

        self.renderer_groups
            .release_object(index.0)
            .ok_or_else(|| {
                "Releasing renderer group, msg = could not find RendererGroup".to_string()
            })
            .map(|_| ())
    }

    fn create_transform(
        &mut self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<ArcRwLock<dyn RendererTransform>, String> {
        let renderer_transform = Arc::new(RwLock::new(RendererTransformObject { transform }));
        let index = self
            .renderer_transforms
            .create_object((renderer_transform, BTreeMap::new()));

        Ok(Arc::new(RwLock::new(RendererTransformIndex(index))))
    }

    fn update_transform(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
        new_transform: Transform<f32, f32, f32>,
    ) -> Result<(), String> {
        let index = self
            .get_transform_index(&transform)
            .map_err(|e| format!("Updating transform, msg = {e}"))?;

        let (transform, observers) =
            self.renderer_transforms.get_mut(index.0).ok_or_else(|| {
                "Updating transform, msg = could not find RendererTransform".to_string()
            })?;

        transform.write().transform = new_transform;

        for observer in observers.values() {
            observer(&new_transform);
        }

        Ok(())
    }

    fn release_transform(
        &mut self,
        transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<(), String> {
        let index = self
            .get_transform_index(&transform)
            .map_err(|e| format!("Releasing transform, msg = {e}"))?;

        self.renderer_transforms
            .release_object(index.0)
            .ok_or_else(|| {
                "Releasing transform, msg = could not find RendererTransform".to_string()
            })
            .map(|_| ())
    }

    fn create_material(
        &mut self,
        material: Material,
    ) -> Result<ArcRwLock<dyn RendererMaterial>, String> {
        let gl_material = Arc::new(GLMaterial::new(&material, &mut self.gl_texture_container));

        let renderer_material = Arc::new(RwLock::new(RendererMaterialObject::new(gl_material)));
        let index = self.renderer_materials.create_object(renderer_material);

        Ok(Arc::new(RwLock::new(RendererMaterialIndex(index))))
    }

    fn release_material(
        &mut self,
        material: ArcRwLock<dyn RendererMaterial>,
    ) -> Result<(), String> {
        let index = self
            .get_material_index(&material)
            .map_err(|e| format!("Releasing material, msg = {e}"))?;

        self.renderer_materials
            .release_object(index.0)
            .ok_or_else(|| "Releasing material, msg = could not find RendererMaterial".to_string())
            .map(|_| ())
    }

    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<ArcRwLock<dyn RendererShader>, String> {
        let gl_shader_program = match self
            .gl_shader_program_container
            .get_shader_program(&shader_name, &self.asset_container.asset_reader().read())
        {
            Ok(shader_program) => Ok(shader_program),
            Err(e) => Err(format!("Loading shader program, msg = {e:?}")),
        }?;

        let renderer_shader = Arc::new(RwLock::new(RendererShaderObject::new(gl_shader_program)));
        let index = self.renderer_shaders.create_object(renderer_shader);

        Ok(Arc::new(RwLock::new(RendererShaderIndex(index))))
    }

    fn release_shader(&mut self, shader: ArcRwLock<dyn RendererShader>) -> Result<(), String> {
        let index = self
            .get_shader_index(&shader)
            .map_err(|e| format!("Releasing shader, msg = {e}"))?;

        self.renderer_shaders
            .release_object(index.0)
            .ok_or_else(|| "Releasing shader, msg = could not find RendererShader".to_string())
            .map(|_| ())
    }

    fn create_mesh(&mut self, mesh: Arc<Mesh>) -> Result<ArcRwLock<dyn RendererMesh>, String> {
        let gl_mesh = self.gl_mesh_container.get_gl_mesh(mesh);

        let renderer_mesh = Arc::new(RwLock::new(RendererMeshObject::new(gl_mesh)));
        let index = self.renderer_meshes.create_object(renderer_mesh);

        Ok(Arc::new(RwLock::new(RendererMeshIndex(index))))
    }

    fn release_mesh(&mut self, mesh: ArcRwLock<dyn RendererMesh>) -> Result<(), String> {
        let index = self
            .get_mesh_index(&mesh)
            .map_err(|e| format!("Releasing mesh, msg = {e}"))?;

        self.renderer_meshes
            .release_object(index.0)
            .ok_or_else(|| "Releasing mesh, msg = could not find RendererMesh".to_string())
            .map(|_| ())
    }

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh: ArcRwLock<dyn RendererMesh>,
        shader: ArcRwLock<dyn RendererShader>,
        material: ArcRwLock<dyn RendererMaterial>,
        renderer_transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<ArcRwLock<dyn RendererObject>, String> {
        let transform = {
            let index = self
                .get_transform_index(&renderer_transform)
                .map_err(|e| format!("Creating renderer object from mesh, msg = {e}"))?;

            &self
                .renderer_transforms
                .get_ref(index.0)
                .ok_or_else(|| {
                    "Creating renderer object from mesh, msg = could not find RendererTransform"
                        .to_string()
                })?
                .0
        };

        let material = {
            let index = self
                .get_material_index(&material)
                .map_err(|e| format!("Creating renderer object from mesh, msg = {e}"))?;

            self.renderer_materials.get_ref(index.0).ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find RendererMaterial"
                    .to_string()
            })?
        };

        let shader = {
            let index = self
                .get_shader_index(&shader)
                .map_err(|e| format!("Creating renderer object from mesh, msg = {e}"))?;

            let shader = self.renderer_shaders.get_ref(index.0).ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find RendererShader"
                    .to_string()
            })?;

            self.gl_shader_program_container
                .get_mesh_shader_program(shader.read().gl_shader_program().clone())
                .map_err(|e| {
                    format!(
                        "Creating renderer object from mesh, RendererShader error = {:?}",
                        e
                    )
                })?
        };

        let mesh = {
            let index = self
                .get_mesh_index(&mesh)
                .map_err(|e| format!("Creating renderer object from mesh, msg = {e}"))?;

            self.renderer_meshes.get_ref(index.0).ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find RendererMesh".to_string()
            })?
        };

        let mesh_renderer_object = Arc::new(RwLock::new(MeshRendererObject {
            transform: renderer_transform.clone(),
            gl_drawable_mesh: GLDrawableMesh::new(
                mesh.read().gl_mesh().clone(),
                material.read().gl_material().clone(),
                transform.read().transform,
                shader,
            ),
        }));

        let index = self
            .mesh_renderer_objects
            .create_object(mesh_renderer_object.clone());

        let ret = Arc::new(RwLock::new(RendererObjectIndex::Mesh(index)));

        {
            let mesh_renderer_object = mesh_renderer_object;
            self.add_transform_observer(&renderer_transform, ret.data_ptr(), move |transform| {
                mesh_renderer_object
                    .write()
                    .gl_drawable_mesh
                    .set_transform(transform)
            })
            .map_err(|e| format!("Creating renderer object from mesh, msg = {e}"))
            .inspect_err(|e| {
                self.mesh_renderer_objects.release_object(index);
                log::error!("{e}");
            })?;
        }

        Ok(ret)
    }

    fn release_renderer_object(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
    ) -> Result<(), String> {
        let index = self
            .get_renderer_object_index(&renderer_object)
            .map_err(|e| format!("Releasing renderer object, msg = {e}"))?;

        match index {
            RendererObjectIndex::Mesh(index) => self
                .mesh_renderer_objects
                .release_object(index)
                .ok_or_else(|| {
                    "Releasing renderer object, msg = could not find RendererObject".to_string()
                })
                .map(|object| {
                    let _ = self
                        .remove_transform_observer_of_renderer_object(
                            &object.read().transform,
                            renderer_object.data_ptr(),
                        )
                        .inspect_err(|e| log::error!("Releasing renderer object, msg = {e}"));
                }),
        }
    }

    fn add_renderer_object_to_group(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String> {
        let renderer_group = {
            let index = self
                .get_renderer_group_index(&renderer_group)
                .map_err(|e| format!("Adding renderer object to group, msg = {e}"))?;

            self.renderer_groups.get_ref(index.0).ok_or_else(|| {
                "Adding renderer object to group, msg = could not find RendererGroup".to_string()
            })?
        };

        let index = self
            .get_renderer_object_index(&renderer_object)
            .map_err(|e| format!("Adding renderer object to group, msg = {e}"))?;

        let missing_renderer_object_error_msg =
            "Adding renderer object to group, msg = could not find renderer object".to_string();
        let adding_twice_error_msg =
            "Adding renderer object to group, msg = cannot add renderer object twice to the same group".to_string();
        match index {
            RendererObjectIndex::Mesh(index) => {
                let renderer_object = self
                    .mesh_renderer_objects
                    .get_mut(index)
                    .ok_or(missing_renderer_object_error_msg)?;
                let old_value = renderer_group
                    .write()
                    .add_mesh_renderer_object(renderer_object.clone());

                match old_value {
                    Some(_) => Err(adding_twice_error_msg),
                    None => Ok(()),
                }?;

                Ok(())
            }
        }
    }

    fn remove_renderer_object_from_group(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
        renderer_group: ArcRwLock<dyn RendererGroup>,
    ) -> Result<(), String> {
        let renderer_group = {
            let index = self
                .get_renderer_group_index(&renderer_group)
                .map_err(|e| format!("Removing renderer object from group, msg = {e}"))?;

            self.renderer_groups.get_ref(index.0).ok_or_else(|| {
                "Removing renderer object from group, msg = could not find RendererGroup"
                    .to_string()
            })?
        };

        let index = self
            .get_renderer_object_index(&renderer_object)
            .map_err(|e| format!("Removing renderer object from group, msg = {e}"))?;

        let missing_renderer_object_error_msg =
            "Removing renderer object from group, msg = could not find renderer object".to_string();
        let missing_renderer_object_in_group_error_msg =
            "Removing renderer object from group, msg = could not find renderer object in group"
                .to_string();
        match index {
            RendererObjectIndex::Mesh(index) => {
                let renderer_object = self
                    .mesh_renderer_objects
                    .get_mut(index)
                    .ok_or(missing_renderer_object_error_msg)?;

                renderer_group
                    .write()
                    .remove_mesh_renderer_object(renderer_object)
                    .ok_or(missing_renderer_object_in_group_error_msg)
                    .map(|_| ())
            }
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
