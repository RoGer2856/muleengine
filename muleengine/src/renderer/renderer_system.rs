use std::{collections::BTreeSet, sync::Arc};

use bytifex_utils::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex},
    sync::types::{arc_rw_lock_new, ArcRwLock},
};
use method_taskifier::{
    method_taskifier_impl,
    task_channel::{TaskReceiver, TaskSender},
    AsyncWorkerRunner, InvalidNumberOfExecutors,
};
use option_inspect_none::OptionInspectNone;
use vek::Transform;

use crate::{
    mesh::{Material, Mesh},
    system_container::System,
    window_context::{Event, EventReceiver, WindowContext},
};

use super::{
    renderer_impl::{RendererImpl, RendererImplAsync},
    renderer_objects::{
        renderer_camera::RendererCameraHandler,
        renderer_layer::{RendererLayer, RendererLayerHandler},
    },
    renderer_pipeline_step::RendererPipelineStep,
    renderer_pipeline_step_impl::RendererPipelineStepImpl,
    RendererCamera, RendererError, RendererGroup, RendererGroupHandler, RendererMaterial,
    RendererMaterialHandler, RendererMesh, RendererMeshHandler, RendererObject,
    RendererObjectHandler, RendererShader, RendererShaderHandler, RendererTransform,
    RendererTransformHandler,
};

pub struct SyncRenderer {
    pub(super) renderer_pri: RendererPri<dyn RendererImpl>,
    _window_context: ArcRwLock<dyn WindowContext>,
    event_receiver: EventReceiver,
}

pub struct AsyncRenderer {
    pub(super) renderer_pri: RendererPri<dyn RendererImplAsync>,
    renderer_impl: Box<dyn RendererImplAsync>,
    worker_runner: AsyncWorkerRunner,
    _window_context: ArcRwLock<dyn WindowContext>,
}

pub(super) struct RendererLayerData {
    pub(super) renderer_layer: ArcRwLock<dyn RendererLayer>,
    pub(super) added_renderer_groups: BTreeSet<ObjectPoolIndex>,
}

pub(super) struct RendererGroupData {
    pub(super) renderer_group: ArcRwLock<dyn RendererGroup>,
    pub(super) added_renderer_objects: BTreeSet<ObjectPoolIndex>,
    pub(super) contained_by_renderer_layers: BTreeSet<ObjectPoolIndex>,
}

pub(super) struct RendererObjectData {
    pub(super) renderer_object: ArcRwLock<dyn RendererObject>,
    pub(super) contained_by_renderer_groups: BTreeSet<ObjectPoolIndex>,
}

pub(super) struct RendererPri<T: RendererImpl + ?Sized> {
    pub(super) renderer_cameras: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererCamera>>>,
    pub(super) renderer_layers: ArcRwLock<ObjectPool<RendererLayerData>>,
    pub(super) renderer_groups: ArcRwLock<ObjectPool<RendererGroupData>>,
    pub(super) renderer_transforms: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererTransform>>>,
    pub(super) renderer_materials: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererMaterial>>>,
    pub(super) renderer_shaders: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererShader>>>,
    pub(super) renderer_meshes: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererMesh>>>,
    pub(super) renderer_objects: ArcRwLock<ObjectPool<RendererObjectData>>,

    task_receiver: TaskReceiver<client::ChanneledTask>,
    task_sender: TaskSender<client::ChanneledTask>,

    renderer_impl: Box<T>,
}

impl Clone for RendererPri<dyn RendererImplAsync> {
    fn clone(&self) -> Self {
        Self {
            renderer_cameras: self.renderer_cameras.clone(),
            renderer_layers: self.renderer_layers.clone(),
            renderer_groups: self.renderer_groups.clone(),
            renderer_transforms: self.renderer_transforms.clone(),
            renderer_materials: self.renderer_materials.clone(),
            renderer_shaders: self.renderer_shaders.clone(),
            renderer_meshes: self.renderer_meshes.clone(),
            renderer_objects: self.renderer_objects.clone(),

            task_receiver: self.task_receiver.clone(),
            task_sender: self.task_sender.clone(),

            renderer_impl: self.renderer_impl.box_clone(),
        }
    }
}

impl SyncRenderer {
    pub fn new(
        renderer_impl: impl RendererImpl + 'static,
        window_context: ArcRwLock<dyn WindowContext>,
    ) -> Self {
        Self::new_from_box(Box::new(renderer_impl), window_context)
    }

    pub fn new_from_box(
        renderer_impl: Box<dyn RendererImpl + 'static>,
        window_context: ArcRwLock<dyn WindowContext>,
    ) -> Self {
        let mut renderer_pri = RendererPri::new(renderer_impl);
        let window_dimensions = window_context.read().window_dimensions();
        let _ = renderer_pri
            .window_dimensions_changed(window_dimensions.x, window_dimensions.y)
            .inspect_err(|e| {
                log::error!("SyncRenderer::new_from_box: window_dimensions_changed, error = {e:?}")
            });

        Self {
            renderer_pri,
            _window_context: window_context.clone(),
            event_receiver: window_context.read().event_receiver(),
        }
    }

    pub fn client(&self) -> RendererClient {
        self.renderer_pri.client()
    }
}

impl AsyncRenderer {
    pub fn new(
        number_of_executors: u8,
        renderer_impl: impl RendererImplAsync,
        window_context: ArcRwLock<dyn WindowContext>,
    ) -> Result<Self, InvalidNumberOfExecutors> {
        let renderer_impl = Box::new(renderer_impl);

        let mut renderer_pri = RendererPri::new(renderer_impl.box_clone());
        let window_dimensions = window_context.read().window_dimensions();
        let _ = renderer_pri
            .window_dimensions_changed(window_dimensions.x, window_dimensions.y)
            .inspect_err(|e| {
                log::error!("AsyncRenderer::new: window_dimensions_changed, error = {e:?}")
            });
        let task_receiver = renderer_pri.task_receiver.clone();
        let mut event_receiver = window_context.read().event_receiver();
        event_receiver.stop();

        let worker_runner = {
            let renderer_pri = renderer_pri.clone();
            let task_receiver = task_receiver.clone();
            let mut event_receiver = event_receiver.clone();
            event_receiver.stop();

            AsyncWorkerRunner::run_worker(number_of_executors, move || {
                let mut renderer_pri = renderer_pri.clone();
                let mut task_receiver = task_receiver.clone();
                let event_receiver = event_receiver.clone();

                || async move {
                    let _ = renderer_pri
                        .execute_channeled_tasks_from_queue_until_clients_dropped(
                            &mut task_receiver,
                        )
                        .await;

                    while let Ok(Some(event)) = event_receiver.try_pop() {
                        if let Event::Resized { width, height } = event {
                            let _ = renderer_pri
                                .window_dimensions_changed(width, height)
                                .inspect_err(|e| {
                                    log::error!("AsyncRenderer::tick: window_dimensions_changed, error = {e:?}")
                                });
                        }
                    }
                }
            })?
        };

        Ok(Self {
            renderer_pri,
            renderer_impl,
            worker_runner,
            _window_context: window_context.clone(),
        })
    }

    pub fn client(&self) -> RendererClient {
        self.renderer_pri.client()
    }
}

pub use client::Client as RendererClient;

#[method_taskifier_impl(module_name = client)]
impl<T: RendererImpl + ?Sized> RendererPri<T> {
    pub fn new(renderer_impl: Box<T>) -> Self {
        let (sender, receiver) = client::channel();

        Self {
            renderer_cameras: arc_rw_lock_new(ObjectPool::new()),
            renderer_layers: arc_rw_lock_new(ObjectPool::new()),
            renderer_groups: arc_rw_lock_new(ObjectPool::new()),
            renderer_transforms: arc_rw_lock_new(ObjectPool::new()),
            renderer_materials: arc_rw_lock_new(ObjectPool::new()),
            renderer_shaders: arc_rw_lock_new(ObjectPool::new()),
            renderer_meshes: arc_rw_lock_new(ObjectPool::new()),
            renderer_objects: arc_rw_lock_new(ObjectPool::new()),

            task_receiver: receiver,
            task_sender: sender,

            renderer_impl,
        }
    }

    pub fn window_dimensions_changed(
        &mut self,
        width: usize,
        height: usize,
    ) -> Result<(), RendererError> {
        self.renderer_impl
            .window_dimensions_changed(width, height)
            .map_err(RendererError::RendererImplError)
    }

    pub fn client(&self) -> RendererClient {
        RendererClient::new(self.task_sender.clone())
    }

    #[method_taskifier_worker_fn]
    fn set_renderer_pipeline(
        &mut self,
        steps: Vec<RendererPipelineStep>,
    ) -> Result<(), RendererError> {
        let mut steps_impl = Vec::with_capacity(steps.capacity());
        for step in steps {
            let step_impl = match step {
                RendererPipelineStep::Clear {
                    depth,
                    color,
                    viewport_start_ndc,
                    viewport_end_ndc,
                } => RendererPipelineStepImpl::Clear {
                    depth,
                    color,
                    viewport_start_ndc,
                    viewport_end_ndc,
                },
                RendererPipelineStep::Draw {
                    renderer_layer_handler,
                    viewport_start_ndc,
                    viewport_end_ndc,
                    compute_projection_matrix,
                } => {
                    let renderer_layer = self
                        .renderer_layers
                        .read()
                        .get_ref(renderer_layer_handler.0.object_pool_index)
                        .ok_or_else(|| {
                            RendererError::InvalidRendererLayerHandler(renderer_layer_handler)
                        })?
                        .renderer_layer
                        .clone();

                    RendererPipelineStepImpl::Draw {
                        renderer_layer,
                        viewport_start_ndc,
                        viewport_end_ndc,

                        compute_projection_matrix,
                    }
                }
            };

            steps_impl.push(step_impl);
        }

        self.renderer_impl
            .set_renderer_pipeline(steps_impl)
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn create_renderer_layer(
        &mut self,
        camera_handler: RendererCameraHandler,
    ) -> Result<RendererLayerHandler, RendererError> {
        let camera = self
            .renderer_cameras
            .read()
            .get_ref(camera_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererCameraHandler(camera_handler))?
            .clone();

        self.renderer_impl
            .create_renderer_layer(camera)
            .map(|renderer_layer| {
                RendererLayerHandler::new(
                    self.renderer_layers
                        .write()
                        .create_object(RendererLayerData {
                            renderer_layer,
                            added_renderer_groups: BTreeSet::new(),
                        }),
                    self.client(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn release_renderer_layer(&mut self, object_pool_index: ObjectPoolIndex) {
        let renderer_layer_data = self
            .renderer_layers
            .write()
            .release_object(object_pool_index);

        if let Some(renderer_layer_data) = renderer_layer_data {
            for renderer_group_index in renderer_layer_data.added_renderer_groups {
                self.renderer_groups
                    .write()
                    .get_mut(renderer_group_index)
                    .or_else(|| { log::warn!("ReleaseRendererLayer, msg = found invalid renderer group index"); None })
                    .map(|renderer_group_data| {
                        let _ = self.renderer_impl.remove_renderer_group_from_layer(
                            renderer_group_data.renderer_group.clone(),
                            renderer_layer_data.renderer_layer.clone()
                        ).inspect_err(|e| {
                            log::warn!("ReleaseRendererLayer, removing group from layer, msg = {e}");
                        });

                        if !renderer_group_data.contained_by_renderer_layers.remove(&object_pool_index) {
                            log::warn!("ReleaseRendererLayer, msg = inconsistent state with renderer group");
                        }

                        renderer_group_data
                    });
            }

            let _ = self
                .renderer_impl
                .release_renderer_layer(renderer_layer_data.renderer_layer.clone())
                .inspect_err(|e| log::error!("ReleaseRendererLayer, msg = {e}"));
        } else {
            log::error!("ReleaseRendererLayer, msg = could not find renderer layer");
        }
    }

    #[method_taskifier_worker_fn]
    fn create_renderer_group(&mut self) -> Result<RendererGroupHandler, RendererError> {
        self.renderer_impl
            .create_renderer_group()
            .map(|renderer_group| {
                RendererGroupHandler::new(
                    self.renderer_groups
                        .write()
                        .create_object(RendererGroupData {
                            renderer_group,
                            added_renderer_objects: BTreeSet::new(),
                            contained_by_renderer_layers: BTreeSet::new(),
                        }),
                    self.client(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn release_renderer_group(&mut self, object_pool_index: ObjectPoolIndex) {
        let renderer_group_data = self
            .renderer_groups
            .write()
            .release_object(object_pool_index);

        if let Some(renderer_group_data) = renderer_group_data {
            for renderer_object_index in renderer_group_data.added_renderer_objects {
                self.renderer_objects
                    .write()
                    .get_mut(renderer_object_index)
                    .inspect_none(|| log::warn!("ReleaseRendererGroup, msg = found invalid renderer object index"))
                    .map(|renderer_object_data| {
                        let _ = self.renderer_impl.remove_renderer_object_from_group(
                            renderer_object_data.renderer_object.clone(),
                            renderer_group_data.renderer_group.clone(),
                        ).inspect_err(|e| {
                            log::warn!("ReleaseRendererGroup, removing object from group, msg = {e}");
                        });

                        if !renderer_object_data.contained_by_renderer_groups.remove(&object_pool_index) {
                            log::warn!("ReleaseRendererGroup, msg = inconsistent state with renderer object");
                        }

                        renderer_object_data
                    });
            }

            for renderer_layer_index in renderer_group_data.contained_by_renderer_layers {
                self.renderer_layers
                    .write()
                    .get_mut(renderer_layer_index)
                    .inspect_none(|| log::warn!("ReleaseRendererGroup, msg = found invalid renderer layer index"))
                    .map(|renderer_layer_data| {
                        let _ = self.renderer_impl.remove_renderer_group_from_layer(
                            renderer_group_data.renderer_group.clone(),
                            renderer_layer_data.renderer_layer.clone(),
                        ).inspect_err(|e| {
                            log::warn!("ReleaseRendererGroup, removing group from layer, msg = {e}");
                        });

                        if !renderer_layer_data.added_renderer_groups.remove(&object_pool_index) {
                            log::warn!("ReleaseRendererGroup, msg = inconsistent state with renderer layer");
                        }

                        renderer_layer_data
                    });
            }

            let _ = self
                .renderer_impl
                .release_renderer_group(renderer_group_data.renderer_group.clone())
                .inspect_err(|e| log::error!("ReleaseRendererGroup, msg = {e}"));
        } else {
            log::error!("ReleaseRendererGroup, msg = could not find renderer group");
        }
    }

    #[method_taskifier_worker_fn]
    fn create_transform(
        &mut self,
        transform: Transform<f32, f32, f32>,
    ) -> Result<RendererTransformHandler, RendererError> {
        self.renderer_impl
            .create_transform(transform)
            .map(|transform| {
                RendererTransformHandler::new(
                    self.renderer_transforms.write().create_object(transform),
                    self.client(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn update_transform(
        &mut self,
        transform_handler: RendererTransformHandler,
        new_transform: Transform<f32, f32, f32>,
    ) -> Result<(), RendererError> {
        let transform = self
            .renderer_transforms
            .read()
            .get_ref(transform_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererTransformHandler(
                transform_handler,
            ))?
            .clone();

        self.renderer_impl
            .update_transform(transform, new_transform)
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn release_transform(&mut self, object_pool_index: ObjectPoolIndex) {
        let transform = self
            .renderer_transforms
            .write()
            .release_object(object_pool_index);

        if let Some(transform) = transform {
            let _ = self
                .renderer_impl
                .release_transform(transform)
                .inspect_err(|e| log::error!("ReleaseTransform, msg = {e}"));
        } else {
            log::error!("ReleaseTransform, msg = could not find transform");
        }
    }

    #[method_taskifier_worker_fn]
    fn create_material(
        &mut self,
        material: Material,
    ) -> Result<RendererMaterialHandler, RendererError> {
        self.renderer_impl
            .create_material(material)
            .map(|material| {
                RendererMaterialHandler::new(
                    self.renderer_materials.write().create_object(material),
                    self.client(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn update_material(
        &mut self,
        material_handler: RendererMaterialHandler,
        new_material: Material,
    ) -> Result<(), RendererError> {
        let material = self
            .renderer_materials
            .read()
            .get_ref(material_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererMaterialHandler(
                material_handler,
            ))?
            .clone();

        self.renderer_impl
            .update_material(material, new_material)
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn release_material(&mut self, object_pool_index: ObjectPoolIndex) {
        let material = self
            .renderer_materials
            .write()
            .release_object(object_pool_index);

        if let Some(material) = material {
            let _ = self
                .renderer_impl
                .release_material(material)
                .inspect_err(|e| log::error!("ReleaseMaterial, msg = {e}"));
        } else {
            log::error!("ReleaseMaterial, msg = could not find material");
        }
    }

    #[method_taskifier_worker_fn]
    fn create_shader(
        &mut self,
        shader_name: String,
    ) -> Result<RendererShaderHandler, RendererError> {
        self.renderer_impl
            .create_shader(shader_name)
            .map(|shader| {
                RendererShaderHandler::new(
                    self.renderer_shaders.write().create_object(shader),
                    self.client(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn update_shader(
        &mut self,
        shader_handler: RendererShaderHandler,
        new_shader_name: String,
    ) -> Result<(), RendererError> {
        let shader = self
            .renderer_shaders
            .read()
            .get_ref(shader_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererShaderHandler(shader_handler))?
            .clone();

        self.renderer_impl
            .update_shader(shader, new_shader_name)
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn release_shader(&mut self, object_pool_index: ObjectPoolIndex) {
        let shader = self
            .renderer_shaders
            .write()
            .release_object(object_pool_index);

        if let Some(shader) = shader {
            let _ = self
                .renderer_impl
                .release_shader(shader)
                .inspect_err(|e| log::error!("ReleaseShader, msg = {e}"));
        } else {
            log::error!("ReleaseShader, msg = could not find shader");
        }
    }

    #[method_taskifier_worker_fn]
    fn create_mesh(&mut self, mesh: Arc<Mesh>) -> Result<RendererMeshHandler, RendererError> {
        self.renderer_impl
            .create_mesh(mesh)
            .map(|mesh| {
                RendererMeshHandler::new(
                    self.renderer_meshes.write().create_object(mesh),
                    self.client(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn update_mesh(
        &mut self,
        mesh_handler: RendererMeshHandler,
        new_mesh: Arc<Mesh>,
    ) -> Result<(), RendererError> {
        let mesh = self
            .renderer_meshes
            .read()
            .get_ref(mesh_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererMeshHandler(mesh_handler))?
            .clone();

        self.renderer_impl
            .update_mesh(mesh, new_mesh)
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn release_mesh(&mut self, object_pool_index: ObjectPoolIndex) {
        let mesh = self
            .renderer_meshes
            .write()
            .release_object(object_pool_index);

        if let Some(mesh) = mesh {
            let _ = self
                .renderer_impl
                .release_mesh(mesh)
                .inspect_err(|e| log::error!("ReleaseMesh, msg = {e}"));
        } else {
            log::error!("ReleaseMesh, msg = could not find mesh");
        }
    }

    #[method_taskifier_worker_fn]
    fn create_renderer_object_from_mesh(
        &mut self,
        mesh_handler: RendererMeshHandler,
        shader_handler: RendererShaderHandler,
        material_handler: RendererMaterialHandler,
        transform_handler: RendererTransformHandler,
    ) -> Result<RendererObjectHandler, RendererError> {
        let mesh = self
            .renderer_meshes
            .read()
            .get_ref(mesh_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererMeshHandler(mesh_handler))?
            .clone();

        let shader = self
            .renderer_shaders
            .read()
            .get_ref(shader_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererShaderHandler(shader_handler))?
            .clone();

        let material = self
            .renderer_materials
            .read()
            .get_ref(material_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererMaterialHandler(
                material_handler,
            ))?
            .clone();

        let transform = self
            .renderer_transforms
            .read()
            .get_ref(transform_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererTransformHandler(
                transform_handler,
            ))?
            .clone();

        self.renderer_impl
            .create_renderer_object_from_mesh(mesh, shader, material, transform)
            .map(|renderer_object| {
                RendererObjectHandler::new(
                    self.renderer_objects
                        .write()
                        .create_object(RendererObjectData {
                            renderer_object,
                            contained_by_renderer_groups: BTreeSet::new(),
                        }),
                    self.client(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn release_renderer_object(&mut self, object_pool_index: ObjectPoolIndex) {
        let renderer_object_data = self
            .renderer_objects
            .write()
            .release_object(object_pool_index);

        if let Some(renderer_object_data) = renderer_object_data {
            for renderer_group_index in renderer_object_data.contained_by_renderer_groups {
                self.renderer_groups
                    .write()
                    .get_mut(renderer_group_index)
                    .inspect_none(|| log::warn!("ReleaseRendererObject, msg = found invalid renderer group index"))
                    .map(|renderer_group_data| {
                        let _ = self.renderer_impl.remove_renderer_object_from_group(
                            renderer_object_data.renderer_object.clone(),
                            renderer_group_data.renderer_group.clone()
                        ).inspect_err(|e| {
                            log::warn!("ReleaseRendererObject, removing object from group, msg = {e}");
                        });

                        if !renderer_group_data.added_renderer_objects.remove(&object_pool_index) {
                            log::warn!("ReleaseRendererObject, msg = inconsistent state with renderer group");
                        }

                        renderer_group_data
                    });
            }

            let _ = self
                .renderer_impl
                .release_renderer_object(renderer_object_data.renderer_object.clone())
                .inspect_err(|e| log::error!("ReleaseRendererObject, msg = {e}"));
        } else {
            log::error!("ReleaseRendererObject, msg = could not find renderer object");
        }
    }

    #[method_taskifier_worker_fn]
    fn add_renderer_group_to_layer(
        &mut self,
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
    ) -> Result<(), RendererError> {
        let mut renderer_groups = self.renderer_groups.write();
        let renderer_group_data = renderer_groups
            .get_mut(renderer_group_handler.0.object_pool_index)
            .ok_or_else(|| {
                RendererError::InvalidRendererGroupHandler(renderer_group_handler.clone())
            })?;

        let mut renderer_layers = self.renderer_layers.write();
        let renderer_layer_data = renderer_layers
            .get_mut(renderer_layer_handler.0.object_pool_index)
            .ok_or_else(|| {
                RendererError::InvalidRendererLayerHandler(renderer_layer_handler.clone())
            })?;

        self.renderer_impl
            .add_renderer_group_to_layer(
                renderer_group_data.renderer_group.clone(),
                renderer_layer_data.renderer_layer.clone(),
            )
            .map(|_| {
                renderer_group_data
                    .contained_by_renderer_layers
                    .insert(renderer_layer_handler.0.object_pool_index);
                renderer_layer_data
                    .added_renderer_groups
                    .insert(renderer_group_handler.0.object_pool_index);
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn remove_renderer_group_from_layer(
        &mut self,
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
    ) -> Result<(), RendererError> {
        let mut renderer_groups = self.renderer_groups.write();
        let renderer_group_data = renderer_groups
            .get_mut(renderer_group_handler.0.object_pool_index)
            .ok_or_else(|| {
                RendererError::InvalidRendererGroupHandler(renderer_group_handler.clone())
            })?;

        let mut renderer_layers = self.renderer_layers.write();
        let renderer_layer_data = renderer_layers
            .get_mut(renderer_layer_handler.0.object_pool_index)
            .ok_or_else(|| {
                RendererError::InvalidRendererLayerHandler(renderer_layer_handler.clone())
            })?;

        self.renderer_impl
            .remove_renderer_group_from_layer(
                renderer_group_data.renderer_group.clone(),
                renderer_layer_data.renderer_layer.clone(),
            )
            .map(|_| {
                if !renderer_group_data
                    .contained_by_renderer_layers
                    .remove(&renderer_layer_handler.0.object_pool_index)
                {
                    log::warn!("RemoveRendererGroupFromLayer, msg = inconsistent state");
                }

                if !renderer_layer_data
                    .added_renderer_groups
                    .remove(&renderer_group_handler.0.object_pool_index)
                {
                    log::warn!("RemoveRendererGroupFromLayer, msg = inconsistent state with renderer layer");
                }
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn add_renderer_object_to_group(
        &mut self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
    ) -> Result<(), RendererError> {
        let mut renderer_objects = self.renderer_objects.write();
        let renderer_object_data = renderer_objects
            .get_mut(renderer_object_handler.0.object_pool_index)
            .ok_or_else(|| {
                RendererError::InvalidRendererObjectHandler(renderer_object_handler.clone())
            })?;

        let mut renderer_groups = self.renderer_groups.write();
        let renderer_group_data = renderer_groups
            .get_mut(renderer_group_handler.0.object_pool_index)
            .ok_or_else(|| {
                RendererError::InvalidRendererGroupHandler(renderer_group_handler.clone())
            })?;

        self.renderer_impl
            .add_renderer_object_to_group(
                renderer_object_data.renderer_object.clone(),
                renderer_group_data.renderer_group.clone(),
            )
            .map(|_| {
                renderer_object_data
                    .contained_by_renderer_groups
                    .insert(renderer_group_handler.0.object_pool_index);
                renderer_group_data
                    .added_renderer_objects
                    .insert(renderer_object_handler.0.object_pool_index);
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn remove_renderer_object_from_group(
        &mut self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
    ) -> Result<(), RendererError> {
        let mut renderer_objects = self.renderer_objects.write();
        let renderer_object_data = renderer_objects
            .get_mut(renderer_object_handler.0.object_pool_index)
            .ok_or_else(|| {
                RendererError::InvalidRendererObjectHandler(renderer_object_handler.clone())
            })?;

        let mut renderer_groups = self.renderer_groups.write();
        let renderer_group_data = renderer_groups
            .get_mut(renderer_group_handler.0.object_pool_index)
            .ok_or_else(|| {
                RendererError::InvalidRendererGroupHandler(renderer_group_handler.clone())
            })?;

        self.renderer_impl
            .remove_renderer_object_from_group(
                renderer_object_data.renderer_object.clone(),
                renderer_group_data.renderer_group.clone(),
            )
            .map(|_| {
                if !renderer_object_data
                    .contained_by_renderer_groups
                    .remove(&renderer_group_handler.0.object_pool_index)
                {
                    log::warn!("RemoveRendererObjectFromGroup, msg = inconsistent state with renderer object");
                }

                if !renderer_group_data
                    .added_renderer_objects
                    .remove(&renderer_object_handler.0.object_pool_index)
                {
                    log::warn!("RemoveRendererObjectFromGroup, msg = inconsistent state");
                }
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn create_camera(
        &mut self,
        transform_handler: RendererTransformHandler,
    ) -> Result<RendererCameraHandler, RendererError> {
        let transform = self
            .renderer_transforms
            .read()
            .get_ref(transform_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererTransformHandler(
                transform_handler,
            ))?
            .clone();

        self.renderer_impl
            .create_camera(transform)
            .map(|camera| {
                RendererCameraHandler::new(
                    self.renderer_cameras.write().create_object(camera),
                    self.client(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    #[method_taskifier_worker_fn]
    fn release_camera(&mut self, object_pool_index: ObjectPoolIndex) {
        let camera = self
            .renderer_cameras
            .write()
            .release_object(object_pool_index);

        if let Some(camera) = camera {
            let _ = self
                .renderer_impl
                .release_camera(camera)
                .inspect_err(|e| log::error!("ReleaseCamera, msg = {e}"));
        } else {
            log::error!("ReleaseCamera, msg = could not find camera");
        }
    }
}

impl System for SyncRenderer {
    fn tick(&mut self, _loop_start: &std::time::Instant, _last_loop_time_secs: f32) {
        while let Ok(task) = self.renderer_pri.task_receiver.try_recv() {
            self.renderer_pri.execute_channeled_task(task);
        }

        while let Ok(Some(event)) = self.event_receiver.try_pop() {
            if let Event::Resized { width, height } = event {
                let _ = self
                    .renderer_pri
                    .window_dimensions_changed(width, height)
                    .inspect_err(|e| {
                        log::error!("SyncRenderer::tick: window_dimensions_changed, error = {e:?}")
                    });
            }
        }

        self.renderer_pri.renderer_impl.render();
    }
}

impl System for AsyncRenderer {
    fn tick(&mut self, _loop_start: &std::time::Instant, _last_loop_time_secs: f32) {
        self.renderer_impl.render();
    }
}

impl Drop for AsyncRenderer {
    fn drop(&mut self) {
        self.worker_runner.stop();
    }
}
