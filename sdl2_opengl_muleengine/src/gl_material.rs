use std::sync::Arc;

use vek::Vec3;

use muleengine::{
    image_container::ImageContainerError,
    mesh::{Material, MaterialTexture, MaterialTextureType, TextureMapMode},
    prelude::ResultInspector,
};

use super::{
    gl_texture_container::GLTextureContainer,
    opengl_utils::texture_2d::{GLTextureMapMode, Texture2D},
};

pub struct GLMaterialTexture {
    pub texture: Arc<Texture2D>,
    pub texture_type: MaterialTextureType,
    pub texture_map_mode: GLTextureMapMode,
    pub uv_channel_id: usize,
    pub blend: f32,
}

pub struct GLMaterial {
    pub opacity: f32,
    pub albedo_color: Vec3<f32>,
    pub emissive_color: Vec3<f32>,
    pub shininess_color: Vec3<f32>,
    pub textures: Vec<GLMaterialTexture>,
}

pub struct RendererMaterialObject {
    gl_material: Arc<GLMaterial>,
}

impl GLMaterial {
    pub fn new(material: &Material, gl_texture_container: &mut GLTextureContainer) -> Self {
        let mut textures = Vec::new();

        for texture in material.textures.iter() {
            let material_texture = GLMaterialTexture::new(texture, gl_texture_container);
            let material_texture = material_texture
                .inspect_err(|e| log::error!("Could not load texture for material, msg = '{e:?}'"));
            if let Ok(material_texture) = material_texture {
                textures.push(material_texture);
            }
        }

        Self {
            opacity: material.opacity,
            albedo_color: material.albedo_color,
            emissive_color: material.emissive_color,
            shininess_color: material.shininess_color,
            textures,
        }
    }
}

impl GLMaterialTexture {
    pub fn new(
        texture: &MaterialTexture,
        gl_texture_container: &mut GLTextureContainer,
    ) -> Result<Self, ImageContainerError> {
        let texture_map_mode = match texture.texture_map_mode {
            TextureMapMode::Clamp => GLTextureMapMode::Clamp,
            TextureMapMode::Repeat => GLTextureMapMode::Repeat,
            TextureMapMode::Mirror => GLTextureMapMode::Mirror,
        };

        Ok(Self {
            texture: gl_texture_container.get_texture(texture.image.clone()?),
            texture_type: texture.texture_type,
            texture_map_mode,
            blend: texture.blend,
            uv_channel_id: texture.uv_channel_id,
        })
    }
}

impl RendererMaterialObject {
    pub fn new(gl_mesh: Arc<GLMaterial>) -> Self {
        Self {
            gl_material: gl_mesh,
        }
    }

    pub fn gl_material(&self) -> &Arc<GLMaterial> {
        &self.gl_material
    }
}
