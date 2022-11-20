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
    DrawableMeshId, DrawableObjectId, MaterialId, RendererError, RendererMaterial, RendererMesh,
    RendererObject, RendererShader, ShaderId,
};

pub struct Renderer {
    renderer_impl: Box<dyn RendererImpl>,

    materials: ObjectPool<Arc<RwLock<dyn RendererMaterial>>>,
    shaders: ObjectPool<Arc<RwLock<dyn RendererShader>>>,
    drawable_meshes: ObjectPool<Arc<RwLock<dyn RendererMesh>>>,
    drawable_objects: ObjectPool<Arc<RwLock<dyn RendererObject>>>,

    command_receiver: CommandReceiver,
    command_sender: CommandSender,
}

impl Renderer {
    pub fn new(renderer_impl: impl RendererImpl + 'static) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            renderer_impl: Box::new(renderer_impl),

            materials: ObjectPool::new(),
            shaders: ObjectPool::new(),
            drawable_meshes: ObjectPool::new(),
            drawable_objects: ObjectPool::new(),

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
                        .map(|material| MaterialId(self.materials.create_object(material.clone())))
                        .map_err(RendererError::RendererImplError);

                    let _ = result_sender
                        .send(ret)
                        .inspect_err(|e| log::error!("CreateMaterial response error = {e:?}"));
                }
                Command::CreateShader {
                    shader_name,
                    result_sender,
                } => {
                    let ret = self
                        .renderer_impl
                        .create_shader(shader_name)
                        .map(|shader| ShaderId(self.shaders.create_object(shader.clone())))
                        .map_err(RendererError::RendererImplError);

                    let _ = result_sender
                        .send(ret)
                        .inspect_err(|e| log::error!("CreateShader response error = {e:?}"));
                }
                Command::CreateDrawableMesh {
                    mesh,
                    result_sender,
                } => {
                    let ret = self
                        .renderer_impl
                        .create_drawable_mesh(mesh)
                        .map(|drawable_mesh| {
                            DrawableMeshId(
                                self.drawable_meshes.create_object(drawable_mesh.clone()),
                            )
                        })
                        .map_err(RendererError::RendererImplError);

                    let _ = result_sender
                        .send(ret)
                        .inspect_err(|e| log::error!("CreateDrawableMesh response error = {e:?}"));
                }
                Command::CreateDrawableObjectFromMesh {
                    mesh_id,
                    shader_id,
                    material_id,
                    result_sender,
                } => {
                    // the closure helps the readability of the code by enabling the usage of the ? operator
                    let mut closure = || {
                        let drawable_mesh = self
                            .drawable_meshes
                            .get_ref(mesh_id.0)
                            .ok_or(RendererError::InvalidDrawableMeshId(mesh_id))?;

                        let shader = self
                            .shaders
                            .get_ref(shader_id.0)
                            .ok_or(RendererError::InvalidShaderId(shader_id))?;

                        let material = self
                            .materials
                            .get_ref(material_id.0)
                            .ok_or(RendererError::InvalidMaterialId(material_id))?;

                        self.renderer_impl
                            .create_drawable_object_from_mesh(drawable_mesh, shader, material)
                            .map(|drawable_object| {
                                DrawableObjectId(
                                    self.drawable_objects.create_object(drawable_object.clone()),
                                )
                            })
                            .map_err(RendererError::RendererImplError)
                    };

                    let _ = result_sender.send(closure()).inspect_err(|e| {
                        log::error!("CreateDrawableObjectFromMesh response error = {e:?}")
                    });
                }
                Command::AddDrawableObject {
                    drawable_object_id,
                    transform,
                    result_sender,
                } => {
                    let ret = if let Some(drawable_object) =
                        self.drawable_objects.get_ref(drawable_object_id.0)
                    {
                        self.renderer_impl
                            .add_drawable_object(drawable_object, transform)
                            .map_err(RendererError::RendererImplError)
                    } else {
                        Err(RendererError::InvalidDrawableObjectId(drawable_object_id))
                    };

                    let _ = result_sender
                        .send(ret)
                        .inspect_err(|e| log::error!("AddDrawableObject response error = {e:?}"));
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
