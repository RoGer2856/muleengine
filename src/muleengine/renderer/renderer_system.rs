use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::mpsc;

use crate::muleengine::{
    containers::object_pool::ObjectPool, result_option_inspect::ResultInspector,
    system_container::System,
};

use super::{
    renderer_client::RendererClient,
    renderer_command::{Command, CommandReceiver, CommandSender},
    renderer_impl::RendererImpl,
    DrawableMesh, DrawableMeshId, DrawableObject, DrawableObjectId, RendererError,
};

pub struct Renderer {
    renderer_impl: Box<dyn RendererImpl>,

    drawable_meshes: ObjectPool<Arc<RwLock<dyn DrawableMesh>>>,
    drawable_objects: ObjectPool<Arc<RwLock<dyn DrawableObject>>>,

    command_receiver: CommandReceiver,
    command_sender: CommandSender,
}

impl Renderer {
    pub fn new(renderer_impl: impl RendererImpl + 'static) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            renderer_impl: Box::new(renderer_impl),

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
                Command::CreateDrawableMesh {
                    mesh,
                    result_sender,
                } => {
                    let ret = match self.renderer_impl.create_drawable_mesh(mesh) {
                        Ok(drawable_mesh) => {
                            let index = self.drawable_meshes.create_object(drawable_mesh.clone());
                            Ok(DrawableMeshId(index))
                        }
                        Err(err) => Err(RendererError::RendererImplError(err)),
                    };

                    let _ = result_sender
                        .send(ret)
                        .inspect_err(|e| log::error!("CreateDrawableMesh response error = {e:?}"));
                }
                Command::CreateDrawableObjectFromMesh {
                    mesh_id,
                    material,
                    shader_path,
                    result_sender,
                } => {
                    let ret = if let Some(drawable_mesh) = self.drawable_meshes.get_ref(mesh_id.0) {
                        match self.renderer_impl.create_drawable_object_from_mesh(
                            drawable_mesh,
                            material,
                            shader_path,
                        ) {
                            Ok(drawable_object) => {
                                let index =
                                    self.drawable_objects.create_object(drawable_object.clone());
                                Ok(DrawableObjectId(index))
                            }
                            Err(err) => Err(RendererError::RendererImplError(err)),
                        }
                    } else {
                        Err(RendererError::InvalidDrawableMeshId(mesh_id))
                    };

                    let _ = result_sender.send(ret).inspect_err(|e| {
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
