use std::sync::{mpsc as std_mpsc, Arc};

use tokio::sync::mpsc;
use vek::{Transform, Vec2};

use super::{
    camera::Camera,
    drawable_object_storage::DrawableObjectStorageIndex,
    mesh::{Material, Mesh},
    result_option_inspect::ResultInspector,
    system_container::System,
};

pub trait RendererImpl {
    fn render(&mut self);
    fn add_drawable_mesh(
        &mut self,
        mesh: Arc<Mesh>,
        transform: Transform<f32, f32, f32>,
        material: Option<Material>,
        shader_path: String,
    ) -> DrawableObjectStorageIndex;
    fn set_camera(&mut self, camera: Camera);
    fn set_window_dimensions(&mut self, dimensions: Vec2<usize>);
}

enum Command {
    AddDrawableMesh {
        mesh: Arc<Mesh>,
        transform: Transform<f32, f32, f32>,
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
                Command::AddDrawableMesh {
                    mesh,
                    transform,
                    material,
                    shader_path,
                    result_sender,
                } => {
                    let index = self.renderer_impl.add_drawable_mesh(
                        mesh,
                        transform,
                        material,
                        shader_path,
                    );
                    let _ = result_sender
                        .send(index)
                        .inspect_err(|e| log::error!("AddDrawableMesh response error = {e:?}"));
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
    pub fn add_drawable_mesh(
        &self,
        mesh: Arc<Mesh>,
        transform: Transform<f32, f32, f32>,
        material: Option<Material>,
        shader_path: String,
    ) -> DrawableObjectStorageIndex {
        let (result_sender, result_receiver) = std_mpsc::channel();
        let _ = self
            .command_sender
            .send(Command::AddDrawableMesh {
                mesh,
                transform,
                material,
                shader_path,
                result_sender,
            })
            .inspect_err(|e| log::error!("Adding drawable object to renderer, error = {e}"));

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
