use std::sync::{mpsc, Arc};

use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2};

use crate::{
    muleengine::{
        camera::Camera, drawable_object::DrawableObject,
        drawable_object_storage::DrawableObjectStorage, system_container,
    },
    sdl2_opengl_engine::Engine,
};

pub enum Command {
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

pub type CommandSender = mpsc::Sender<Command>;

pub struct System {
    drawable_object_storage: DrawableObjectStorage,
    command_receiver: mpsc::Receiver<Command>,
    command_sender: CommandSender,
    camera: Camera,
    projection_matrix: Mat4<f32>,
    window_dimensions: Vec2<usize>,
    engine: Arc<RwLock<Engine>>,
}

impl System {
    pub fn new(initial_window_dimensions: Vec2<usize>, engine: Arc<RwLock<Engine>>) -> Self {
        let (sender, receiver) = mpsc::channel();

        let mut ret = Self {
            drawable_object_storage: DrawableObjectStorage::new(),
            command_receiver: receiver,
            command_sender: sender,
            camera: Camera::new(),
            projection_matrix: Mat4::identity(),
            window_dimensions: Vec2::zero(),
            engine,
        };

        ret.set_window_dimensions(initial_window_dimensions);

        ret
    }

    pub fn get_sender(&self) -> CommandSender {
        self.command_sender.clone()
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

impl system_container::System for System {
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

        self.engine.write().gl_swap_window();
    }
}
