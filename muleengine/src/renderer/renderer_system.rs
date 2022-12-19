use std::{collections::BTreeSet, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::watch;
use vek::Transform;

use crate::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex},
    mesh::{Material, Mesh},
    prelude::{ArcRwLock, OptionInspector},
    result_option_inspect::ResultInspector,
    system_container::System,
};

use super::{
    renderer_client::RendererClient,
    renderer_command::{Command, CommandReceiver, CommandSender},
    renderer_impl::{RendererImpl, RendererImplAsync},
    renderer_objects::renderer_layer::{RendererLayer, RendererLayerHandler},
    renderer_pipeline_step::RendererPipelineStep,
    renderer_pipeline_step_impl::RendererPipelineStepImpl,
    MaterialHandler, MeshHandler, RendererError, RendererGroup, RendererGroupHandler,
    RendererMaterial, RendererMesh, RendererObject, RendererObjectHandler, RendererShader,
    RendererTransform, ShaderHandler, TransformHandler,
};

pub struct SyncRenderer {
    pub(super) renderer_pri: RendererPri,
    pub(super) renderer_impl: Box<dyn RendererImpl>,
}

pub struct AsyncRenderer {
    pub(super) renderer_pri: RendererPri,
    pub(super) renderer_impl: Box<dyn RendererImplAsync>,
    halt_sender: watch::Sender<bool>,
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

#[derive(Clone)]
pub(super) struct RendererPri {
    pub(super) renderer_layers: ArcRwLock<ObjectPool<RendererLayerData>>,
    pub(super) renderer_groups: ArcRwLock<ObjectPool<RendererGroupData>>,
    pub(super) renderer_transforms: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererTransform>>>,
    pub(super) renderer_materials: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererMaterial>>>,
    pub(super) renderer_shaders: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererShader>>>,
    pub(super) renderer_meshes: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererMesh>>>,
    pub(super) renderer_objects: ArcRwLock<ObjectPool<RendererObjectData>>,

    pub(super) command_receiver: CommandReceiver,
    pub(super) command_sender: CommandSender,
}

impl SyncRenderer {
    pub fn new(renderer_impl: impl RendererImpl + 'static) -> Self {
        Self {
            renderer_pri: RendererPri::new(),
            renderer_impl: Box::new(renderer_impl),
        }
    }

    pub fn render(&mut self) {
        self.renderer_impl.render();
    }

    pub fn client(&self) -> RendererClient {
        self.renderer_pri.client()
    }
}

impl AsyncRenderer {
    pub fn new(number_of_executors: u8, renderer_impl: impl RendererImplAsync) -> Self {
        if number_of_executors == 0 {
            panic!("Number of executors given to Renderer::new_async(..) has to be more than 0");
        }

        let (halt_sender, halt_receiver) = watch::channel(false);

        let ret = Self {
            renderer_pri: RendererPri::new(),
            renderer_impl: Box::new(renderer_impl),
            halt_sender,
        };

        for _ in 0..number_of_executors {
            let renderer_pri = ret.renderer_pri.clone();
            let renderer_impl = ret.renderer_impl.box_clone();
            let halt_receiver = halt_receiver.clone();

            tokio::spawn(async move {
                while {
                    let mut renderer_pri = renderer_pri.clone();
                    let mut renderer_impl = renderer_impl.box_clone();
                    let mut halt_receiver = halt_receiver.clone();

                    let task_result = tokio::spawn(async move {
                        log::info!("Starting AsyncRenderer executor");

                        loop {
                            tokio::select! {
                                command = renderer_pri.command_receiver.recv_async() => {
                                    if let Ok(command) = command {
                                        renderer_pri.execute_command(command, renderer_impl.as_renderer_impl_mut());
                                    } else {
                                        log::error!("ALl the command senders are dropped");
                                        break;
                                    }
                                }
                                should_halt = halt_receiver.changed() => {
                                    let mut should_break = false;
                                    if should_halt.is_err() {
                                        log::error!("AsyncRenderer's halt_sender is closed but it did not send the halt signal");
                                        should_break = true;
                                    }

                                    if *halt_receiver.borrow() {
                                        should_break = true;
                                    }

                                    if should_break {
                                        log::info!("Stopping renderer executor");
                                    }
                                }
                            }
                        }
                    });

                    task_result.await.is_err()
                } {}
            });
        }

        ret
    }

    pub fn render(&mut self) {
        self.renderer_impl.render();
    }

    pub fn client(&self) -> RendererClient {
        self.renderer_pri.client()
    }
}

impl RendererPri {
    pub fn new() -> Self {
        let (sender, receiver) = flume::unbounded();

        Self {
            renderer_layers: Arc::new(RwLock::new(ObjectPool::new())),
            renderer_groups: Arc::new(RwLock::new(ObjectPool::new())),
            renderer_transforms: Arc::new(RwLock::new(ObjectPool::new())),
            renderer_materials: Arc::new(RwLock::new(ObjectPool::new())),
            renderer_shaders: Arc::new(RwLock::new(ObjectPool::new())),
            renderer_meshes: Arc::new(RwLock::new(ObjectPool::new())),
            renderer_objects: Arc::new(RwLock::new(ObjectPool::new())),

            command_receiver: receiver,
            command_sender: sender,
        }
    }

    pub fn client(&self) -> RendererClient {
        RendererClient {
            command_sender: self.command_sender.clone(),
        }
    }

    fn set_renderer_pipeline(
        &mut self,
        steps: Vec<RendererPipelineStep>,
        renderer_impl: &mut dyn RendererImpl,
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
                    }
                }
            };

            steps_impl.push(step_impl);
        }

        renderer_impl
            .set_renderer_pipeline(steps_impl)
            .map_err(RendererError::RendererImplError)
    }

    fn create_renderer_layer(
        &mut self,
        renderer_impl: &mut dyn RendererImpl,
    ) -> Result<RendererLayerHandler, RendererError> {
        renderer_impl
            .create_renderer_layer()
            .map(|renderer_layer| {
                RendererLayerHandler::new(
                    self.renderer_layers
                        .write()
                        .create_object(RendererLayerData {
                            renderer_layer,
                            added_renderer_groups: BTreeSet::new(),
                        }),
                    self.command_sender.clone(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    fn release_renderer_layer(
        &mut self,
        object_pool_index: ObjectPoolIndex,
        renderer_impl: &mut dyn RendererImpl,
    ) {
        let renderer_layer_data = self
            .renderer_layers
            .write()
            .release_object(object_pool_index);

        if let Some(renderer_layer_data) = renderer_layer_data {
            for renderer_group_index in renderer_layer_data.added_renderer_groups {
                self.renderer_groups
                    .write()
                    .get_mut(renderer_group_index)
                    .inspect_none(|| log::warn!("ReleaseRendererLayer, msg = found invalid renderer group index"))
                    .map(|renderer_group_data| {
                        let _ = renderer_impl.remove_renderer_group_from_layer(
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

            let _ = renderer_impl
                .release_renderer_layer(renderer_layer_data.renderer_layer.clone())
                .inspect_err(|e| log::error!("ReleaseRendererLayer, msg = {e}"));
        } else {
            log::error!("ReleaseRendererLayer, msg = could not find renderer layer");
        }
    }

    fn create_renderer_group(
        &mut self,
        renderer_impl: &mut dyn RendererImpl,
    ) -> Result<RendererGroupHandler, RendererError> {
        renderer_impl
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
                    self.command_sender.clone(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    fn release_renderer_group(
        &mut self,
        object_pool_index: ObjectPoolIndex,
        renderer_impl: &mut dyn RendererImpl,
    ) {
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
                        let _ = renderer_impl.remove_renderer_object_from_group(
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
                        let _ = renderer_impl.remove_renderer_group_from_layer(
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

            let _ = renderer_impl
                .release_renderer_group(renderer_group_data.renderer_group.clone())
                .inspect_err(|e| log::error!("ReleaseRendererGroup, msg = {e}"));
        } else {
            log::error!("ReleaseRendererGroup, msg = could not find renderer group");
        }
    }

    fn create_transform(
        &mut self,
        transform: Transform<f32, f32, f32>,
        renderer_impl: &mut dyn RendererImpl,
    ) -> Result<TransformHandler, RendererError> {
        renderer_impl
            .create_transform(transform)
            .map(|transform| {
                TransformHandler::new(
                    self.renderer_transforms
                        .write()
                        .create_object(transform.clone()),
                    self.command_sender.clone(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    fn update_transform(
        &mut self,
        transform_handler: TransformHandler,
        new_transform: Transform<f32, f32, f32>,
        renderer_impl: &mut dyn RendererImpl,
    ) -> Result<(), RendererError> {
        let transform = self
            .renderer_transforms
            .read()
            .get_ref(transform_handler.0.object_pool_index)
            .ok_or(RendererError::InvalidRendererTransformHandler(
                transform_handler,
            ))?
            .clone();

        renderer_impl
            .update_transform(transform, new_transform)
            .map_err(RendererError::RendererImplError)
    }

    fn release_transform(
        &mut self,
        object_pool_index: ObjectPoolIndex,
        renderer_impl: &mut dyn RendererImpl,
    ) {
        let transform = self
            .renderer_transforms
            .write()
            .release_object(object_pool_index);

        if let Some(transform) = transform {
            let _ = renderer_impl
                .release_transform(transform.clone())
                .inspect_err(|e| log::error!("ReleaseTransform, msg = {e}"));
        } else {
            log::error!("ReleaseTransform, msg = could not find transform");
        }
    }

    fn create_material(
        &mut self,
        material: Material,
        renderer_impl: &mut dyn RendererImpl,
    ) -> Result<MaterialHandler, RendererError> {
        renderer_impl
            .create_material(material)
            .map(|material| {
                MaterialHandler::new(
                    self.renderer_materials
                        .write()
                        .create_object(material.clone()),
                    self.command_sender.clone(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    fn release_material(
        &mut self,
        object_pool_index: ObjectPoolIndex,
        renderer_impl: &mut dyn RendererImpl,
    ) {
        let material = self
            .renderer_materials
            .write()
            .release_object(object_pool_index);

        if let Some(material) = material {
            let _ = renderer_impl
                .release_material(material.clone())
                .inspect_err(|e| log::error!("ReleaseMaterial, msg = {e}"));
        } else {
            log::error!("ReleaseMaterial, msg = could not find material");
        }
    }

    fn create_shader(
        &mut self,
        shader_name: String,
        renderer_impl: &mut dyn RendererImpl,
    ) -> Result<ShaderHandler, RendererError> {
        renderer_impl
            .create_shader(shader_name)
            .map(|shader| {
                ShaderHandler::new(
                    self.renderer_shaders.write().create_object(shader.clone()),
                    self.command_sender.clone(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    fn release_shader(
        &mut self,
        object_pool_index: ObjectPoolIndex,
        renderer_impl: &mut dyn RendererImpl,
    ) {
        let shader = self
            .renderer_shaders
            .write()
            .release_object(object_pool_index);

        if let Some(shader) = shader {
            let _ = renderer_impl
                .release_shader(shader.clone())
                .inspect_err(|e| log::error!("ReleaseShader, msg = {e}"));
        } else {
            log::error!("ReleaseShader, msg = could not find shader");
        }
    }

    fn create_mesh(
        &mut self,
        mesh: Arc<Mesh>,
        renderer_impl: &mut dyn RendererImpl,
    ) -> Result<MeshHandler, RendererError> {
        renderer_impl
            .create_mesh(mesh)
            .map(|mesh| {
                MeshHandler::new(
                    self.renderer_meshes.write().create_object(mesh.clone()),
                    self.command_sender.clone(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    fn release_mesh(
        &mut self,
        object_pool_index: ObjectPoolIndex,
        renderer_impl: &mut dyn RendererImpl,
    ) {
        let mesh = self
            .renderer_meshes
            .write()
            .release_object(object_pool_index);

        if let Some(mesh) = mesh {
            let _ = renderer_impl
                .release_mesh(mesh.clone())
                .inspect_err(|e| log::error!("ReleaseMesh, msg = {e}"));
        } else {
            log::error!("ReleaseMesh, msg = could not find mesh");
        }
    }

    fn create_renderer_object_from_mesh(
        &mut self,
        mesh_handler: MeshHandler,
        shader_handler: ShaderHandler,
        material_handler: MaterialHandler,
        transform_handler: TransformHandler,
        renderer_impl: &mut dyn RendererImpl,
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

        renderer_impl
            .create_renderer_object_from_mesh(mesh, shader, material, transform)
            .map(|renderer_object| {
                RendererObjectHandler::new(
                    self.renderer_objects
                        .write()
                        .create_object(RendererObjectData {
                            renderer_object,
                            contained_by_renderer_groups: BTreeSet::new(),
                        }),
                    self.command_sender.clone(),
                )
            })
            .map_err(RendererError::RendererImplError)
    }

    fn release_renderer_object(
        &mut self,
        object_pool_index: ObjectPoolIndex,
        renderer_impl: &mut dyn RendererImpl,
    ) {
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
                        let _ = renderer_impl.remove_renderer_object_from_group(
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

            let _ = renderer_impl
                .release_renderer_object(renderer_object_data.renderer_object.clone())
                .inspect_err(|e| log::error!("ReleaseRendererObject, msg = {e}"));
        } else {
            log::error!("ReleaseRendererObject, msg = could not find renderer object");
        }
    }

    fn add_renderer_group_to_layer(
        &mut self,
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
        renderer_impl: &mut dyn RendererImpl,
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

        renderer_impl
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

    fn remove_renderer_group_from_layer(
        &mut self,
        renderer_group_handler: RendererGroupHandler,
        renderer_layer_handler: RendererLayerHandler,
        renderer_impl: &mut dyn RendererImpl,
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

        renderer_impl
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

    fn add_renderer_object_to_group(
        &mut self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
        renderer_impl: &mut dyn RendererImpl,
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

        renderer_impl
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

    fn remove_renderer_object_from_group(
        &mut self,
        renderer_object_handler: RendererObjectHandler,
        renderer_group_handler: RendererGroupHandler,
        renderer_impl: &mut dyn RendererImpl,
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

        renderer_impl
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

    fn execute_command(&mut self, command: Command, renderer_impl: &mut dyn RendererImpl) {
        match command {
            Command::SetRendererPipeline {
                steps,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.set_renderer_pipeline(steps, renderer_impl))
                    .inspect_err(|e| log::error!("SetRendererPipeline response, msg = {e:?}"));
            }
            Command::CreateRendererLayer { result_sender } => {
                let _ = result_sender
                    .send(self.create_renderer_layer(renderer_impl))
                    .inspect_err(|e| log::error!("CreateRendererLayer response, msg = {e:?}"));
            }
            Command::ReleaseRendererLayer { object_pool_index } => {
                self.release_renderer_layer(object_pool_index, renderer_impl);
            }
            Command::CreateRendererGroup { result_sender } => {
                let _ = result_sender
                    .send(self.create_renderer_group(renderer_impl))
                    .inspect_err(|e| log::error!("CreateRendererGroup response, msg = {e:?}"));
            }
            Command::ReleaseRendererGroup { object_pool_index } => {
                self.release_renderer_group(object_pool_index, renderer_impl);
            }
            Command::CreateTransform {
                transform,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.create_transform(transform, renderer_impl))
                    .inspect_err(|e| log::error!("CreateTransform response, msg = {e:?}"));
            }
            Command::UpdateTransform {
                transform_handler,
                new_transform,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.update_transform(transform_handler, new_transform, renderer_impl))
                    .inspect_err(|e| log::error!("UpdateTransform response, msg = {e:?}"));
            }
            Command::ReleaseTransform { object_pool_index } => {
                self.release_transform(object_pool_index, renderer_impl);
            }
            Command::CreateMaterial {
                material,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.create_material(material, renderer_impl))
                    .inspect_err(|e| log::error!("CreateMaterial response, msg = {e:?}"));
            }
            Command::ReleaseMaterial { object_pool_index } => {
                self.release_material(object_pool_index, renderer_impl);
            }
            Command::CreateShader {
                shader_name,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.create_shader(shader_name, renderer_impl))
                    .inspect_err(|e| log::error!("CreateShader response, msg = {e:?}"));
            }
            Command::ReleaseShader { object_pool_index } => {
                self.release_shader(object_pool_index, renderer_impl);
            }
            Command::CreateMesh {
                mesh,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.create_mesh(mesh, renderer_impl))
                    .inspect_err(|e| log::error!("CreateMesh response, msg = {e:?}"));
            }
            Command::ReleaseMesh { object_pool_index } => {
                self.release_mesh(object_pool_index, renderer_impl);
            }
            Command::CreateRendererObjectFromMesh {
                mesh_handler,
                shader_handler,
                material_handler,
                transform_handler,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.create_renderer_object_from_mesh(
                        mesh_handler,
                        shader_handler,
                        material_handler,
                        transform_handler,
                        renderer_impl,
                    ))
                    .inspect_err(|e| {
                        log::error!("CreateRendererObjectFromMesh response, msg = {e:?}")
                    });
            }
            Command::ReleaseRendererObject { object_pool_index } => {
                self.release_renderer_object(object_pool_index, renderer_impl);
            }
            Command::AddRendererGroupToLayer {
                renderer_group_handler,
                renderer_layer_handler,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.add_renderer_group_to_layer(
                        renderer_group_handler,
                        renderer_layer_handler,
                        renderer_impl,
                    ))
                    .inspect_err(|e| log::error!("AddRendererGroupToLayer response, msg = {e:?}"));
            }
            Command::RemoveRendererGroupFromLayer {
                renderer_group_handler,
                renderer_layer_handler,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.remove_renderer_group_from_layer(
                        renderer_group_handler,
                        renderer_layer_handler,
                        renderer_impl,
                    ))
                    .inspect_err(|e| {
                        log::error!("RemoveRendererGroupFromLayer response, msg = {e:?}")
                    });
            }
            Command::AddRendererObjectToGroup {
                renderer_object_handler,
                renderer_group_handler,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.add_renderer_object_to_group(
                        renderer_object_handler,
                        renderer_group_handler,
                        renderer_impl,
                    ))
                    .inspect_err(|e| log::error!("AddRendererObjectToGroup response, msg = {e:?}"));
            }
            Command::RemoveRendererObjectFromGroup {
                renderer_object_handler,
                renderer_group_handler,
                result_sender,
            } => {
                let _ = result_sender
                    .send(self.remove_renderer_object_from_group(
                        renderer_object_handler,
                        renderer_group_handler,
                        renderer_impl,
                    ))
                    .inspect_err(|e| {
                        log::error!("RemoveRendererObjectFromGroup response, msg = {e:?}")
                    });
            }
            Command::SetCamera { camera } => {
                renderer_impl.set_camera(camera);
            }
            Command::SetWindowDimensions { dimensions } => {
                renderer_impl.set_window_dimensions(dimensions);
            }
        }
    }

    fn execute_command_queue(&mut self, renderer_impl: &mut dyn RendererImpl) {
        while let Ok(command) = self.command_receiver.try_recv() {
            self.execute_command(command, renderer_impl);
        }
    }
}

impl System for SyncRenderer {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        self.renderer_pri
            .execute_command_queue(self.renderer_impl.as_mut());

        self.render();
    }
}

impl System for AsyncRenderer {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        self.render();
    }
}

impl Drop for AsyncRenderer {
    fn drop(&mut self) {
        let _ = self
            .halt_sender
            .send(true)
            .inspect_err(|_| log::error!("AsyncRenderer is dropped but no executors are running"));
    }
}
