use std::sync::Arc;

use crate::gl_shader_program::{GLShaderProgram, GLShaderProgramError};

use super::opengl_utils::shader_input::{ShaderAttribute, ShaderUniform};

pub(super) struct Attributes {
    pub(super) position: Option<ShaderAttribute>,
    pub(super) normal: Option<ShaderAttribute>,
    pub(super) tangent: Option<ShaderAttribute>,
    pub(super) uv_channels: Option<ShaderAttribute>,
    pub(super) bone_ids: Option<ShaderAttribute>,
    pub(super) bone_weights: Option<ShaderAttribute>,
}

pub(super) struct Uniforms {
    pub(super) eye_position: Option<ShaderUniform>,
    pub(super) object_matrix: Option<ShaderUniform>,
    pub(super) view_matrix: Option<ShaderUniform>,
    pub(super) projection_matrix: Option<ShaderUniform>,
    pub(super) normal_matrix: Option<ShaderUniform>,
    pub(super) bones: Option<ShaderUniform>,

    pub(super) use_albedo_texture: Option<ShaderUniform>,
    pub(super) albedo_texture: Option<ShaderUniform>,
    pub(super) albedo_texture_uv_channel_id: Option<ShaderUniform>,

    pub(super) use_normal_texture: Option<ShaderUniform>,
    pub(super) normal_texture: Option<ShaderUniform>,
    pub(super) normal_texture_uv_channel_id: Option<ShaderUniform>,

    pub(super) use_displacement_texture: Option<ShaderUniform>,
    pub(super) displacement_texture: Option<ShaderUniform>,
    pub(super) displacement_texture_uv_channel_id: Option<ShaderUniform>,

    pub(super) opacity: Option<ShaderUniform>,
    pub(super) albedo_color: Option<ShaderUniform>,
    pub(super) emissive_color: Option<ShaderUniform>,
    pub(super) shininess_color: Option<ShaderUniform>,
}

pub struct GLMeshShaderProgram {
    pub(super) gl_shader_program: Arc<GLShaderProgram>,
    pub(super) uniforms: Uniforms,
    pub(super) attributes: Attributes,
}

impl GLMeshShaderProgram {
    pub fn new(gl_shader_program: Arc<GLShaderProgram>) -> Result<Self, GLShaderProgramError> {
        let attributes = Attributes {
            position: gl_shader_program
                .shader_program
                .get_attribute_by_name("position"),
            normal: gl_shader_program
                .shader_program
                .get_attribute_by_name("normal"),
            tangent: gl_shader_program
                .shader_program
                .get_attribute_by_name("tangent"),
            uv_channels: gl_shader_program
                .shader_program
                .get_attribute_by_name("uvChannels"),
            bone_ids: gl_shader_program
                .shader_program
                .get_attribute_by_name("boneIds"),
            bone_weights: gl_shader_program
                .shader_program
                .get_attribute_by_name("boneWeights"),
        };

        let uniforms = Uniforms {
            eye_position: gl_shader_program
                .shader_program
                .get_uniform_by_name("eyePosition"),
            object_matrix: gl_shader_program
                .shader_program
                .get_uniform_by_name("objectMatrix"),
            view_matrix: gl_shader_program
                .shader_program
                .get_uniform_by_name("viewMatrix"),
            projection_matrix: gl_shader_program
                .shader_program
                .get_uniform_by_name("projectionMatrix"),
            normal_matrix: gl_shader_program
                .shader_program
                .get_uniform_by_name("normalMatrix"),
            bones: gl_shader_program
                .shader_program
                .get_uniform_by_name("bones"),

            use_albedo_texture: gl_shader_program
                .shader_program
                .get_uniform_by_name("useAlbedoTexture"),
            albedo_texture: gl_shader_program
                .shader_program
                .get_uniform_by_name("albedoTexture"),
            albedo_texture_uv_channel_id: gl_shader_program
                .shader_program
                .get_uniform_by_name("albedoTextureUvChannelId"),

            use_normal_texture: gl_shader_program
                .shader_program
                .get_uniform_by_name("useNormalTexture"),
            normal_texture: gl_shader_program
                .shader_program
                .get_uniform_by_name("normalTexture"),
            normal_texture_uv_channel_id: gl_shader_program
                .shader_program
                .get_uniform_by_name("normalTextureUvChannelId"),

            use_displacement_texture: gl_shader_program
                .shader_program
                .get_uniform_by_name("useDisplacementTexture"),
            displacement_texture: gl_shader_program
                .shader_program
                .get_uniform_by_name("displacementTexture"),
            displacement_texture_uv_channel_id: gl_shader_program
                .shader_program
                .get_uniform_by_name("displacementTextureUvChannelId"),

            opacity: gl_shader_program
                .shader_program
                .get_uniform_by_name("opacity"),
            albedo_color: gl_shader_program
                .shader_program
                .get_uniform_by_name("albedoColor"),
            emissive_color: gl_shader_program
                .shader_program
                .get_uniform_by_name("emissiveColor"),
            shininess_color: gl_shader_program
                .shader_program
                .get_uniform_by_name("shininessColor"),
        };

        Ok(Self {
            gl_shader_program,
            uniforms,
            attributes,
        })
    }

    pub fn get_shader_base_path(&self) -> &String {
        self.gl_shader_program.get_shader_base_path()
    }
}
