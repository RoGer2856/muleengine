use std::{fs::read_to_string, sync::Arc};

use game_2::{
    muleengine::camera::Camera,
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

struct Mesh {
    _indices: Vec<u32>,
    _positions: Vec<Vec3<f32>>,

    index_buffer_object: IndexBufferObject,
    positions_vbo: VertexBufferObject,
}

struct MeshObject {
    _transform: Transform<f32, f32, f32>,
    object_matrix: Mat4<f32>,
    mesh: Arc<Mesh>,
}

struct DrawableMesh {
    mesh_object: Arc<MeshObject>,
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
    drawable_mesh: DrawableMesh,
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
        // shader program info
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

        // mesh initialization
        let mesh = Arc::new({
            let indices = vec![0, 1, 2];
            let positions: Vec<Vec3<f32>> = vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
            ];

            let index_buffer_object =
                IndexBufferObject::new(indices.as_ptr(), indices.len(), PrimitiveMode::Triangles);
            let positions_vbo = VertexBufferObject::new(
                positions.as_ptr(),
                positions.len(),
                DataType::F32,
                DataCount::Coords3,
            );

            Mesh {
                _indices: indices,
                _positions: positions,
                index_buffer_object,
                positions_vbo,
            }
        });

        // mesh object initialization
        let mesh_object = Arc::new({
            let mut transform = Transform::<f32, f32, f32>::default();
            transform.position.z = -1.0;
            let object_matrix = Into::<Mat4<f32>>::into(transform);

            MeshObject {
                _transform: transform,
                object_matrix,
                mesh,
            }
        });

        // drawable mesh initialization
        let drawable_mesh = {
            let vertex_array_object = VertexArrayObject::new(|vao_interface| {
                vao_interface.use_index_buffer_object(&mesh_object.mesh.index_buffer_object);

                vao_interface.use_vertex_buffer_object(
                    &mesh_object.mesh.positions_vbo,
                    &shader_program_info.attributes.position,
                );
            });

            DrawableMesh {
                mesh_object,
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
            drawable_mesh,
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
            .send_uniform_matrix_4fv(self.drawable_mesh.mesh_object.object_matrix.as_col_ptr(), 1);
        self.drawable_mesh
            .shader_program_info
            .uniforms
            .view_matrix
            .send_uniform_matrix_4fv(self.camera.compute_view_matrix().as_col_ptr(), 1);

        self.drawable_mesh.vertex_array_object.use_vao();
        self.drawable_mesh
            .mesh_object
            .mesh
            .index_buffer_object
            .draw();
    }
}
