use std::{
    io::{BufReader, Read},
    sync::Arc,
};

use tobj::{load_mtl_buf, LoadError, LoadOptions, MTLLoadResult, Material as TobjMaterial};
use vek::{Mat4, Vec2, Vec3, Vec4};

use crate::{
    asset_reader::AssetReader,
    image_container::ImageContainer,
    mesh::{Bone, Material as MeMaterial, Mesh, Scene, VertexBoneWeight},
};

#[derive(Debug)]
pub enum MaterialConversionError {}

#[derive(Debug)]
pub enum ObjLoadError {
    TobjLoadError(LoadError),
    TobjMtlLoadError(LoadError),
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
    material: &TobjMaterial,
) -> Result<MeMaterial, MaterialConversionError> {
    todo!();
}

pub fn load(
    reader: impl Read,
    asset_reader: &AssetReader,
    image_container: &mut ImageContainer,
) -> Result<Scene, ObjLoadError> {
    let (models, materials_result) =
        tobj::load_obj_buf(&mut BufReader::new(reader), &LOAD_OPTIONS, |path| {
            let path = path.to_str().ok_or(LoadError::GenericFailure)?;
            let reader = asset_reader
                .get_reader(path)
                .ok_or(LoadError::GenericFailure)?;
            load_mtl_buf(&mut BufReader::new(reader))
        })
        .map_err(ObjLoadError::TobjLoadError)?;

    let materials = materials_result.map_err(ObjLoadError::TobjMtlLoadError)?;
    let materials = materials
        .into_iter()
        .enumerate()
        .map(|(material_index, material)| {
            convert_tobj_material_to_me_material(&material).map_err(|e| {
                ObjLoadError::MaterialConversionError {
                    material_index,
                    error: e,
                }
            })
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

        mesh.compute_tangents(0);

        // todo!("if the mesh does not contain normal coordinates, then it should be recalculated")

        scene.add_mesh(Arc::new(mesh));
    }

    Ok(scene)
}
