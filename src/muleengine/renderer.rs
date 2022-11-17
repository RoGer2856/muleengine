use std::sync::{mpsc as std_mpsc, Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc;
use vek::{Transform, Vec2};

use super::{
    camera::Camera,
    drawable_object::DrawableObject,
    drawable_object_storage::{DrawableObjectStorage, DrawableObjectStorageIndex},
    mesh::{Material, Mesh},
    result_option_inspect::ResultInspector,
    system_container::System,
};

pub trait RendererImpl {
    fn render(&mut self);

    fn set_window_dimensions(&mut self, dimensions: Vec2<usize>);
    fn set_camera(&mut self, camera: Camera);

    fn add_drawable_object(
        &mut self,
        drawable_object: &Arc<RwLock<dyn DrawableObject>>,
        transform: Transform<f32, f32, f32>,
    );
    fn remove_drawable_object(&mut self, drawable_object: &Arc<RwLock<dyn DrawableObject>>);

    fn create_drawable_mesh(
        &mut self,
        mesh: Arc<Mesh>,
        material: Option<Material>,
        shader_path: String,
    ) -> Arc<RwLock<dyn DrawableObject>>;
}

enum Command {
    AddDrawableObject {
        drawable_object_id: DrawableObjectStorageIndex,
        transform: Transform<f32, f32, f32>,
    },
    CreateDrawableMesh {
        mesh: Arc<Mesh>,
        material: Option<Material>,
        shader_path: String,
        result_sender: std_mpsc::Sender<DrawableObjectStorageIndex>,
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

    drawable_object_storage: DrawableObjectStorage,

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

            drawable_object_storage: DrawableObjectStorage::new(),

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
                    material,
                    shader_path,
                    result_sender,
                } => {
                    let drawable_object =
                        self.renderer_impl
                            .create_drawable_mesh(mesh, material, shader_path);

                    let index = self
                        .drawable_object_storage
                        .add_drawable_object(drawable_object.clone());

                    let _ = result_sender
                        .send(index)
                        .inspect_err(|e| log::error!("CreateDrawableMesh response error = {e:?}"));
                }
                Command::AddDrawableObject {
                    drawable_object_id,
                    transform,
                } => {
                    if let Some(drawable_object) =
                        self.drawable_object_storage.get(drawable_object_id)
                    {
                        self.renderer_impl
                            .add_drawable_object(drawable_object, transform);
                    }
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
    pub fn add_drawable_object(
        &self,
        drawable_object_id: DrawableObjectStorageIndex,
        transform: Transform<f32, f32, f32>,
    ) {
        let _ = self
            .command_sender
            .send(Command::AddDrawableObject {
                drawable_object_id,
                transform,
            })
            .inspect_err(|e| log::error!("Adding drawable object to renderer, error = {e}"));
    }

    pub fn create_drawable_mesh(
        &self,
        mesh: Arc<Mesh>,
        material: Option<Material>,
        shader_path: String,
    ) -> DrawableObjectStorageIndex {
        let (result_sender, result_receiver) = std_mpsc::channel();
        let _ = self
            .command_sender
            .send(Command::CreateDrawableMesh {
                mesh,
                material,
                shader_path,
                result_sender,
            })
            .inspect_err(|e| log::error!("Creating drawable object, error = {e}"));

        match result_receiver
            .recv()
            .inspect_err(|e| log::error!("Add drawable object response error = {e}"))
        {
            Ok(index) => index,
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
