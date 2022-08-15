pub mod gl_mesh;
pub mod gl_mesh_shader_program;
pub mod gl_shader_program_container;
pub mod opengl_utils;

use sdl2::event::Event;
use sdl2::keyboard::KeyboardState;
use sdl2::mouse::MouseState;
use sdl2::video::{GLContext, Window, WindowBuildError};
use sdl2::{video, EventPump, Sdl, VideoSubsystem};

pub struct Engine {
    _sdl_context: Sdl,
    _sdl_video: VideoSubsystem,
    _gl_context: GLContext,
    sdl_window: Window,
    event_pump: EventPump,
}

#[derive(Debug)]
pub enum ContextCreationError {
    CouldNotCreateSdlContext(String),
    CouldNotCreateVideoSystem(String),
    CouldNotCreateGLContext(String),
    CouldNotCreateEventPump(String),
    CouldNotBuildWindow(WindowBuildError),
    CouldNotCreateContextWithGLVersion {
        gl_profile: GLProfile,
        gl_major_version: u8,
        gl_minor_version: u8,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum GLProfile {
    Core,
    Compatibility,
    GLES,
}

impl From<GLProfile> for video::GLProfile {
    fn from(gl_profile: GLProfile) -> video::GLProfile {
        match gl_profile {
            GLProfile::Compatibility => video::GLProfile::Compatibility,
            GLProfile::Core => video::GLProfile::Core,
            GLProfile::GLES => video::GLProfile::GLES,
        }
    }
}

pub fn init(
    window_name: &str,
    window_width: u32,
    window_height: u32,
    gl_profile: GLProfile,
    gl_major_version: u8,
    gl_minor_version: u8,
) -> Result<Engine, ContextCreationError> {
    let sdl2_gl_profile = gl_profile.into();
    let sdl_context =
        sdl2::init().map_err(|e| ContextCreationError::CouldNotCreateSdlContext(e))?;
    let sdl_video = sdl_context
        .video()
        .map_err(|e| ContextCreationError::CouldNotCreateVideoSystem(e))?;

    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_context_profile(sdl2_gl_profile);
    gl_attr.set_context_version(gl_major_version, gl_minor_version);

    let sdl_window = sdl_video
        .window(window_name, window_width, window_height)
        .opengl()
        .build()
        .map_err(|e| ContextCreationError::CouldNotBuildWindow(e))?;

    let gl_context = sdl_window
        .gl_create_context()
        .map_err(|e| ContextCreationError::CouldNotCreateGLContext(e))?;
    gl::load_with(|name| sdl_video.gl_get_proc_address(name) as *const _);

    if gl_attr.context_profile() != sdl2_gl_profile {
        Err(ContextCreationError::CouldNotCreateContextWithGLVersion {
            gl_profile,
            gl_major_version,
            gl_minor_version,
        })
    } else {
        let event_pump = sdl_context
            .event_pump()
            .map_err(|e| ContextCreationError::CouldNotCreateEventPump(e))?;

        Ok(Engine {
            _sdl_context: sdl_context,
            sdl_window,
            _sdl_video: sdl_video,
            _gl_context: gl_context,
            event_pump,
        })
    }
}

impl Engine {
    pub fn poll_event(&mut self) -> Option<Event> {
        self.event_pump.poll_event()
    }

    pub fn keyboard_state(&self) -> KeyboardState {
        self.event_pump.keyboard_state()
    }

    pub fn mouse_state(&self) -> MouseState {
        self.event_pump.mouse_state()
    }

    pub fn gl_swap_window(&mut self) {
        self.sdl_window.gl_swap_window()
    }
}
