use std::sync::Arc;

use muleengine::{
    asset_container::AssetContainer,
    containers::object_pool::ObjectPool,
    mesh::{Material, Mesh},
    messaging::observable_fn::{Observable, Observer},
    prelude::ArcRwLock,
    renderer::{
        renderer_impl::RendererImpl, renderer_pipeline_step_impl::RendererPipelineStepImpl,
        RendererCamera, RendererGroup, RendererLayer, RendererMaterial, RendererMesh,
        RendererObject, RendererShader, RendererTransform,
    },
    window_context::WindowContext,
};
use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2, Vec4};

use crate::{
    gl_drawable_mesh::GLDrawableMesh,
    gl_material::{GLMaterial, RendererMaterialObject},
    gl_mesh::RendererMeshObject,
    gl_mesh_container::GLMeshContainer,
    gl_shader_program::RendererShaderObject,
    gl_shader_program_container::GLShaderProgramContainer,
    gl_texture_container::GLTextureContainer,
    me_renderer_indices::{
        RendererCameraIndex, RendererGroupIndex, RendererLayerIndex, RendererMaterialIndex,
        RendererMeshIndex, RendererObjectIndex, RendererShaderIndex, RendererTransformIndex,
    },
    mesh_renderer_object::MeshRendererObject,
};

use super::{
    renderer_group_object::RendererGroupObject, renderer_layer_object::RendererLayerObject,
    renderer_pipeline_step_object::RendererPipelineStepObject, RendererCameraObject,
    RendererTransformObject,
};

type TransformObserver = Observer<RendererTransformObject>;

pub struct Renderer {
    renderer_pipeline_steps: Vec<RendererPipelineStepObject>,

    renderer_cameras: ObjectPool<(ArcRwLock<RendererCameraObject>, TransformObserver)>,
    renderer_layers: ObjectPool<ArcRwLock<RendererLayerObject>>,
    renderer_groups: ObjectPool<ArcRwLock<RendererGroupObject>>,
    renderer_transforms: ObjectPool<ArcRwLock<Observable<RendererTransformObject>>>,
    renderer_materials: ObjectPool<ArcRwLock<RendererMaterialObject>>,
    renderer_shaders: ObjectPool<ArcRwLock<RendererShaderObject>>,
    renderer_meshes: ObjectPool<ArcRwLock<RendererMeshObject>>,

    mesh_renderer_objects: ObjectPool<(ArcRwLock<MeshRendererObject>, TransformObserver)>,

    screen_clear_color: Vec4<f32>,

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
            renderer_pipeline_steps: Vec::new(),

            renderer_cameras: ObjectPool::new(),
            renderer_layers: ObjectPool::new(),
            renderer_groups: ObjectPool::new(),
            renderer_transforms: ObjectPool::new(),
            renderer_materials: ObjectPool::new(),
            renderer_shaders: ObjectPool::new(),
            renderer_meshes: ObjectPool::new(),

            mesh_renderer_objects: ObjectPool::new(),

            screen_clear_color: Vec4::zero(),

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

    fn get_renderer_layer_index(
        &self,
        renderer_layer: &ArcRwLock<dyn RendererLayer>,
    ) -> Result<RendererLayerIndex, String> {
        let renderer_layer = renderer_layer.read();
        renderer_layer
            .as_any()
            .downcast_ref::<RendererLayerIndex>()
            .ok_or_else(|| "invalid RendererLayer provided".to_string())
            .cloned()
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

    fn get_camera_index(
        &self,
        renderer_camera: &ArcRwLock<dyn RendererCamera>,
    ) -> Result<RendererCameraIndex, String> {
        let renderer_camera = renderer_camera.read();
        renderer_camera
            .as_any()
            .downcast_ref::<RendererCameraIndex>()
            .ok_or_else(|| "invalid RendererCamera provided".to_string())
            .cloned()
    }

    fn ndc_to_ssc(&self, ndc: &Vec2<f32>) -> Vec2<f32> {
        Vec2::new(
            ndc.x * self.window_dimensions.x as f32,
            ndc.y * self.window_dimensions.y as f32,
        )
    }

    fn set_gl_viewport(&self, viewport_start_ndc: &Vec2<f32>, viewport_dimensions_ndc: &Vec2<f32>) {
        let viewport_start_ssc = self.ndc_to_ssc(viewport_start_ndc);
        let viewport_dimensions_ssc = self.ndc_to_ssc(viewport_dimensions_ndc);
        unsafe {
            gl::Viewport(
                viewport_start_ssc.x as i32,
                viewport_start_ssc.y as i32,
                viewport_dimensions_ssc.x as i32,
                viewport_dimensions_ssc.y as i32,
            );
        }
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

impl RendererImpl for Renderer {
    fn render(&mut self) {
        let window_dimensions = self.window_context.read().window_dimensions();
        self.set_window_dimensions(window_dimensions);

        unsafe {
            gl::ClearColor(
                self.screen_clear_color.x,
                self.screen_clear_color.y,
                self.screen_clear_color.z,
                self.screen_clear_color.w,
            );
            gl::Enable(gl::DEPTH_TEST);
        }

        for step in self.renderer_pipeline_steps.iter() {
            match step {
                RendererPipelineStepObject::Clear {
                    depth,
                    color,
                    viewport_start_ndc,
                    viewport_end_ndc: viewport_dimensions_ndc,
                } => {
                    self.set_gl_viewport(viewport_start_ndc, viewport_dimensions_ndc);

                    if *depth && *color {
                        unsafe {
                            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                        }
                    } else if *depth {
                        unsafe {
                            gl::Clear(gl::DEPTH_BUFFER_BIT);
                        }
                    } else if *color {
                        unsafe {
                            gl::Clear(gl::COLOR_BUFFER_BIT);
                        }
                    }
                }
                RendererPipelineStepObject::Draw {
                    renderer_layer: renderer_layer_object,
                    viewport_start_ndc,
                    viewport_end_ndc: viewport_dimensions_ndc,
                } => {
                    self.set_gl_viewport(viewport_start_ndc, viewport_dimensions_ndc);

                    renderer_layer_object.read().draw(&self.projection_matrix);
                }
            }
        }

        self.window_context.read().swap_buffers();
    }

    fn set_renderer_pipeline(
        &mut self,
        steps: Vec<RendererPipelineStepImpl>,
    ) -> Result<(), String> {
        self.renderer_pipeline_steps = Vec::with_capacity(steps.capacity());
        for step in steps {
            let step_object = match step {
                RendererPipelineStepImpl::Clear {
                    depth,
                    color,
                    viewport_start_ndc,
                    viewport_end_ndc,
                } => RendererPipelineStepObject::Clear {
                    depth,
                    color,
                    viewport_start_ndc,
                    viewport_end_ndc,
                },
                RendererPipelineStepImpl::Draw {
                    renderer_layer,
                    viewport_start_ndc,
                    viewport_end_ndc,
                } => {
                    let renderer_layer = {
                        let index = self
                            .get_renderer_layer_index(&renderer_layer)
                            .map_err(|e| format!("Setting renderer pipeline, msg = {e}"))?;

                        self.renderer_layers
                            .get_ref(index.0)
                            .ok_or_else(|| {
                                "Setting renderer pipeline, msg = could not find RendererLayer"
                                    .to_string()
                            })?
                            .clone()
                    };

                    RendererPipelineStepObject::Draw {
                        renderer_layer,
                        viewport_start_ndc,
                        viewport_end_ndc,
                    }
                }
            };

            self.renderer_pipeline_steps.push(step_object);
        }

        Ok(())
    }

    fn create_renderer_layer(
        &mut self,
        camera: ArcRwLock<dyn RendererCamera>,
    ) -> Result<ArcRwLock<dyn RendererLayer>, String> {
        let camera = {
            let index = self
                .get_camera_index(&camera)
                .map_err(|e| format!("Creating renderer layer, msg = {e}"))?;

            &self
                .renderer_cameras
                .get_ref(index.0)
                .ok_or_else(|| {
                    "Creating renderer layer, msg = could not find RendererCamera".to_string()
                })?
                .0
        };

        let renderer_layer = Arc::new(RwLock::new(RendererLayerObject::new(camera.clone())));
        let index = self.renderer_layers.create_object(renderer_layer);

        Ok(Arc::new(RwLock::new(RendererLayerIndex(index))))
    }

    fn release_renderer_layer(
        &mut self,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        let index = self
            .get_renderer_layer_index(&renderer_layer)
            .map_err(|e| format!("Releasing renderer layer, msg = {e}"))?;

        self.renderer_layers
            .release_object(index.0)
            .ok_or_else(|| {
                "Releasing renderer layer, msg = could not find RendererLayer".to_string()
            })
            .map(|_| ())
    }

    fn add_renderer_group_to_layer(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        let renderer_group_index = self
            .get_renderer_group_index(&renderer_group)
            .map_err(|e| format!("Adding renderer group to layer, msg = {e}"))?;

        let renderer_group = self
            .renderer_groups
            .get_ref(renderer_group_index.0)
            .ok_or_else(|| {
                "Adding renderer group to layer, msg = could not find RendererGroup".to_string()
            })?;

        let renderer_layer_index = self
            .get_renderer_layer_index(&renderer_layer)
            .map_err(|e| format!("Adding renderer group to layer, msg = {e}"))?;

        let renderer_layer = self
            .renderer_layers
            .get_ref(renderer_layer_index.0)
            .ok_or_else(|| {
                "Adding renderer group to layer, msg = could not find RendererLayer".to_string()
            })?;

        if renderer_layer
            .write()
            .add_renderer_group(renderer_group.clone())
            .is_some()
        {
            Err("Adding renderer object to group, msg = cannot add renderer object twice to the same group".to_string())
        } else {
            Ok(())
        }
    }

    fn remove_renderer_group_from_layer(
        &mut self,
        renderer_group: ArcRwLock<dyn RendererGroup>,
        renderer_layer: ArcRwLock<dyn RendererLayer>,
    ) -> Result<(), String> {
        let renderer_group_index = self
            .get_renderer_group_index(&renderer_group)
            .map_err(|e| format!("Removing renderer group from layer, msg = {e}"))?;

        let renderer_group = self
            .renderer_groups
            .get_ref(renderer_group_index.0)
            .ok_or_else(|| {
                "Removing renderer group from layer, msg = could not find RendererGroup".to_string()
            })?;

        let renderer_layer_index = self
            .get_renderer_layer_index(&renderer_layer)
            .map_err(|e| format!("Removing renderer group from layer, msg = {e}"))?;

        let renderer_layer = self
            .renderer_layers
            .get_ref(renderer_layer_index.0)
            .ok_or_else(|| {
                "Removing renderer group from layer, msg = could not find RendererLayer".to_string()
            })?;

        if renderer_layer
            .write()
            .remove_renderer_group(renderer_group)
            .is_none()
        {
            Err(
                "Removing renderer group from layer, msg = could not find renderer group in layer"
                    .to_string(),
            )
        } else {
            Ok(())
        }
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
        let renderer_transform = Observable::new(RendererTransformObject { transform });
        let index = self
            .renderer_transforms
            .create_object(Arc::new(RwLock::new(renderer_transform)));

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

        let transform = self.renderer_transforms.get_mut(index.0).ok_or_else(|| {
            "Updating transform, msg = could not find RendererTransform".to_string()
        })?;

        transform.write().borrow_mut().transform = new_transform;

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

            self.renderer_transforms.get_ref(index.0).ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find RendererTransform"
                    .to_string()
            })?
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
            gl_drawable_mesh: GLDrawableMesh::new(
                mesh.read().gl_mesh().clone(),
                material.read().gl_material().clone(),
                transform.read().transform,
                shader,
            ),
        }));

        let index = self.mesh_renderer_objects.create_object((
            mesh_renderer_object.clone(),
            transform.write().observe(move |transform| {
                mesh_renderer_object
                    .write()
                    .gl_drawable_mesh
                    .set_transform(&transform.transform);
            }),
        ));

        Ok(Arc::new(RwLock::new(RendererObjectIndex::Mesh(index))))
    }

    fn release_renderer_object(
        &mut self,
        renderer_object: ArcRwLock<dyn RendererObject>,
    ) -> Result<(), String> {
        let index = self
            .get_renderer_object_index(&renderer_object)
            .map_err(|e| format!("Releasing renderer object, msg = {e}"))?;

        match index {
            RendererObjectIndex::Mesh(index) => {
                self.mesh_renderer_objects
                    .release_object(index)
                    .ok_or_else(|| {
                        "Releasing renderer object, msg = could not find RendererObject".to_string()
                    })?;
            }
        }

        Ok(())
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
                let (renderer_object, _observers) = self
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
                let (renderer_object, _observers) = self
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

    fn create_camera(
        &mut self,
        renderer_transform: ArcRwLock<dyn RendererTransform>,
    ) -> Result<ArcRwLock<dyn RendererCamera>, String> {
        let transform = {
            let index = self
                .get_transform_index(&renderer_transform)
                .map_err(|e| format!("Creating renderer object from mesh, msg = {e}"))?;

            self.renderer_transforms.get_ref(index.0).ok_or_else(|| {
                "Creating renderer object from mesh, msg = could not find RendererTransform"
                    .to_string()
            })?
        };

        let camera = Arc::new(RwLock::new(RendererCameraObject {
            transform: transform.read().transform,
        }));

        let index = self.renderer_cameras.create_object((
            camera.clone(),
            transform.write().observe(move |transform| {
                camera.write().transform = transform.transform;
            }),
        ));

        Ok(Arc::new(RwLock::new(RendererCameraIndex(index))))
    }

    fn release_camera(&mut self, camera: ArcRwLock<dyn RendererCamera>) -> Result<(), String> {
        let index = self
            .get_camera_index(&camera)
            .map_err(|e| format!("Releasing camera, msg = {e}"))?;

        self.renderer_cameras
            .release_object(index.0)
            .ok_or_else(|| "Releasing camera, msg = could not find RendererCamera".to_string())
            .map(|_| ())
    }
}
