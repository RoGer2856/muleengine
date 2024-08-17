use std::{
    io::{BufReader, Read},
    path::PathBuf,
    sync::Arc,
};

use bytifex_utils::sync::types::rc_mutex_new;
use tobj::{load_mtl_buf, LoadError, LoadOptions, Material as TobjMaterial};
use vek::{Mat4, Vec2, Vec3, Vec4};

use crate::{
    asset_reader::AssetReader,
    image_container::{ImageContainer, ImageContainerError},
    mesh::{
        Bone, Material as MeMaterial, MaterialTexture, MaterialTextureType, Mesh, Scene,
        TextureMapMode, VertexBoneWeight,
    },
};

#[derive(Debug)]
pub enum MaterialConversionError {
    ImageContainerError(ImageContainerError),
}

impl From<ImageContainerError> for MaterialConversionError {
    fn from(value: ImageContainerError) -> Self {
        MaterialConversionError::ImageContainerError(value)
    }
}

#[derive(Debug)]
pub enum ObjLoadError {
    TobjLoadError(LoadError),
    TobjMtlLoadError(LoadError),
    CouldNotOpenMtlFile(PathBuf),
    IncorrectNumberOfPositionsInModel(usize),
    IncorrectNumberOfVerticesInFace(usize),
    MaterialConversionError {
        material_index: usize,
        error: MaterialConversionError,
    },
}

const LOAD_OPTIONS: LoadOptions = LoadOptions {
    single_index: true,
    triangulate: true,
    ignore_points: true,
    ignore_lines: true,
};

fn convert_tobj_material_to_me_material(
    tobj_material: &TobjMaterial,
    asset_reader: &AssetReader,
    image_container: &mut ImageContainer,
) -> Result<MeMaterial, MaterialConversionError> {
    let mut me_material = MeMaterial::new();

    if let Some(diffuse) = tobj_material.diffuse {
        me_material.albedo_color = Vec3::new(diffuse[0], diffuse[1], diffuse[1]);
    }

    if let Some(ambient) = tobj_material.ambient {
        me_material.emissive_color = Vec3::new(ambient[0], ambient[1], ambient[2]);
    }

    if let Some(specular) = tobj_material.specular {
        me_material.shininess_color = Vec3::new(specular[0], specular[1], specular[2]);
    }

    if let Some(texture_path) = &tobj_material.diffuse_texture {
        me_material.add_texture(MaterialTexture::new(
            image_container.get_image(texture_path, asset_reader)?,
            MaterialTextureType::Albedo,
            TextureMapMode::Repeat,
            1.0,
            0,
        ));
    }

    if let Some(texture_path) = &tobj_material.ambient_texture {
        me_material.add_texture(MaterialTexture::new(
            image_container.get_image(texture_path, asset_reader)?,
            MaterialTextureType::Emission,
            TextureMapMode::Repeat,
            1.0,
            0,
        ));
    }

    if let Some(texture_path) = &tobj_material.specular_texture {
        me_material.add_texture(MaterialTexture::new(
            image_container.get_image(texture_path, asset_reader)?,
            MaterialTextureType::Shininess,
            TextureMapMode::Repeat,
            1.0,
            0,
        ));
    }

    if let Some(texture_path) = &tobj_material.normal_texture {
        me_material.add_texture(MaterialTexture::new(
            image_container.get_image(texture_path, asset_reader)?,
            MaterialTextureType::Normal,
            TextureMapMode::Repeat,
            1.0,
            0,
        ));
    }

    if let Some(texture_path) = &tobj_material.specular_texture {
        me_material.add_texture(MaterialTexture::new(
            image_container.get_image(texture_path, asset_reader)?,
            MaterialTextureType::Shininess,
            TextureMapMode::Repeat,
            1.0,
            0,
        ));
    }

    Ok(me_material)
}

pub fn load(
    reader: impl Read,
    asset_reader: &AssetReader,
    image_container: &mut ImageContainer,
) -> Result<Scene, ObjLoadError> {
    let open_file_failed = rc_mutex_new(None);
    let obj_load_result = tobj::load_obj_buf(&mut BufReader::new(reader), &LOAD_OPTIONS, |path| {
        let path_str = path.to_string_lossy();
        let reader = asset_reader.get_reader(path_str);
        if let Some(reader) = reader {
            load_mtl_buf(&mut BufReader::new(reader))
        } else {
            *open_file_failed.lock() = Some(path.to_path_buf());
            Err(LoadError::OpenFileFailed)
        }
    });

    if let Some(path) = open_file_failed.lock().take() {
        return Err(ObjLoadError::CouldNotOpenMtlFile(path));
    }

    let (models, materials_result) = obj_load_result.map_err(ObjLoadError::TobjLoadError)?;

    let materials = materials_result.map_err(ObjLoadError::TobjMtlLoadError)?;
    let materials = materials
        .into_iter()
        .enumerate()
        .map(|(material_index, material)| {
            convert_tobj_material_to_me_material(&material, asset_reader, image_container).map_err(
                |e| ObjLoadError::MaterialConversionError {
                    material_index,
                    error: e,
                },
            )
        })
        .collect::<Result<Vec<MeMaterial>, ObjLoadError>>()?;

    let mut scene = Scene::new();

    for model in models.iter() {
        let material = model
            .mesh
            .material_id
            .and_then(|index| materials.get(index));
        let mut mesh = material
            .map(|material| Mesh::with_material(material.clone()))
            .unwrap_or_else(Mesh::new);

        mesh.add_bone(Bone::new("root".to_string(), Mat4::identity()));

        let mut recalculate_normals = true;
        for i in 0..model.mesh.positions.len() / 3 {
            let position = if i * 3 + 2 < model.mesh.positions.len() {
                Vec3::new(
                    model.mesh.positions[i * 3],
                    model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                )
            } else {
                return Err(ObjLoadError::IncorrectNumberOfPositionsInModel(
                    model.mesh.positions.len(),
                ));
            };

            let normal = if i * 3 + 2 < model.mesh.normals.len() {
                Vec3::new(
                    model.mesh.normals[i * 3],
                    model.mesh.normals[i * 3 + 1],
                    model.mesh.normals[i * 3 + 2],
                )
            } else {
                recalculate_normals = true;
                Vec3::new(0.0, 0.0, 1.0)
            };

            let texcoords = if i * 2 + 1 < model.mesh.texcoords.len() {
                Vec2::new(model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1])
            } else {
                Vec2::new(0.0, 0.0)
            };

            mesh.add_vertex(
                position,
                normal,
                None,
                None,
                vec![Vec2::new(texcoords[0], texcoords[1])],
                VertexBoneWeight {
                    bone_ids: Vec4::new(0, 0, 0, 0),
                    weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
                },
            );
        }

        for face in model.mesh.indices.as_slice().chunks(3) {
            if face.len() == 3 {
                mesh.add_face(face[0], face[1], face[2]);
            } else {
                return Err(ObjLoadError::IncorrectNumberOfVerticesInFace(face.len()));
            }
        }

        if recalculate_normals {
            mesh.compute_normals();
        }

        mesh.compute_tangents(0);

        scene.add_mesh(Arc::new(mesh));
    }

    Ok(scene)
}
