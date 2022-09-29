use std::sync::{mpsc, Arc};

use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2};

use crate::muleengine::{
    camera::Camera, drawable_object::DrawableObject,
    drawable_object_storage::DrawableObjectStorage,
    renderer::RendererClient as MuleEngineRendererClient, result_option_inspect::ResultInspector,
    system_container, window_context::WindowContext,
};

enum Command {
    AddDrawableObject {
        drawable_object: Arc<RwLock<dyn DrawableObject>>,
        transform: Transform<f32, f32, f32>,
    },
    SetCamera {
        camera: Camera,
    },
    SetWindowDimensions {
        dimensions: Vec2<usize>,
    },
}

type CommandSender = mpsc::Sender<Command>;
type CommandReceiver = mpsc::Receiver<Command>;

pub struct Renderer {
    drawable_object_storage: DrawableObjectStorage,
    command_receiver: CommandReceiver,
    command_sender: CommandSender,
    camera: Camera,
    projection_matrix: Mat4<f32>,
    window_dimensions: Vec2<usize>,
    window_context: Arc<RwLock<dyn WindowContext>>,
}

pub struct RendererClient {
    command_sender: CommandSender,
}

impl Renderer {
    pub fn new(
        initial_window_dimensions: Vec2<usize>,
        window_context: Arc<RwLock<dyn WindowContext>>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();

        let mut ret = Self {
            drawable_object_storage: DrawableObjectStorage::new(),
            command_receiver: receiver,
            command_sender: sender,
            camera: Camera::new(),
            projection_matrix: Mat4::identity(),
            window_dimensions: Vec2::zero(),
            window_context,
        };

        ret.set_window_dimensions(initial_window_dimensions);

        ret
    }

    pub fn client(&self) -> RendererClient {
        RendererClient {
            command_sender: self.command_sender.clone(),
        }
    }

    fn set_window_dimensions(&mut self, window_dimensions: Vec2<usize>) {
        self.window_dimensions = window_dimensions;

        let fov_y_degrees = 45.0f32;
        let near_plane = 0.01;
        let far_plane = 1000.0;
        self.projection_matrix = Mat4::perspective_fov_rh_zo(
            fov_y_degrees.to_radians(),
            window_dimensions.x as f32,
            window_dimensions.y as f32,
            near_plane,
            far_plane,
        );

        unsafe {
            gl::Viewport(0, 0, window_dimensions.x as i32, window_dimensions.y as i32);
        }
    }

    fn execute_command_queue(&mut self) {
        while let Ok(command) = self.command_receiver.try_recv() {
            match command {
                Command::AddDrawableObject {
                    drawable_object,
                    transform,
                } => {
                    self.drawable_object_storage
                        .add_drawable_object(drawable_object, transform);
                }
                Command::SetCamera { camera } => {
                    self.camera = camera;
                }
                Command::SetWindowDimensions { dimensions } => {
                    self.set_window_dimensions(dimensions);
                }
            }
        }
    }
}

impl system_container::System for Renderer {
    fn tick(&mut self, _delta_time_in_secs: f32) {
        self.execute_command_queue();

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Enable(gl::DEPTH_TEST);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let view_matrix = self.camera.compute_view_matrix();
        self.drawable_object_storage.render_all(
            &self.camera.transform.position,
            &self.projection_matrix,
            &view_matrix,
        );

        self.window_context.read().swap_buffers();
    }
}

impl MuleEngineRendererClient for RendererClient {
    fn add_drawable_object(
        &self,
        drawable_object: Arc<RwLock<dyn DrawableObject>>,
        transform: Transform<f32, f32, f32>,
    ) {
        let _ = self
            .command_sender
            .send(Command::AddDrawableObject {
                drawable_object,
                transform,
            })
            .inspect_err(|e| log::error!("Adding drawable object to renderer, error = {e}"));
    }

    fn set_camera(&self, camera: Camera) {
        let _ = self
            .command_sender
            .send(Command::SetCamera { camera })
            .inspect_err(|e| log::error!("Setting camera of renderer, error = {e}"));
    }

    fn set_window_dimensions(&self, dimensions: Vec2<usize>) {
        let _ = self
            .command_sender
            .send(Command::SetWindowDimensions { dimensions })
            .inspect_err(|e| log::error!("Setting window dimensions of renderer, error = {e}"));
    }
}
