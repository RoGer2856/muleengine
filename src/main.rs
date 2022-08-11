use std::fs::read_to_string;

use game_2::sdl2_opengl_engine::{
    self,
    opengl_utils::{
        index_buffer_object::{IndexBufferObject, PrimitiveMode},
        shader::{Shader, ShaderType},
        shader_program::ShaderProgram,
        vertex_array_object::VertexArrayObject,
        vertex_buffer_object::{DataCount, DataType, VertexBufferObject},
    },
    GLProfile,
};
use sdl2::event::Event;
use vek::Vec3;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut engine = sdl2_opengl_engine::init("game_2", 800, 600, GLProfile::Core, 4, 0).unwrap();

    // shader initialization
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

    // mesh initialization
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

    let vao = VertexArrayObject::new(|vao_interface| {
        vao_interface.use_index_buffer_object(&index_buffer_object);

        vao_interface.use_vertex_buffer_object(
            &positions_vbo,
            &shader_program.get_attribute_by_name("position").unwrap(),
        );
    });

    'running: loop {
        // draw
        {
            unsafe {
                gl::ClearColor(0.2, 0.2, 0.2, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            shader_program.use_program();
            vao.use_vao();
            index_buffer_object.draw();

            engine.gl_swap_window();
        }

        while let Some(event) = engine.poll_event() {
            log::info!("{:?}", event);
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
    }
}
