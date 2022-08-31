use std::{ffi::c_void, sync::Arc};

use gl::types::GLuint;

use crate::muleengine::image::{ColorType, Image};

#[derive(Clone, Copy)]
pub enum GLTextureAnisotropyMode {
    Anisotropy1,
    Anisotropy2,
    Anisotropy4,
    Anisotropy8,
}

#[derive(Clone, Copy)]
pub enum GLTextureMapMode {
    Repeat,
    Clamp,
    Mirror,
}

#[derive(Clone, Copy)]
pub enum GLTextureSamplingMode {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

pub struct Texture2D {
    texture_id: GLuint,
}

fn set_texture_anisotropy_mode(mode: GLTextureAnisotropyMode) {
    const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;

    match mode {
        GLTextureAnisotropyMode::Anisotropy1 => unsafe {
            gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, 1.0);
        },
        GLTextureAnisotropyMode::Anisotropy2 => unsafe {
            gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, 2.0);
        },
        GLTextureAnisotropyMode::Anisotropy4 => unsafe {
            gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, 4.0);
        },
        GLTextureAnisotropyMode::Anisotropy8 => unsafe {
            gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, 8.0);
        },
    }
}

fn set_texture_map_mode(mode: GLTextureMapMode) {
    match mode {
        GLTextureMapMode::Clamp => unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        },
        GLTextureMapMode::Repeat => unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        },
        GLTextureMapMode::Mirror => unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::MIRRORED_REPEAT as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::MIRRORED_REPEAT as i32,
            );
        },
    }
}

fn set_texture_sampling_mode(mode: GLTextureSamplingMode) {
    match mode {
        GLTextureSamplingMode::Nearest => unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        },
        GLTextureSamplingMode::Linear => unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        },
        GLTextureSamplingMode::NearestMipmapNearest => unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_NEAREST as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        },
        GLTextureSamplingMode::LinearMipmapNearest => unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_NEAREST as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        },
        GLTextureSamplingMode::NearestMipmapLinear => unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        },
        GLTextureSamplingMode::LinearMipmapLinear => unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        },
    }
}

impl Texture2D {
    pub fn new(image: Arc<Image>) -> Self {
        let mut texture_id = 0;

        unsafe {
            gl::GenTextures(1, &mut texture_id);
        }

        let (format, data_type) = match image.color_type() {
            ColorType::L8 => (gl::RED, gl::UNSIGNED_BYTE),
            ColorType::La8 => (gl::RG, gl::UNSIGNED_BYTE),
            ColorType::Rgb8 => (gl::RGB, gl::UNSIGNED_BYTE),
            ColorType::Rgba8 => (gl::RGBA, gl::UNSIGNED_BYTE),
            ColorType::L16 => (gl::RED, gl::UNSIGNED_SHORT),
            ColorType::La16 => (gl::RG, gl::UNSIGNED_SHORT),
            ColorType::Rgb16 => (gl::RGB, gl::UNSIGNED_SHORT),
            ColorType::Rgba16 => (gl::RGBA, gl::UNSIGNED_SHORT),
            ColorType::RgbF32 => (gl::RGB, gl::FLOAT),
            ColorType::RgbaF32 => (gl::RGBA, gl::FLOAT),
        };

        // if image.width() % 4 != 0 {
        //     unsafe {
        //         gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        //     }
        // }

        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                format,
                data_type,
                image.as_bytes().as_ptr() as *const c_void,
            );
        }

        // if image.height() % 4 != 0 {
        //     unsafe {
        //         gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
        //     }
        // }

        unsafe {
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        set_texture_anisotropy_mode(GLTextureAnisotropyMode::Anisotropy8);

        set_texture_map_mode(GLTextureMapMode::Repeat);
        set_texture_sampling_mode(GLTextureSamplingMode::LinearMipmapLinear);

        Self { texture_id }
    }

    pub fn use_texture(&self, layer: usize) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + layer as u32);

            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
    }

    pub fn set_texture_map_mode(&self, mode: GLTextureMapMode) {
        unsafe {
            match mode {
                GLTextureMapMode::Clamp => {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                }
                GLTextureMapMode::Repeat => {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                }
                GLTextureMapMode::Mirror => {
                    gl::TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_WRAP_S,
                        gl::MIRRORED_REPEAT as i32,
                    );
                    gl::TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_WRAP_T,
                        gl::MIRRORED_REPEAT as i32,
                    );
                }
            }
        }
    }
}
