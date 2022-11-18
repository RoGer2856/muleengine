use std::sync::{mpsc as std_mpsc, Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc;
use vek::{Transform, Vec2};

use super::{
    camera::Camera,
    containers::object_pool::{ObjectPool, ObjectPoolIndex},
    mesh::{Material, Mesh},
    prelude::AsAny,
    result_option_inspect::ResultInspector,
    system_container::System,
};

pub trait DrawableMesh: AsAny + 'static {}
pub trait DrawableObject: AsAny + 'static {}

#[derive(Debug)]
pub struct DrawableMeshId(ObjectPoolIndex);
#[derive(Debug)]
pub struct DrawableObjectId(ObjectPoolIndex);

#[derive(Debug)]
pub enum RendererError {
    InvalidDrawableMeshId(DrawableMeshId),
    InvalidDrawableObjectId(DrawableObjectId),
    RendererImplError(String),
}

pub trait RendererImpl {
    fn render(&mut self);

    fn set_window_dimensions(&mut self, dimensions: Vec2<usize>);
    fn set_camera(&mut self, camera: Camera);

    fn create_drawable_mesh(
        &mut self,
        mesh: Arc<Mesh>,
    ) -> Result<Arc<RwLock<dyn DrawableMesh>>, String>;

    fn create_drawable_object_from_mesh(
        &mut self,
        mesh: &Arc<RwLock<dyn DrawableMesh>>,
        material: Option<Material>,
        shader_path: String,
    ) -> Result<Arc<RwLock<dyn DrawableObject>>, String>;

    fn add_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn DrawableObject>>,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), String>;
    fn remove_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn DrawableObject>>,
    ) -> Result<(), String>;
}

enum Command {
    CreateDrawableMesh {
        mesh: Arc<Mesh>,
        result_sender: std_mpsc::Sender<Result<DrawableMeshId, RendererError>>,
    },
    CreateDrawableObjectFromMesh {
        mesh_id: DrawableMeshId,
        material: Option<Material>,
        shader_path: String,
        result_sender: std_mpsc::Sender<Result<DrawableObjectId, RendererError>>,
    },

    AddDrawableObject {
        drawable_object_id: DrawableObjectId,
        transform: Transform<f32, f32, f32>,
        result_sender: std_mpsc::Sender<Result<(), RendererError>>,
    },

    SetCamera {
        camera: Camera,
    },
    SetWindowDimensions {
        dimensions: Vec2<usize>,
    },
}

type CommandSender = mpsc::UnboundedSender<Command>;
type CommandReceiver = mpsc::UnboundedReceiver<Command>;

pub struct Renderer {
    renderer_impl: Box<dyn RendererImpl>,

    drawable_meshes: ObjectPool<Arc<RwLock<dyn DrawableMesh>>>,
    drawable_objects: ObjectPool<Arc<RwLock<dyn DrawableObject>>>,

    command_receiver: CommandReceiver,
    command_sender: CommandSender,
}

#[derive(Clone)]
pub struct RendererClient {
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

impl RendererClient {
    pub fn create_drawable_mesh(&self, mesh: Arc<Mesh>) -> Result<DrawableMeshId, RendererError> {
        let (result_sender, result_receiver) = std_mpsc::channel();
        let _ = self
            .command_sender
            .send(Command::CreateDrawableMesh {
                mesh,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating drawable mesh, error = {e}"));

        match result_receiver
            .recv()
            .inspect_err(|e| log::error!("Creating drawable mesh response error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub fn create_drawable_object_from_mesh(
        &self,
        mesh_id: DrawableMeshId,
        material: Option<Material>,
        shader_path: String,
    ) -> Result<DrawableObjectId, RendererError> {
        let (result_sender, result_receiver) = std_mpsc::channel();
        let _ = self
            .command_sender
            .send(Command::CreateDrawableObjectFromMesh {
                mesh_id,
                material,
                shader_path,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating drawable object from mesh, error = {e}"));

        match result_receiver
            .recv()
            .inspect_err(|e| log::error!("Creating drawable object from mesh response error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub fn add_drawable_object(
        &self,
        drawable_object_id: DrawableObjectId,
        transform: Transform<f32, f32, f32>,
    ) -> Result<(), RendererError> {
        let (result_sender, result_receiver) = std_mpsc::channel();
        let _ = self
            .command_sender
            .send(Command::AddDrawableObject {
                drawable_object_id,
                transform,
                result_sender,
            })
            .inspect_err(|e| log::error!("Adding drawable object to renderer, error = {e}"));

        match result_receiver
            .recv()
            .inspect_err(|e| log::error!("Adding drawable object to renderer response error = {e}"))
        {
            Ok(ret) => ret,
            Err(_) => unreachable!(),
        }
    }

    pub fn set_camera(&self, camera: Camera) {
        let _ = self
            .command_sender
            .send(Command::SetCamera { camera })
            .inspect_err(|e| log::error!("Setting camera of renderer, error = {e}"));
    }

    pub fn set_window_dimensions(&self, dimensions: Vec2<usize>) {
        let _ = self
            .command_sender
            .send(Command::SetWindowDimensions { dimensions })
            .inspect_err(|e| log::error!("Setting window dimensions of renderer, error = {e}"));
    }
}
