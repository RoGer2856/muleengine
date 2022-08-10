use std::ops::Index;
use std::os::raw::{c_float, c_uint};
use std::str::Utf8Error;
use std::sync::Arc;

use vek::{Mat4, Vec2, Vec3, Vec4};

use super::aabb::AxisAlignedBoundingBox;
use super::assets_reader::{canonicalize_path, parent_path, AssetsReader};

pub enum SceneLoadError {
    CannotCreateTempFile(std::io::Error),
    TempFileCopyError(std::io::Error),
    CannotOpenAsset { path: String },
    AssimpReadError(String),
    Unexpected,
}

pub enum MeshConvertError {
    FaceIndexError {
        face_id: u32,
    },
    MissingBone {
        bone_id: u32,
    },
    MissingVertexWeight {
        bone_id: u32,
        vertex_weight_id: u32,
    },
    MissingTextureCoords {
        channel_id: usize,
        vertex_index: u32,
    },
    MissingVertex {
        vertex_index: u32,
    },
    MissingNormal {
        vertex_index: u32,
    },
    MissingTangent {
        vertex_index: u32,
    },
    MissingBitangent {
        vertex_index: u32,
    },
    Utf8Error(Utf8Error),
}

pub struct VertexBoneWeight {
    pub bone_ids: Vec4<usize>,
    pub weights: Vec4<f32>,
}

pub struct Bone {
    pub name: String,
    pub transform_matrix: Mat4<f32>,
}

#[derive(Debug, Copy, Clone)]
pub enum MaterialTextureType {
    Albedo,
    Shininess,
    Normal,
    Displacement,
    Emission,
}

#[derive(Debug, Copy, Clone)]
pub enum TextureMapMode {
    Repeat,
    Clamp,
    Mirror,
}

#[derive(Clone)]
pub struct MaterialTexture {
    pub path: String,
    pub texture_type: MaterialTextureType,
    pub texture_map_mode: TextureMapMode,
    pub blend: f32,
    pub uv_channel_id: usize,
}

pub struct Material {
    pub textures: Vec<MaterialTexture>,
    pub opacity: f32,
    pub albedo_color: Vec3<f32>,
    pub shininess_color: Vec3<f32>,
    pub emissive_color: Vec3<f32>,
}

impl Clone for Material {
    fn clone(&self) -> Self {
        Self {
            textures: self.textures.clone(),
            opacity: self.opacity,
            albedo_color: self.albedo_color,
            shininess_color: self.shininess_color,
            emissive_color: self.shininess_color,
        }
    }
}

unsafe impl std::marker::Sync for Material {}
unsafe impl std::marker::Send for Material {}

#[repr(C)]
pub struct Mesh {
    faces: Vec<u32>,
    vertex_bone_weights: Vec<VertexBoneWeight>,
    positions: Vec<Vec3<f32>>,
    normals: Vec<Vec3<f32>>,
    tangents: Vec<Vec3<f32>>,
    bitangents: Vec<Vec3<f32>>,
    uv_channels: Vec<Vec<Vec2<f32>>>,
    bones: Vec<Bone>,
    material: Material,
    aabb: AxisAlignedBoundingBox,
}

unsafe impl std::marker::Sync for Mesh {}
unsafe impl std::marker::Send for Mesh {}

pub struct Scene {
    meshes: Vec<Result<Arc<Mesh>, MeshConvertError>>,
}

unsafe impl std::marker::Sync for Scene {}
unsafe impl std::marker::Send for Scene {}

fn ai_string_to_str(ai_string: &assimp_sys::AiString) -> Result<&str, Utf8Error> {
    std::str::from_utf8(&ai_string.data[0..ai_string.length])
}

impl std::convert::From<assimp_sys::AiTextureType> for MaterialTextureType {
    fn from(ai_texture_type: assimp_sys::AiTextureType) -> Self {
        match ai_texture_type {
            assimp_sys::AiTextureType::Diffuse => MaterialTextureType::Albedo,
            assimp_sys::AiTextureType::Normals => MaterialTextureType::Normal,
            assimp_sys::AiTextureType::Displacement => MaterialTextureType::Displacement,
            assimp_sys::AiTextureType::Specular => MaterialTextureType::Shininess,
            _ => MaterialTextureType::Albedo,
        }
    }
}

impl std::convert::From<assimp_sys::AiTextureMapMode> for TextureMapMode {
    fn from(ai_texture_map_mode: assimp_sys::AiTextureMapMode) -> Self {
        match ai_texture_map_mode {
            assimp_sys::AiTextureMapMode::Clamp => TextureMapMode::Clamp,
            assimp_sys::AiTextureMapMode::Mirror => TextureMapMode::Mirror,
            assimp_sys::AiTextureMapMode::Wrap => TextureMapMode::Repeat,
            _ => TextureMapMode::Clamp,
        }
    }
}

impl MaterialTexture {
    pub fn new(
        path: String,
        texture_type: MaterialTextureType,
        texture_map_mode: TextureMapMode,
        blend: f32,
        uv_channel_id: usize,
    ) -> Self {
        Self {
            path,
            texture_type,
            texture_map_mode,
            blend,
            uv_channel_id,
        }
    }

    pub fn from_assimp_material_texture(
        assets_reader: &AssetsReader,
        scene_parent_dir: String,
        ai_material: &assimp_sys::AiMaterial,
        texture_type: assimp_sys::AiTextureType,
        index: usize,
    ) -> Result<Self, Utf8Error> {
        let mut path = assimp_sys::AiString {
            length: 0,
            data: [0; 1024],
        };
        let mapping = assimp_sys::AiTextureMapping::UV;
        let mut uv_index: c_uint = 0;
        let mut blend: c_float = 0.0;
        let mut op = assimp_sys::AiTextureOp::Multiply;
        let mut map_mode = assimp_sys::AiTextureMapMode::Wrap;
        let mut flags: c_uint = 0;
        unsafe {
            assimp_sys::aiGetMaterialTexture(
                ai_material,
                texture_type,
                index as c_uint,
                &mut path,
                &mapping,
                &mut uv_index,
                &mut blend,
                &mut op,
                &mut map_mode,
                &mut flags,
            );
        };

        let path = canonicalize_path(ai_string_to_str(&path)?.to_string());

        let tmp_path = scene_parent_dir + "/" + &path;
        let path = if assets_reader.get_reader(&tmp_path).is_some() {
            tmp_path
        } else {
            path
        };

        Ok(Self {
            path,
            texture_type: MaterialTextureType::from(texture_type),
            texture_map_mode: TextureMapMode::from(map_mode),
            blend: blend,
            uv_channel_id: uv_index as usize,
        })
    }
}

impl Material {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
            opacity: 1.0,
            albedo_color: Vec3::broadcast(0.0),
            shininess_color: Vec3::broadcast(0.0),
            emissive_color: Vec3::broadcast(0.0),
        }
    }

    pub fn add_texture(&mut self, texture: MaterialTexture) {
        self.textures.push(texture);
    }
}

impl Bone {
    pub fn new(name: String, transform_matrix: Mat4<f32>) -> Self {
        Self {
            name,
            transform_matrix,
        }
    }
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            faces: Vec::new(),
            vertex_bone_weights: Vec::new(),
            bones: Vec::new(),
            positions: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            bitangents: Vec::new(),
            uv_channels: Vec::new(),
            material: Material::new(),
            aabb: AxisAlignedBoundingBox::new(Vec3::broadcast(0.0)),
        }
    }

    fn from_assimp_mesh(
        assets_reader: &AssetsReader,
        scene_path: &str,
        ai_mesh: assimp::Mesh,
        materials: &[*mut assimp_sys::AiMaterial],
    ) -> Result<Self, MeshConvertError> {
        let mut mesh = Self::new();

        for i in 0..ai_mesh.num_faces() {
            for j in 0..3 {
                let mut vertex_bone_weight = VertexBoneWeight {
                    bone_ids: Vec4::new(0, 0, 0, 0),
                    weights: Vec4::new(0.0, 0.0, 0.0, 0.0),
                };

                let mut bone_weights = Vec::new();

                let vertex_index = *ai_mesh
                    .get_face(i)
                    .ok_or(MeshConvertError::FaceIndexError { face_id: i })?
                    .index(j);
                if ai_mesh.num_bones() != 0 {
                    for bone_id in 0..ai_mesh.num_bones() {
                        let bone = ai_mesh
                            .get_bone(bone_id)
                            .ok_or(MeshConvertError::MissingBone { bone_id })?;
                        for l in 0..bone.num_weights() {
                            let weight = bone.get_weight(l).ok_or(
                                MeshConvertError::MissingVertexWeight {
                                    vertex_weight_id: l,
                                    bone_id,
                                },
                            )?;
                            if weight.vertex_id == vertex_index {
                                bone_weights.push((bone_id, weight.weight));
                                break;
                            }
                        }
                    }
                } else {
                    vertex_bone_weight.bone_ids.x = 0;
                    vertex_bone_weight.weights.x = 1.0;
                }

                bone_weights
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                let mut bone_weight_counter = 0;
                for bone_weight in bone_weights {
                    match bone_weight_counter {
                        0 => {
                            vertex_bone_weight.bone_ids.x = bone_weight.0 as usize;
                            vertex_bone_weight.weights.x = bone_weight.1;
                        }
                        1 => {
                            vertex_bone_weight.bone_ids.y = bone_weight.0 as usize;
                            vertex_bone_weight.weights.y = bone_weight.1;
                        }
                        2 => {
                            vertex_bone_weight.bone_ids.z = bone_weight.0 as usize;
                            vertex_bone_weight.weights.z = bone_weight.1;
                        }
                        3 => {
                            vertex_bone_weight.bone_ids.w = bone_weight.0 as usize;
                            vertex_bone_weight.weights.w = bone_weight.1;
                        }
                        _ => (),
                    }
                    bone_weight_counter += 1;
                }

                let mut vertex_uv_channels = Vec::new();
                for channel_id in 0..ai_mesh.get_num_uv_channels() {
                    let texture_coords = ai_mesh
                        .get_texture_coord(channel_id, vertex_index)
                        .ok_or(MeshConvertError::MissingTextureCoords {
                            channel_id,
                            vertex_index,
                        })?;
                    vertex_uv_channels.push(Vec2 {
                        x: texture_coords.x,
                        y: texture_coords.y,
                    });
                }

                // add vertex to the mesh
                let position = ai_mesh
                    .get_vertex(vertex_index)
                    .ok_or(MeshConvertError::MissingVertex { vertex_index })?;
                let normal = ai_mesh
                    .get_normal(vertex_index)
                    .ok_or(MeshConvertError::MissingNormal { vertex_index })?;

                let position = Vec3 {
                    x: position.x,
                    y: position.y,
                    z: position.z,
                };

                let normal = Vec3 {
                    x: normal.x,
                    y: normal.y,
                    z: normal.z,
                };

                let (tangent, bitangent) = if ai_mesh.has_tangents_and_bitangents() {
                    let tangent = ai_mesh
                        .get_tangent(vertex_index)
                        .ok_or(MeshConvertError::MissingTangent { vertex_index })?;
                    let bitangent = ai_mesh
                        .get_bitangent(vertex_index)
                        .ok_or(MeshConvertError::MissingBitangent { vertex_index })?;

                    (
                        Some(Vec3 {
                            x: tangent.x,
                            y: tangent.y,
                            z: tangent.z,
                        }),
                        Some(Vec3 {
                            x: bitangent.x,
                            y: bitangent.y,
                            z: bitangent.z,
                        }),
                    )
                } else {
                    (None, None)
                };

                mesh.add_vertex(
                    position,
                    normal,
                    tangent,
                    bitangent,
                    vertex_uv_channels,
                    vertex_bone_weight,
                );
            }

            // add face to mesh
            mesh.add_face(
                (mesh.number_of_vertices() - 3) as u32,
                (mesh.number_of_vertices() - 2) as u32,
                (mesh.number_of_vertices() - 1) as u32,
            );
        }

        if ai_mesh.num_bones() != 0 {
            for bone in ai_mesh.bone_iter() {
                mesh.add_bone(Bone::new(
                    bone.name().to_string(),
                    assimp_matrix4x4_to_vek_mat4(&bone.offset_matrix()),
                ));
            }
        } else {
            mesh.add_bone(Bone::new("root".to_string(), Mat4::identity()));
        }

        // material
        let mut material = Material::new();

        let ai_material = materials[ai_mesh.material_index as usize];
        let ai_material = unsafe { &*ai_material };

        let scene_path = scene_path.to_string();

        if unsafe {
            assimp_sys::aiGetMaterialTextureCount(ai_material, assimp_sys::AiTextureType::Diffuse)
        } != 0
        {
            let material_texture = MaterialTexture::from_assimp_material_texture(
                assets_reader,
                parent_path(scene_path.clone()),
                &ai_material,
                assimp_sys::AiTextureType::Diffuse,
                0,
            )
            .map_err(|e| MeshConvertError::Utf8Error(e))?;

            material.add_texture(material_texture);
        }

        if unsafe {
            assimp_sys::aiGetMaterialTextureCount(ai_material, assimp_sys::AiTextureType::Normals)
        } != 0
        {
            let material_texture = MaterialTexture::from_assimp_material_texture(
                assets_reader,
                parent_path(scene_path.clone()),
                &ai_material,
                assimp_sys::AiTextureType::Normals,
                0,
            )
            .map_err(|e| MeshConvertError::Utf8Error(e))?;

            material.add_texture(material_texture);
        }

        if unsafe {
            assimp_sys::aiGetMaterialTextureCount(
                ai_material,
                assimp_sys::AiTextureType::Displacement,
            )
        } != 0
        {
            let material_texture = MaterialTexture::from_assimp_material_texture(
                assets_reader,
                parent_path(scene_path.clone()),
                &ai_material,
                assimp_sys::AiTextureType::Displacement,
                0,
            )
            .map_err(|e| MeshConvertError::Utf8Error(e))?;

            material.add_texture(material_texture);
        }

        if unsafe {
            assimp_sys::aiGetMaterialTextureCount(ai_material, assimp_sys::AiTextureType::Height)
        } != 0
        {
            let material_texture = MaterialTexture::from_assimp_material_texture(
                assets_reader,
                parent_path(scene_path.clone()),
                &ai_material,
                assimp_sys::AiTextureType::Height,
                0,
            )
            .map_err(|e| MeshConvertError::Utf8Error(e))?;

            material.add_texture(material_texture);
        }

        if unsafe {
            assimp_sys::aiGetMaterialTextureCount(ai_material, assimp_sys::AiTextureType::Specular)
        } != 0
        {
            let material_texture = MaterialTexture::from_assimp_material_texture(
                assets_reader,
                parent_path(scene_path.clone()),
                &ai_material,
                assimp_sys::AiTextureType::Specular,
                0,
            )
            .map_err(|e| MeshConvertError::Utf8Error(e))?;

            material.add_texture(material_texture);
        }

        let material_properties: &[*mut assimp_sys::AiMaterialProperty] = unsafe {
            std::slice::from_raw_parts(ai_material.properties, ai_material.num_properties as usize)
        };

        for property in material_properties {
            let property = unsafe { &**property };

            let key =
                ai_string_to_str(&property.key).map_err(|e| MeshConvertError::Utf8Error(e))?;
            if key == "$clr.diffuse" {
                if property.data_length == 12 {
                    let color = unsafe { *(property.data as *const assimp_sys::AiColor3D) };
                    material.albedo_color = Vec3::new(color.r, color.g, color.b);
                }
            } else if key == "$clr.emissive" {
                if property.data_length == 12 {
                    let color = unsafe { *(property.data as *const assimp_sys::AiColor3D) };
                    material.emissive_color = Vec3::new(color.r, color.g, color.b);
                }
            } else if key == "$clr.specular" {
                if property.data_length == 12 {
                    let color = unsafe { *(property.data as *const assimp_sys::AiColor3D) };
                    material.shininess_color = Vec3::new(color.r, color.g, color.b);
                }
            } else if key == "$mat.opacity" {
                if property.data_length == 4 {
                    material.opacity = unsafe { *(property.data as *const f32) };
                }
            }
        }

        mesh.material = material;

        // mesh.compute_tangents(0);

        Ok(mesh)
    }

    pub fn add_vertex(
        &mut self,
        position: Vec3<f32>,
        normal: Vec3<f32>,
        tangent: Option<Vec3<f32>>,
        bitangent: Option<Vec3<f32>>,
        uv_channels: Vec<Vec2<f32>>,
        vertex_bone_weight: VertexBoneWeight,
    ) {
        let number_of_vertices = self.number_of_vertices() as usize;

        self.vertex_bone_weights.push(vertex_bone_weight);

        self.positions.push(position);
        self.normals.push(normal);
        if let Some(tangent) = tangent {
            self.tangents.push(tangent);
        }
        if let Some(bitangent) = bitangent {
            self.bitangents.push(bitangent);
        }

        self.uv_channels.resize_with(uv_channels.len(), || {
            let mut v = Vec::new();
            v.resize(number_of_vertices, Vec2::broadcast(0.0));
            v
        });

        for i in 0..uv_channels.len() {
            self.uv_channels.as_mut_slice()[i].push(uv_channels.as_slice()[i]);
        }
    }

    pub fn add_bone(&mut self, bone: Bone) {
        self.bones.push(bone);
    }

    pub fn add_face(&mut self, vertex_index0: u32, vertex_index1: u32, vertex_index2: u32) {
        if self.faces.is_empty() {
            self.aabb =
                AxisAlignedBoundingBox::new(self.positions.as_slice()[vertex_index0 as usize]);
        } else {
            self.aabb
                .add_vertex(self.positions.as_slice()[vertex_index0 as usize]);
        }

        self.aabb
            .add_vertex(self.positions.as_slice()[vertex_index1 as usize]);
        self.aabb
            .add_vertex(self.positions.as_slice()[vertex_index2 as usize]);

        self.faces.push(vertex_index0);
        self.faces.push(vertex_index1);
        self.faces.push(vertex_index2);
    }

    pub fn number_of_vertices(&self) -> usize {
        self.positions.len()
    }

    pub fn material_ref(&self) -> &Material {
        &self.material
    }

    pub fn compute_tangents(&mut self, uv_channel_id: usize) {
        if uv_channel_id < self.uv_channels.len() as usize {
            let faces = self.faces.as_slice();
            let uv_channel = self.uv_channels.as_slice()[uv_channel_id].as_slice();
            let positions = self.positions.as_slice();

            self.tangents.clear();
            self.bitangents.clear();

            self.tangents = (0..positions.len())
                .map(|_| Vec3::new(1.0, 0.0, 0.0))
                .collect();
            self.bitangents = (0..positions.len())
                .map(|_| Vec3::new(0.0, 1.0, 0.0))
                .collect();

            let mut tangent_vectors: Vec<Vec<vek::Vec3<f32>>> =
                (0..positions.len()).map(|_| Vec::new()).collect();
            let mut bitangent_vectors: Vec<Vec<vek::Vec3<f32>>> =
                (0..positions.len()).map(|_| Vec::new()).collect();

            for i in (0..faces.len() as usize).step_by(3) {
                let index_a = faces[i + 0] as usize;
                let index_b = faces[i + 1] as usize;
                let index_c = faces[i + 2] as usize;

                let a2b_tex = uv_channel[index_b] - uv_channel[index_a];
                let a2c_tex = uv_channel[index_c] - uv_channel[index_a];

                let a2b = positions[index_b] - positions[index_a];
                let a2c = positions[index_c] - positions[index_a];

                let m = vek::mat::repr_c::column_major::Mat2::new(
                    a2c_tex.x, a2b_tex.x, a2c_tex.y, a2b_tex.y,
                );
                let determinant_reciprocal: f32 = 1.0 / m.determinant();
                let m = vek::mat::repr_c::column_major::Mat2::new(
                    determinant_reciprocal * m.cols[1][1],
                    -determinant_reciprocal * m.cols[1][0],
                    -determinant_reciprocal * m.cols[0][1],
                    determinant_reciprocal * m.cols[0][0],
                );

                let tmp0: vek::Vec2<f32> = m * vek::Vec2::new(a2b.x, a2c.x);
                let tmp1: vek::Vec2<f32> = m * vek::Vec2::new(a2b.y, a2c.y);
                let tmp2: vek::Vec2<f32> = m * vek::Vec2::new(a2b.z, a2c.z);

                // let tangent = vek::Vec3::new(tmp0.x, tmp1.x, tmp2.x);
                let bitangent = vek::Vec3::new(tmp0.y, tmp1.y, tmp2.y);

                // tangent_vectors[index_a].push(tangent);
                // tangent_vectors[index_b].push(tangent);
                // tangent_vectors[index_c].push(tangent);

                // TODO(@RoGer)
                tangent_vectors[index_a].push(bitangent);
                tangent_vectors[index_b].push(bitangent);
                tangent_vectors[index_c].push(bitangent);

                bitangent_vectors[index_a].push(bitangent);
                bitangent_vectors[index_b].push(bitangent);
                bitangent_vectors[index_c].push(bitangent);
            }

            for vertex_index in 0..tangent_vectors.len() {
                let mut sum = vek::Vec3::broadcast(0.0);
                for v in tangent_vectors[vertex_index].iter() {
                    sum += *v;
                }
                sum.normalize();

                self.tangents[vertex_index] = sum;
            }

            for vertex_index in 0..bitangent_vectors.len() {
                let mut sum = vek::Vec3::broadcast(0.0);
                for v in bitangent_vectors[vertex_index].iter() {
                    sum += *v;
                }
                sum.normalize();

                self.bitangents[vertex_index] = sum;
            }
        }
    }
}

impl Scene {
    fn from_assimp_scene(
        assets_reader: &AssetsReader,
        scene_path: &str,
        scene: assimp::Scene,
    ) -> Scene {
        let mut meshes = Vec::new();

        for index in 0..scene.num_meshes() as usize {
            if let Some(mesh) = scene.mesh(index) {
                let materials = unsafe {
                    std::slice::from_raw_parts(scene.materials, scene.num_materials as usize)
                };
                meshes.push(
                    Mesh::from_assimp_mesh(assets_reader, scene_path, mesh, materials)
                        .map(|mesh| Arc::new(mesh)),
                );
            }
        }

        Scene { meshes }
    }

    pub fn load(assets_reader: &AssetsReader, scene_path: &str) -> Result<Scene, SceneLoadError> {
        let mut reader =
            assets_reader
                .get_reader(scene_path)
                .ok_or(SceneLoadError::CannotOpenAsset {
                    path: scene_path.to_string(),
                })?;

        let mut tmp_file =
            tempfile::NamedTempFile::new().map_err(|e| SceneLoadError::CannotCreateTempFile(e))?;

        std::io::copy(&mut reader, tmp_file.as_file_mut())
            .map_err(|e| SceneLoadError::TempFileCopyError(e))?;

        let mut importer = assimp::Importer::new();
        importer.triangulate(true);
        importer.generate_normals(|generate_normals_mut| {
            generate_normals_mut.enable = true;
            generate_normals_mut.smooth = false;
            generate_normals_mut.max_smoothing_angle = 0.0;
        });
        importer.calc_tangent_space(|calc_tanget_space| {
            calc_tanget_space.enable = true;
        });
        importer.transform_uv_coords(|transform_uv_coords| {
            transform_uv_coords.enable = true;
        });
        importer.gen_uv_coords(true);
        importer.flip_uvs(true);
        importer.gen_uv_coords(true);
        importer.find_instances(true);
        importer.join_identical_vertices(true);
        importer.limit_bone_weights(|limit_bone_weights| {
            limit_bone_weights.enable = true;
            limit_bone_weights.max_weights = 4;
        });
        importer.validate_data_structure(true);
        importer.fbx_read_textures(false);

        let scene = importer
            .read_file(&tmp_file.path().to_str().ok_or(SceneLoadError::Unexpected)?)
            .map_err(|e| SceneLoadError::AssimpReadError(e.to_string()))?;

        Ok(Self::from_assimp_scene(assets_reader, scene_path, scene))
    }

    pub fn meshes_ref(&self) -> &Vec<Result<Arc<Mesh>, MeshConvertError>> {
        &self.meshes
    }
}

fn assimp_matrix4x4_to_vek_mat4(m: &assimp::math::matrix4::Matrix4x4) -> Mat4<f32> {
    Mat4::new(
        m.a1, m.b1, m.c1, m.d1, m.a2, m.b2, m.c2, m.d2, m.a3, m.b3, m.c3, m.d3, m.a4, m.b4, m.c4,
        m.d4,
    )
}
