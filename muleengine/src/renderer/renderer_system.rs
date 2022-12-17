use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::watch;

use crate::{
    containers::object_pool::ObjectPool, prelude::ArcRwLock,
    result_option_inspect::ResultInspector, system_container::System,
};

use super::{
    renderer_client::RendererClient,
    renderer_command::{Command, CommandReceiver, CommandSender},
    renderer_impl::{RendererImpl, RendererImplAsync},
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

#[derive(Clone)]
pub(super) struct RendererPri {
    pub(super) renderer_groups: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererGroup>>>,
    pub(super) renderer_transforms: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererTransform>>>,
    pub(super) renderer_materials: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererMaterial>>>,
    pub(super) renderer_shaders: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererShader>>>,
    pub(super) renderer_meshes: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererMesh>>>,
    pub(super) renderer_objects: ArcRwLock<ObjectPool<ArcRwLock<dyn RendererObject>>>,

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

    fn execute_command(&mut self, command: Command, renderer_impl: &mut dyn RendererImpl) {
        match command {
            Command::CreateRendererGroup { result_sender } => {
                let ret = renderer_impl
                    .create_renderer_group()
                    .map(|transform| {
                        RendererGroupHandler::new(
                            self.renderer_groups
                                .write()
                                .create_object(transform.clone()),
                            self.command_sender.clone(),
                        )
                    })
                    .map_err(RendererError::RendererImplError);

                let _ = result_sender
                    .send(ret)
                    .inspect_err(|e| log::error!("CreateRendererGroup response, msg = {e:?}"));
            }
            Command::ReleaseRendererGroup { object_pool_index } => {
                let renderer_group = self
                    .renderer_groups
                    .write()
                    .release_object(object_pool_index);

                if let Some(renderer_group) = renderer_group {
                    let _ = renderer_impl
                        .release_renderer_group(renderer_group.clone())
                        .inspect_err(|e| log::error!("ReleaseRendererGroup, msg = {e}"));
                } else {
                    log::error!("ReleaseRendererGroup, msg = could not find renderer group");
                }
            }
            Command::CreateTransform {
                transform,
                result_sender,
            } => {
                let ret = renderer_impl
                    .create_transform(transform)
                    .map(|transform| {
                        TransformHandler::new(
                            self.renderer_transforms
                                .write()
                                .create_object(transform.clone()),
                            self.command_sender.clone(),
                        )
                    })
                    .map_err(RendererError::RendererImplError);

                let _ = result_sender
                    .send(ret)
                    .inspect_err(|e| log::error!("CreateTransform response, msg = {e:?}"));
            }
            Command::UpdateTransform {
                transform_handler,
                new_transform,
                result_sender,
            } => {
                // the closure helps the readability of the code by enabling the usage of the ? operator
                let closure = || {
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
                };

                let _ = result_sender
                    .send(closure())
                    .inspect_err(|e| log::error!("UpdateTransform response, msg = {e:?}"));
            }
            Command::ReleaseTransform { object_pool_index } => {
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
            Command::CreateMaterial {
                material,
                result_sender,
            } => {
                let ret = renderer_impl
                    .create_material(material)
                    .map(|material| {
                        MaterialHandler::new(
                            self.renderer_materials
                                .write()
                                .create_object(material.clone()),
                            self.command_sender.clone(),
                        )
                    })
                    .map_err(RendererError::RendererImplError);

                let _ = result_sender
                    .send(ret)
                    .inspect_err(|e| log::error!("CreateMaterial response, msg = {e:?}"));
            }
            Command::ReleaseMaterial { object_pool_index } => {
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
            Command::CreateShader {
                shader_name,
                result_sender,
            } => {
                let ret = renderer_impl
                    .create_shader(shader_name)
                    .map(|shader| {
                        ShaderHandler::new(
                            self.renderer_shaders.write().create_object(shader.clone()),
                            self.command_sender.clone(),
                        )
                    })
                    .map_err(RendererError::RendererImplError);

                let _ = result_sender
                    .send(ret)
                    .inspect_err(|e| log::error!("CreateShader response, msg = {e:?}"));
            }
            Command::ReleaseShader { object_pool_index } => {
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
            Command::CreateMesh {
                mesh,
                result_sender,
            } => {
                let ret = renderer_impl
                    .create_mesh(mesh)
                    .map(|mesh| {
                        MeshHandler::new(
                            self.renderer_meshes.write().create_object(mesh.clone()),
                            self.command_sender.clone(),
                        )
                    })
                    .map_err(RendererError::RendererImplError);

                let _ = result_sender
                    .send(ret)
                    .inspect_err(|e| log::error!("CreateMesh response, msg = {e:?}"));
            }
            Command::ReleaseMesh { object_pool_index } => {
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
            Command::CreateRendererObjectFromMesh {
                mesh_handler,
                shader_handler,
                material_handler,
                transform_handler,
                result_sender,
            } => {
                // the closure helps the readability of the code by enabling the usage of the ? operator
                let closure = || {
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
                                    .create_object(renderer_object.clone()),
                                self.command_sender.clone(),
                            )
                        })
                        .map_err(RendererError::RendererImplError)
                };

                let _ = result_sender.send(closure()).inspect_err(|e| {
                    log::error!("CreateRendererObjectFromMesh response, msg = {e:?}")
                });
            }
            Command::ReleaseRendererObject { object_pool_index } => {
                let mesh = self
                    .renderer_objects
                    .write()
                    .release_object(object_pool_index);

                if let Some(renderer_object) = mesh {
                    let _ = renderer_impl
                        .release_renderer_object(renderer_object.clone())
                        .inspect_err(|e| log::error!("ReleaseRendererObject, msg = {e}"));
                } else {
                    log::error!("ReleaseRendererObject, msg = could not find renderer object");
                }
            }
            Command::AddRendererObjectToGroup {
                renderer_object_handler,
                renderer_group_handler,
                result_sender,
            } => {
                // the closure helps the readability of the code by enabling the usage of the ? operator
                let closure = || {
                    let renderer_object = self
                        .renderer_objects
                        .read()
                        .get_ref(renderer_object_handler.0.object_pool_index)
                        .ok_or(RendererError::InvalidRendererObjectHandler(
                            renderer_object_handler,
                        ))?
                        .clone();

                    let renderer_group = self
                        .renderer_groups
                        .read()
                        .get_ref(renderer_group_handler.0.object_pool_index)
                        .ok_or(RendererError::InvalidRendererGroupHandler(
                            renderer_group_handler,
                        ))?
                        .clone();

                    renderer_impl
                        .add_renderer_object_to_group(renderer_object, renderer_group)
                        .map_err(RendererError::RendererImplError)
                };

                let _ = result_sender
                    .send(closure())
                    .inspect_err(|e| log::error!("AddRendererObjectToGroup response, msg = {e:?}"));
            }
            Command::RemoveRendererObjectFromGroup {
                renderer_object_handler,
                renderer_group_handler,
                result_sender,
            } => {
                // the closure helps the readability of the code by enabling the usage of the ? operator
                let closure = || {
                    let renderer_object = self
                        .renderer_objects
                        .read()
                        .get_ref(renderer_object_handler.0.object_pool_index)
                        .ok_or(RendererError::InvalidRendererObjectHandler(
                            renderer_object_handler,
                        ))?
                        .clone();

                    let renderer_group = self
                        .renderer_groups
                        .read()
                        .get_ref(renderer_group_handler.0.object_pool_index)
                        .ok_or(RendererError::InvalidRendererGroupHandler(
                            renderer_group_handler,
                        ))?
                        .clone();

                    renderer_impl
                        .remove_renderer_object_from_group(renderer_object, renderer_group)
                        .map_err(RendererError::RendererImplError)
                };

                let _ = result_sender.send(closure()).inspect_err(|e| {
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
