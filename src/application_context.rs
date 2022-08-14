use std::{fs::read_to_string, sync::Arc};

use game_2::{
    muleengine::{camera::Camera, mesh::Mesh, mesh_creator},
    sdl2_opengl_engine::opengl_utils::{
        index_buffer_object::{IndexBufferObject, PrimitiveMode},
        shader::{Shader, ShaderType},
        shader_input::{ShaderAttribute, ShaderUniform},
        shader_program::ShaderProgram,
        vertex_array_object::VertexArrayObject,
        vertex_buffer_object::{DataCount, DataType, VertexBufferObject},
    },
};
use vek::{Mat4, Transform, Vec3};

struct GLMesh {
    _mesh: Arc<Mesh>,

    index_buffer_object: IndexBufferObject,
    positions_vbo: VertexBufferObject,
}

struct GLDrawableMesh {
    gl_mesh: Arc<GLMesh>,
    vertex_array_object: VertexArrayObject,
    shader_program_info: Arc<ShaderProgramInfo>,
}

struct Attributes {
    position: ShaderAttribute,
}

struct Uniforms {
    projection_matrix: ShaderUniform,
    object_matrix: ShaderUniform,
    view_matrix: ShaderUniform,
}

struct ShaderProgramInfo {
    shader_program: ShaderProgram,
    uniforms: Uniforms,
    attributes: Attributes,
}

pub struct ApplicationContext {
    drawable_mesh: GLDrawableMesh,
    camera: Camera,
    moving_direction: Vec3<f32>,

    // projection
    window_dimensions: (usize, usize),
    projection_matrix: Mat4<f32>,
    fov_y_degrees: f32,
    near_plane: f32,
    far_plane: f32,
}

impl ApplicationContext {
    pub fn new(initial_window_dimensions: (usize, usize)) -> Self {
        // ShaderProgramInfo
        let shader_program_info = Arc::new({
            let vertex_shader = Shader::new(
                ShaderType::Vertex,
                read_to_string("src/shaders/unlit.vert").unwrap().as_str(),
            )
            .unwrap();
            let fragment_shader = Shader::new(
                ShaderType::Fragment,
                read_to_string("src/shaders/unlit.frag").unwrap().as_str(),
            )
            .unwrap();

            let mut shader_program = ShaderProgram::new();
            shader_program.attach_shader(vertex_shader);
            shader_program.attach_shader(fragment_shader);
            shader_program.link_program().unwrap();

            let attributes = Attributes {
                position: shader_program.get_attribute_by_name("position").unwrap(),
            };

            let uniforms = Uniforms {
                projection_matrix: shader_program
                    .get_uniform_by_name("projectionMatrix")
                    .unwrap(),
                object_matrix: shader_program.get_uniform_by_name("objectMatrix").unwrap(),
                view_matrix: shader_program.get_uniform_by_name("viewMatrix").unwrap(),
            };

            ShaderProgramInfo {
                shader_program,
                uniforms,
                attributes,
            }
        });

        // Mesh initialization
        let mesh = Arc::new(mesh_creator::capsule::create(0.5, 2.0, 16));

        // GLMesh initialization
        let gl_mesh = Arc::new({
            let index_buffer_object = IndexBufferObject::new(
                mesh.get_faces().as_ptr(),
                mesh.get_faces().len(),
                PrimitiveMode::Triangles,
            );
            let positions_vbo = VertexBufferObject::new(
                mesh.get_positions().as_ptr(),
                mesh.get_positions().len(),
                DataType::F32,
                DataCount::Coords3,
            );

            GLMesh {
                index_buffer_object,
                positions_vbo,
                _mesh: mesh,
            }
        });

        // GLDrawableMesh initialization
        let gl_drawable_mesh = {
            let vertex_array_object = VertexArrayObject::new(|vao_interface| {
                vao_interface.use_index_buffer_object(&gl_mesh.index_buffer_object);

                vao_interface.use_vertex_buffer_object(
                    &gl_mesh.positions_vbo,
                    &shader_program_info.attributes.position,
                );
            });

            GLDrawableMesh {
                gl_mesh,
                vertex_array_object,
                shader_program_info,
            }
        };

        // camera
        let camera = Camera::new();

        // projection
        let fov_y_degrees = 45.0f32;
        let near_plane = 0.01;
        let far_plane = 1000.0;
        let projection_matrix = Mat4::perspective_fov_rh_zo(
            fov_y_degrees.to_radians(),
            initial_window_dimensions.0 as f32,
            initial_window_dimensions.1 as f32,
            near_plane,
            far_plane,
        );

        Self {
            drawable_mesh: gl_drawable_mesh,
            camera,
            moving_direction: Vec3::zero(),

            // projection
            window_dimensions: initial_window_dimensions,
            projection_matrix,
            fov_y_degrees,
            near_plane,
            far_plane,
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        let velocity = 0.5;

        self.camera
            .move_by(self.moving_direction * velocity * delta_time);
    }

    pub fn set_moving_direction(&mut self, mut direction: Vec3<f32>) {
        if direction != Vec3::zero() {
            direction.normalize();
        }

        self.moving_direction = direction
    }

    pub fn window_resized(&mut self, width: usize, height: usize) {
        self.window_dimensions = (width, height);

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }

        self.projection_matrix = Mat4::perspective_fov_rh_zo(
            self.fov_y_degrees.to_radians(),
            width as f32,
            height as f32,
            self.near_plane,
            self.far_plane,
        );
    }

    pub fn render(&self) {
        let mut transform = Transform::<f32, f32, f32>::default();
        transform.position.z = -5.0;
        let object_matrix = Into::<Mat4<f32>>::into(transform);

        self.drawable_mesh
            .shader_program_info
            .shader_program
            .use_program();

        self.drawable_mesh
            .shader_program_info
            .uniforms
            .projection_matrix
            .send_uniform_matrix_4fv(self.projection_matrix.as_col_ptr(), 1);

        self.drawable_mesh
            .shader_program_info
            .uniforms
            .object_matrix
            .send_uniform_matrix_4fv(object_matrix.as_col_ptr(), 1);
        self.drawable_mesh
            .shader_program_info
            .uniforms
            .view_matrix
            .send_uniform_matrix_4fv(self.camera.compute_view_matrix().as_col_ptr(), 1);

        self.drawable_mesh.vertex_array_object.use_vao();
        self.drawable_mesh.gl_mesh.index_buffer_object.draw();
    }
}
