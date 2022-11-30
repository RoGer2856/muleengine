use std::sync::Arc;

use vek::{Mat4, Transform, Vec3};

use muleengine::mesh::MaterialTextureType;

use crate::gl_mesh::GLMesh;

use super::{
    gl_material::{GLMaterial, GLMaterialTexture},
    gl_mesh_shader_program::GLMeshShaderProgram,
    opengl_utils::{shader_input::ShaderUniform, vertex_array_object::VertexArrayObject},
};

pub struct GLDrawableMesh {
    gl_mesh: Arc<GLMesh>,
    material: Arc<GLMaterial>,
    object_matrix: Mat4<f32>,
    bone_transforms: Option<Vec<Mat4<f32>>>,
    vertex_array_object: VertexArrayObject,
    gl_mesh_shader_program: Arc<GLMeshShaderProgram>,
}

impl GLDrawableMesh {
    pub fn new(
        gl_mesh: Arc<GLMesh>,
        material: Arc<GLMaterial>,
        transform: Transform<f32, f32, f32>,
        gl_mesh_shader_program: Arc<GLMeshShaderProgram>,
    ) -> Self {
        let vertex_array_object = VertexArrayObject::new(|vao_interface| {
            vao_interface.use_index_buffer_object(&gl_mesh.index_buffer_object);

            if let Some(attribute) = &gl_mesh_shader_program.attributes.position {
                vao_interface.bind_vbo_to_shader_attrib(&gl_mesh.positions_vbo, attribute);
            }

            if let Some(attribute) = &gl_mesh_shader_program.attributes.normal {
                vao_interface.bind_vbo_to_shader_attrib(&gl_mesh.normals_vbo, attribute);
            }

            if let Some(attribute) = &gl_mesh_shader_program.attributes.tangent {
                vao_interface.bind_vbo_to_shader_attrib(&gl_mesh.tangents_vbo, attribute);
            }

            if let Some(attribute) = &gl_mesh_shader_program.attributes.uv_channels {
                for i in 0..gl_mesh.uv_channel_vbos.len() {
                    let uv_channel_vbo = &gl_mesh.uv_channel_vbos[i];
                    vao_interface.bind_vbo_to_shader_attrib_array(uv_channel_vbo, attribute, i);
                }
            }

            if let Some(attribute) = &gl_mesh_shader_program.attributes.bone_ids {
                vao_interface.bind_vbo_to_shader_attrib(&gl_mesh.bone_ids_vbo, attribute);
            }

            if let Some(attribute) = &gl_mesh_shader_program.attributes.bone_weights {
                vao_interface.bind_vbo_to_shader_attrib(&gl_mesh.bone_weights_vbo, attribute);
            }
        });

        Self {
            gl_mesh,
            material,
            object_matrix: transform.into(),
            bone_transforms: None,
            vertex_array_object,
            gl_mesh_shader_program,
        }
    }

    pub fn render(
        &self,
        eye_position: &Vec3<f32>,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
    ) {
        self.gl_mesh_shader_program.shader_program.use_program();

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.eye_position {
            uniform.send_uniform_3fv(eye_position.as_slice(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.object_matrix {
            uniform.send_uniform_matrix_4fv(self.object_matrix.as_col_slice(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.view_matrix {
            uniform.send_uniform_matrix_4fv(view_matrix.as_col_slice(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.projection_matrix {
            uniform.send_uniform_matrix_4fv(projection_matrix.as_col_slice(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.normal_matrix {
            let mut normal_matrix = self.object_matrix.inverted_affine_transform();
            normal_matrix.transpose();
            uniform.send_uniform_matrix_4fv(normal_matrix.as_col_slice(), 1);
        }

        let bone_transforms = self
            .bone_transforms
            .as_ref()
            .unwrap_or(&self.gl_mesh.bone_transforms);
        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.bones {
            uniform
                .send_uniform_matrix_4fv(bone_transforms[0].as_col_slice(), bone_transforms.len());
        }

        let mut texture_layer_counter = 0;

        self.use_texture(
            &mut texture_layer_counter,
            find_texture_with_min_uv_id(&self.material.textures, MaterialTextureType::Albedo),
            self.gl_mesh_shader_program
                .uniforms
                .use_albedo_texture
                .as_ref(),
            self.gl_mesh_shader_program.uniforms.albedo_texture.as_ref(),
            self.gl_mesh_shader_program
                .uniforms
                .albedo_texture_uv_channel_id
                .as_ref(),
        );

        self.use_texture(
            &mut texture_layer_counter,
            find_texture_with_min_uv_id(&self.material.textures, MaterialTextureType::Normal),
            self.gl_mesh_shader_program
                .uniforms
                .use_normal_texture
                .as_ref(),
            self.gl_mesh_shader_program.uniforms.normal_texture.as_ref(),
            self.gl_mesh_shader_program
                .uniforms
                .normal_texture_uv_channel_id
                .as_ref(),
        );

        self.use_texture(
            &mut texture_layer_counter,
            find_texture_with_min_uv_id(&self.material.textures, MaterialTextureType::Displacement),
            self.gl_mesh_shader_program
                .uniforms
                .use_displacement_texture
                .as_ref(),
            self.gl_mesh_shader_program
                .uniforms
                .displacement_texture
                .as_ref(),
            self.gl_mesh_shader_program
                .uniforms
                .displacement_texture_uv_channel_id
                .as_ref(),
        );

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.opacity {
            uniform.send_uniform_1f(self.material.opacity);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.albedo_color {
            uniform.send_uniform_3fv(self.material.albedo_color.as_slice(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.emissive_color {
            uniform.send_uniform_3fv(self.material.emissive_color.as_slice(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.shininess_color {
            uniform.send_uniform_3fv(self.material.shininess_color.as_slice(), 1);
        }

        self.vertex_array_object.use_vao(|| {
            self.gl_mesh.index_buffer_object.draw();
        });
    }

    fn use_texture(
        &self,
        texture_layer_id: &mut usize,
        material_texture: Option<&GLMaterialTexture>,
        use_texture: Option<&ShaderUniform>,
        texture: Option<&ShaderUniform>,
        texture_uv_channel_id: Option<&ShaderUniform>,
    ) {
        if let Some(material_texture) = material_texture {
            material_texture.texture.use_texture(*texture_layer_id);
            material_texture
                .texture
                .set_texture_map_mode(material_texture.texture_map_mode);

            if let Some(use_texture) = use_texture {
                use_texture.send_uniform_1i(1);
            }

            if let Some(texture) = texture {
                texture.send_uniform_1i(*texture_layer_id as i32);
            }

            if let Some(texture_uv_channel_id) = texture_uv_channel_id {
                texture_uv_channel_id.send_uniform_1ui(material_texture.uv_channel_id as u32);
            }

            *texture_layer_id += 1;
        } else if let Some(use_texture) = use_texture {
            use_texture.send_uniform_1i(0);
        }
    }

    pub fn set_transform(&mut self, transform: &Transform<f32, f32, f32>) {
        self.object_matrix = (*transform).into();
    }
}

fn find_texture_with_min_uv_id(
    textures: &[GLMaterialTexture],
    texture_type: MaterialTextureType,
) -> Option<&GLMaterialTexture> {
    textures
        .iter()
        .filter(|texture| texture.texture_type == texture_type)
        .min_by(|item0, item1| item0.uv_channel_id.cmp(&item1.uv_channel_id))
}
