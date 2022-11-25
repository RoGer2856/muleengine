use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::mpsc;

use crate::{
    containers::object_pool::ObjectPool, result_option_inspect::ResultInspector,
    system_container::System,
};

use super::{
    renderer_client::RendererClient,
    renderer_command::{Command, CommandReceiver, CommandSender},
    renderer_impl::RendererImpl,
    MaterialHandler, MeshHandler, RendererError, RendererMaterial, RendererMesh, RendererObject,
    RendererObjectHandler, RendererShader, ShaderHandler,
};

pub struct Renderer {
    pub(super) renderer_impl: Box<dyn RendererImpl>,

    pub(super) renderer_materials: ObjectPool<Arc<RwLock<dyn RendererMaterial>>>,
    pub(super) renderer_shaders: ObjectPool<Arc<RwLock<dyn RendererShader>>>,
    pub(super) renderer_meshes: ObjectPool<Arc<RwLock<dyn RendererMesh>>>,
    pub(super) renderer_objects: ObjectPool<Arc<RwLock<dyn RendererObject>>>,

    pub(super) command_receiver: CommandReceiver,
    pub(super) command_sender: CommandSender,
}

impl Renderer {
    pub fn new(renderer_impl: impl RendererImpl + 'static) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            renderer_impl: Box::new(renderer_impl),

            renderer_materials: ObjectPool::new(),
            renderer_shaders: ObjectPool::new(),
            renderer_meshes: ObjectPool::new(),
            renderer_objects: ObjectPool::new(),

            command_receiver: receiver,
            command_sender: sender,
        }
    }

    pub fn render(&mut self) {
        self.execute_command_queue();

        self.renderer_impl.render();
    }

    pub fn client(&self) -> RendererClient {
        RendererClient {
            command_sender: self.command_sender.clone(),
        }
    }

    fn execute_command_queue(&mut self) {
        while let Ok(command) = self.command_receiver.try_recv() {
            match command {
                Command::CreateMaterial {
                    material,
                    result_sender,
                } => {
                    let ret = self
                        .renderer_impl
                        .create_material(material)
                        .map(|material| {
                            MaterialHandler::new(
                                self.renderer_materials.create_object(material.clone()),
                                self.command_sender.clone(),
                            )
                        })
                        .map_err(RendererError::RendererImplError);

                    let _ = result_sender
                        .send(ret)
                        .inspect_err(|e| log::error!("CreateMaterial response, error = {e:?}"));
                }
                Command::ReleaseMaterial { object_pool_index } => {
                    let material = self.renderer_materials.release_object(object_pool_index);

                    if let Some(material) = material {
                        let _ = self
                            .renderer_impl
                            .release_material(material.clone())
                            .inspect_err(|e| log::error!("ReleaseMaterial, error = {e}"));
                    } else {
                        log::error!("ReleaseMaterial, error = could not found material");
                    }
                }
                Command::CreateShader {
                    shader_name,
                    result_sender,
                } => {
                    let ret = self
                        .renderer_impl
                        .create_shader(shader_name)
                        .map(|shader| {
                            ShaderHandler::new(
                                self.renderer_shaders.create_object(shader.clone()),
                                self.command_sender.clone(),
                            )
                        })
                        .map_err(RendererError::RendererImplError);

                    let _ = result_sender
                        .send(ret)
                        .inspect_err(|e| log::error!("CreateShader response, error = {e:?}"));
                }
                Command::ReleaseShader { object_pool_index } => {
                    let shader = self.renderer_shaders.release_object(object_pool_index);

                    if let Some(shader) = shader {
                        let _ = self
                            .renderer_impl
                            .release_shader(shader.clone())
                            .inspect_err(|e| log::error!("ReleaseShader, error = {e}"));
                    } else {
                        log::error!("ReleaseShader, error = could not found shader");
                    }
                }
                Command::CreateMesh {
                    mesh,
                    result_sender,
                } => {
                    let ret = self
                        .renderer_impl
                        .create_mesh(mesh)
                        .map(|mesh| {
                            MeshHandler::new(
                                self.renderer_meshes.create_object(mesh.clone()),
                                self.command_sender.clone(),
                            )
                        })
                        .map_err(RendererError::RendererImplError);

                    let _ = result_sender
                        .send(ret)
                        .inspect_err(|e| log::error!("CreateMesh response, error = {e:?}"));
                }
                Command::ReleaseMesh { object_pool_index } => {
                    let mesh = self.renderer_meshes.release_object(object_pool_index);

                    if let Some(mesh) = mesh {
                        let _ = self
                            .renderer_impl
                            .release_mesh(mesh.clone())
                            .inspect_err(|e| log::error!("ReleaseMesh, error = {e}"));
                    } else {
                        log::error!("ReleaseMesh, error = could not found mesh");
                    }
                }
                Command::CreateRendererObjectFromMesh {
                    mesh_handler,
                    shader_handler,
                    material_handler,
                    result_sender,
                } => {
                    // the closure helps the readability of the code by enabling the usage of the ? operator
                    let closure = || {
                        let mesh = self
                            .renderer_meshes
                            .get_ref(mesh_handler.0.object_pool_index)
                            .ok_or(RendererError::InvalidRendererMeshHandler(mesh_handler))?;

                        let shader = self
                            .renderer_shaders
                            .get_ref(shader_handler.0.object_pool_index)
                            .ok_or(RendererError::InvalidRendererShaderHandler(shader_handler))?;

                        let material = self
                            .renderer_materials
                            .get_ref(material_handler.0.object_pool_index)
                            .ok_or(RendererError::InvalidRendererMaterialHandler(
                                material_handler,
                            ))?;

                        self.renderer_impl
                            .create_renderer_object_from_mesh(mesh, shader, material)
                            .map(|renderer_object| {
                                RendererObjectHandler::new(
                                    self.renderer_objects.create_object(renderer_object.clone()),
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
                    let mesh = self.renderer_objects.release_object(object_pool_index);

                    if let Some(renderer_object) = mesh {
                        let _ = self
                            .renderer_impl
                            .release_renderer_object(renderer_object.clone())
                            .inspect_err(|e| log::error!("ReleaseRendererObject, error = {e}"));
                    } else {
                        log::error!(
                            "ReleaseRendererObject, error = could not found renderer object"
                        );
                    }
                }
                Command::AddRendererObject {
                    renderer_object_handler,
                    transform,
                    result_sender,
                } => {
                    let ret = if let Some(renderer_object) = self
                        .renderer_objects
                        .get_ref(renderer_object_handler.0.object_pool_index)
                    {
                        self.renderer_impl
                            .add_renderer_object(renderer_object, transform)
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
                    self.renderer_impl.set_camera(camera);
                }
                Command::SetWindowDimensions { dimensions } => {
                    self.renderer_impl.set_window_dimensions(dimensions);
                }
            }
        }
    }
}

impl System for Renderer {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        self.render();
    }
}
