use std::sync::Arc;

use vek::{Mat4, Vec4};

use muleengine::mesh::Mesh;

use super::{
    gl_material::GLMaterial,
    gl_texture_container::GLTextureContainer,
    opengl_utils::{
        index_buffer_object::{IndexBufferObject, PrimitiveMode},
        vertex_buffer_object::{DataCount, DataType, VertexBufferObject},
    },
};

pub struct GLMesh {
    _mesh: Arc<Mesh>,

    pub(super) material: Arc<GLMaterial>,
    pub(super) bone_transforms: Vec<Mat4<f32>>,

    pub(super) index_buffer_object: IndexBufferObject,
    pub(super) positions_vbo: VertexBufferObject,
    pub(super) normals_vbo: VertexBufferObject,
    pub(super) tangents_vbo: VertexBufferObject,
    pub(super) uv_channel_vbos: Vec<VertexBufferObject>,
    pub(super) bone_ids_vbo: VertexBufferObject,
    pub(super) bone_weights_vbo: VertexBufferObject,

    _bone_weights_vector: Vec<Vec4<f32>>,
    _bone_ids_vector: Vec<Vec4<u32>>,
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
            _mesh: mesh,
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
}
