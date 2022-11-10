use std::sync::Arc;

use vek::{Mat4, Vec3, Vec4};

use crate::muleengine::{
    drawable_object::DrawableObject,
    mesh::{Bone, MaterialTextureType, Mesh},
};

use super::{
    gl_material::{GLMaterial, GLMaterialTexture},
    gl_mesh_shader_program::GLMeshShaderProgram,
    gl_texture_container::GLTextureContainer,
    opengl_utils::{
        index_buffer_object::{IndexBufferObject, PrimitiveMode},
        shader_input::ShaderUniform,
        vertex_array_object::VertexArrayObject,
        vertex_buffer_object::{DataCount, DataType, VertexBufferObject},
    },
};

pub struct GLMesh {
    mesh: Arc<Mesh>,

    material: Arc<GLMaterial>,
    bone_transforms: Vec<Mat4<f32>>,

    index_buffer_object: IndexBufferObject,
    positions_vbo: VertexBufferObject,
    normals_vbo: VertexBufferObject,
    tangents_vbo: VertexBufferObject,
    uv_channel_vbos: Vec<VertexBufferObject>,
    bone_ids_vbo: VertexBufferObject,
    bone_weights_vbo: VertexBufferObject,

    _bone_weights_vector: Vec<Vec4<f32>>,
    _bone_ids_vector: Vec<Vec4<u32>>,
}

pub struct GLDrawableMesh {
    gl_mesh: Arc<GLMesh>,
    pub material: Option<GLMaterial>,
    bone_transforms: Option<Vec<Mat4<f32>>>,
    vertex_array_object: VertexArrayObject,
    gl_mesh_shader_program: Arc<GLMeshShaderProgram>,
}

impl DrawableObject for GLDrawableMesh {
    fn render(
        &self,
        eye_position: &Vec3<f32>,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
        object_matrix: &Mat4<f32>,
    ) {
        GLDrawableMesh::render(
            &self,
            eye_position,
            projection_matrix,
            view_matrix,
            object_matrix,
        );
    }
}

impl GLMesh {
    pub fn new(mesh: Arc<Mesh>, gl_texture_container: &mut GLTextureContainer) -> Self {
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
        let normals_vbo = VertexBufferObject::new(
            mesh.get_normals().as_ptr(),
            mesh.get_normals().len(),
            DataType::F32,
            DataCount::Coords3,
        );
        let tangents_vbo = VertexBufferObject::new(
            mesh.get_tangents().as_ptr(),
            mesh.get_tangents().len(),
            DataType::F32,
            DataCount::Coords3,
        );

        let mut uv_channel_vbos = Vec::new();
        for uv_channel in mesh.get_uv_channels() {
            uv_channel_vbos.push(VertexBufferObject::new(
                uv_channel.as_ptr(),
                uv_channel.len(),
                DataType::F32,
                DataCount::Coords2,
            ));
        }

        let mut bone_weights_vector = Vec::new();
        let mut bone_ids_vector = Vec::new();

        for bone_weight in mesh.get_vertex_bone_weights() {
            let bone_weights = Vec4::new(
                bone_weight.weights.x,
                bone_weight.weights.y,
                bone_weight.weights.z,
                bone_weight.weights.w,
            );

            let bone_ids = Vec4::new(
                bone_weight.bone_ids.x as u32,
                bone_weight.bone_ids.y as u32,
                bone_weight.bone_ids.z as u32,
                bone_weight.bone_ids.w as u32,
            );

            bone_weights_vector.push(bone_weights);
            bone_ids_vector.push(bone_ids);
        }

        let bone_ids_vbo = VertexBufferObject::new(
            bone_ids_vector.as_ptr(),
            bone_ids_vector.len(),
            DataType::U32,
            DataCount::Coords4,
        );
        let bone_weights_vbo = VertexBufferObject::new(
            bone_weights_vector.as_ptr(),
            bone_weights_vector.len(),
            DataType::F32,
            DataCount::Coords4,
        );

        let material = GLMaterial::new(mesh.get_material(), gl_texture_container);
        let bone_transforms = mesh
            .get_bones()
            .iter()
            .map(|bone| bone.transform_matrix)
            .collect();

        Self {
            mesh,
            material: Arc::new(material),
            bone_transforms,

            index_buffer_object,

            positions_vbo,
            normals_vbo,
            tangents_vbo,
            uv_channel_vbos,
            bone_ids_vbo,
            bone_weights_vbo,

            _bone_weights_vector: bone_weights_vector,
            _bone_ids_vector: bone_ids_vector,
        }
    }

    pub fn get_bones(&self) -> &Vec<Bone> {
        self.mesh.get_bones()
    }
}

impl GLDrawableMesh {
    pub fn new(gl_mesh: Arc<GLMesh>, gl_mesh_shader_program: Arc<GLMeshShaderProgram>) -> Self {
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
                    vao_interface.bind_vbo_to_shader_attrib_array(&uv_channel_vbo, attribute, i);
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
            material: None,
            bone_transforms: None,
            vertex_array_object,
            gl_mesh_shader_program,
        }
    }

    pub fn parent_material_ref(&self) -> &GLMaterial {
        &self.gl_mesh.material
    }

    pub fn render(
        &self,
        eye_position: &Vec3<f32>,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
        object_matrix: &Mat4<f32>,
    ) {
        self.gl_mesh_shader_program.shader_program.use_program();

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.eye_position {
            uniform.send_uniform_3fv(eye_position.as_ptr(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.object_matrix {
            uniform.send_uniform_matrix_4fv(object_matrix.as_col_ptr(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.view_matrix {
            uniform.send_uniform_matrix_4fv(view_matrix.as_col_ptr(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.projection_matrix {
            uniform.send_uniform_matrix_4fv(projection_matrix.as_col_ptr(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.normal_matrix {
            let mut normal_matrix = object_matrix.inverted_affine_transform();
            normal_matrix.transpose();
            uniform.send_uniform_matrix_4fv(normal_matrix.as_col_ptr(), 1);
        }

        let bone_transforms = self
            .bone_transforms
            .as_ref()
            .unwrap_or(&self.gl_mesh.bone_transforms);
        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.bones {
            uniform.send_uniform_matrix_4fv(bone_transforms[0].as_col_ptr(), bone_transforms.len());
        }

        let material = self.material.as_ref().unwrap_or(&self.gl_mesh.material);

        let mut texture_layer_counter = 0;

        self.use_texture(
            &mut texture_layer_counter,
            find_texture_with_min_uv_id(&material.textures, MaterialTextureType::Albedo),
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
            find_texture_with_min_uv_id(&material.textures, MaterialTextureType::Normal),
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
            find_texture_with_min_uv_id(&material.textures, MaterialTextureType::Displacement),
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
            uniform.send_uniform_1f(material.opacity);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.albedo_color {
            uniform.send_uniform_3fv(material.albedo_color.as_ptr(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.emissive_color {
            uniform.send_uniform_3fv(material.emissive_color.as_ptr(), 1);
        }

        if let Some(uniform) = &self.gl_mesh_shader_program.uniforms.shininess_color {
            uniform.send_uniform_3fv(material.shininess_color.as_ptr(), 1);
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
}

fn find_texture_with_min_uv_id(
    textures: &Vec<GLMaterialTexture>,
    texture_type: MaterialTextureType,
) -> Option<&GLMaterialTexture> {
    textures
        .iter()
        .filter(|texture| texture.texture_type == texture_type)
        .min_by(|item0, item1| item0.uv_channel_id.cmp(&item1.uv_channel_id))
}
