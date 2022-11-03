use std::sync::{mpsc, Arc};

use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec2};

use crate::{
    muleengine::{
        asset_container::AssetContainer,
        camera::Camera,
        drawable_object_storage::{DrawableObjectStorage, DrawableObjectStorageIndex},
        mesh::{Material, Mesh},
        renderer::RendererClient as MuleEngineRendererClient,
        result_option_inspect::ResultInspector,
        system_container,
        window_context::WindowContext,
    },
    sdl2_opengl_engine::{
        gl_material::GLMaterial, gl_mesh_container::GLMeshContainer,
        gl_shader_program_container::GLShaderProgramContainer,
        gl_texture_container::GLTextureContainer,
    },
};

enum Command {
    AddDrawableMesh {
        mesh: Arc<Mesh>,
        transform: Transform<f32, f32, f32>,
        material: Option<Material>,
        shader_path: String,
        result_sender: mpsc::Sender<DrawableObjectStorageIndex>,
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

    asset_container: AssetContainer,

    gl_mesh_container: GLMeshContainer,
    gl_shader_program_container: GLShaderProgramContainer,
    gl_texture_container: GLTextureContainer,
}

#[derive(Clone)]
pub struct RendererClient {
    command_sender: CommandSender,
}

impl Renderer {
    pub fn new(
        initial_window_dimensions: Vec2<usize>,
        window_context: Arc<RwLock<dyn WindowContext>>,
        asset_container: AssetContainer,
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

            asset_container,

            gl_mesh_container: GLMeshContainer::new(),
            gl_shader_program_container: GLShaderProgramContainer::new(),
            gl_texture_container: GLTextureContainer::new(),
        };

        ret.set_window_dimensions(initial_window_dimensions);

        ret
    }

    pub fn client(&self) -> Box<dyn MuleEngineRendererClient> {
        Box::new(RendererClient {
            command_sender: self.command_sender.clone(),
        })
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
                Command::AddDrawableMesh {
                    mesh,
                    transform,
                    material,
                    shader_path,
                    result_sender,
                } => {
                    let gl_mesh_shader_program =
                        match self.gl_shader_program_container.get_mesh_shader_program(
                            &shader_path,
                            &mut self.asset_container.asset_reader().write(),
                        ) {
                            Ok(shader_program) => shader_program,
                            Err(_) => todo!(),
                        };

                    let drawable_mesh = self.gl_mesh_container.get_drawable_mesh(
                        gl_mesh_shader_program,
                        mesh,
                        &mut self.gl_texture_container,
                    );

                    if let Some(material) = material {
                        let material = GLMaterial::new(&material, &mut self.gl_texture_container);
                        drawable_mesh.write().material = Some(material);
                    }

                    let index = self
                        .drawable_object_storage
                        .add_drawable_object(drawable_mesh, transform);
                    let _ = result_sender
                        .send(index)
                        .inspect_err(|e| log::error!("AddDrawableObject response error = {e}"));
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
    fn add_drawable_mesh(
        &self,
        mesh: Arc<Mesh>,
        transform: Transform<f32, f32, f32>,
        material: Option<Material>,
        shader_path: String,
    ) -> DrawableObjectStorageIndex {
        let (result_sender, result_receiver) = mpsc::channel();
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

        DrawableObjectStorageIndex::invalid()

        // match result_receiver
        //     .recv()
        //     .inspect_err(|e| log::error!("Add drawable object response error = {e}"))
        // {
        //     Ok(index) => index,
        //     Err(_) => unreachable!(),
        // }
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
