use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Mat4, Vec2};

use crate::muleengine::{
    camera::Camera, drawable_object_storage::DrawableObjectStorage, renderer, system_container,
    window_context::WindowContext,
};

pub struct System {
    drawable_object_storage: DrawableObjectStorage,
    command_receiver: renderer::CommandReceiver,
    command_sender: renderer::CommandSender,
    camera: Camera,
    projection_matrix: Mat4<f32>,
    window_dimensions: Vec2<usize>,
    window_context: Arc<RwLock<dyn WindowContext>>,
}

impl System {
    pub fn new(
        initial_window_dimensions: Vec2<usize>,
        window_context: Arc<RwLock<dyn WindowContext>>,
    ) -> Self {
        let (sender, receiver) = renderer::command_channel();

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

    pub fn get_sender(&self) -> renderer::CommandSender {
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
                renderer::Command::AddDrawableObject {
                    drawable_object,
                    transform,
                } => {
                    self.drawable_object_storage
                        .add_drawable_object(drawable_object, transform);
                }
                renderer::Command::SetCamera { camera } => {
                    self.camera = camera;
                }
                renderer::Command::SetWindowDimensions { dimensions } => {
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

        self.window_context.read().swap_buffers();
    }
}
