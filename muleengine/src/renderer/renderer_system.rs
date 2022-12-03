use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    containers::object_pool::ObjectPool, prelude::ArcRwLock,
    result_option_inspect::ResultInspector, system_container::System,
};

use super::{
    renderer_client::RendererClient,
    renderer_command::{Command, CommandReceiver, CommandSender},
    renderer_impl::{RendererImpl, RendererImplAsync},
    MaterialHandler, MeshHandler, RendererError, RendererMaterial, RendererMesh, RendererObject,
    RendererObjectHandler, RendererShader, RendererTransform, ShaderHandler, TransformHandler,
};

pub struct SyncRenderer {
    pub(super) renderer_pri: RendererPri,
    pub(super) renderer_impl: Box<dyn RendererImpl>,
}

#[derive(Clone)]
pub struct AsyncRenderer {
    pub(super) renderer_pri: RendererPri,
    pub(super) renderer_impl: Box<dyn RendererImplAsync>,
}

#[derive(Clone)]
pub(super) struct RendererPri {
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
            panic!("Number of executors given to Renderer::new_async(..) has to be greater than 0");
        }

        let ret = Self {
            renderer_pri: RendererPri::new(),
            renderer_impl: Box::new(renderer_impl),
        };

        for _ in 0..number_of_executors {
            todo!("RendererTransform has to be Send");
            let renderer = ret.clone();
            // tokio::spawn(async move {
            //     todo!("implement some stopping mechanism (drop)");
            //     loop {
            //         tokio::select! {
            //             command = renderer.renderer_pri.command_receiver.recv_async() => {
            //                 todo!("handle panick");
            //                 let command = command.unwrap();
            //                 renderer.renderer_pri.execute_command(command, renderer.renderer_impl.as_renderer_impl_mut());
            //             }
            //         }
            //     }
            // });
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
                    .inspect_err(|e| log::error!("CreateTransform response, error = {e:?}"));
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
                    .inspect_err(|e| log::error!("UpdateTransform response, error = {e:?}"));
            }
            Command::ReleaseTransform { object_pool_index } => {
                let transform = self
                    .renderer_transforms
                    .write()
                    .release_object(object_pool_index);

                if let Some(transform) = transform {
                    let _ = renderer_impl
                        .release_transform(transform.clone())
                        .inspect_err(|e| log::error!("ReleaseTransform, error = {e}"));
                } else {
                    log::error!("ReleaseTransform, error = could not find transform");
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
                    .inspect_err(|e| log::error!("CreateMaterial response, error = {e:?}"));
            }
            Command::ReleaseMaterial { object_pool_index } => {
                let material = self
                    .renderer_materials
                    .write()
                    .release_object(object_pool_index);

                if let Some(material) = material {
                    let _ = renderer_impl
                        .release_material(material.clone())
                        .inspect_err(|e| log::error!("ReleaseMaterial, error = {e}"));
                } else {
                    log::error!("ReleaseMaterial, error = could not find material");
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
                    .inspect_err(|e| log::error!("CreateShader response, error = {e:?}"));
            }
            Command::ReleaseShader { object_pool_index } => {
                let shader = self
                    .renderer_shaders
                    .write()
                    .release_object(object_pool_index);

                if let Some(shader) = shader {
                    let _ = renderer_impl
                        .release_shader(shader.clone())
                        .inspect_err(|e| log::error!("ReleaseShader, error = {e}"));
                } else {
                    log::error!("ReleaseShader, error = could not find shader");
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
                    .inspect_err(|e| log::error!("CreateMesh response, error = {e:?}"));
            }
            Command::ReleaseMesh { object_pool_index } => {
                let mesh = self
                    .renderer_meshes
                    .write()
                    .release_object(object_pool_index);

                if let Some(mesh) = mesh {
                    let _ = renderer_impl
                        .release_mesh(mesh.clone())
                        .inspect_err(|e| log::error!("ReleaseMesh, error = {e}"));
                } else {
                    log::error!("ReleaseMesh, error = could not find mesh");
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
                    log::error!("CreateRendererObjectFromMesh response, error = {e:?}")
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
                        .inspect_err(|e| log::error!("ReleaseRendererObject, error = {e}"));
                } else {
                    log::error!("ReleaseRendererObject, error = could not find renderer object");
                }
            }
            Command::AddRendererObject {
                renderer_object_handler,
                result_sender,
            } => {
                let ret = if let Some(renderer_object) = self
                    .renderer_objects
                    .read()
                    .get_ref(renderer_object_handler.0.object_pool_index)
                {
                    renderer_impl
                        .add_renderer_object(renderer_object.clone())
                        .map_err(RendererError::RendererImplError)
                } else {
                    Err(RendererError::InvalidRendererObjectHandler(
                        renderer_object_handler,
                    ))
                };

                let _ = result_sender
                    .send(ret)
                    .inspect_err(|e| log::error!("AddRendererObject response, error = {e:?}"));
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
