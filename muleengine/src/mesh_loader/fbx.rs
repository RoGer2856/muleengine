use std::io::{self, Cursor, Read};

use fbxcel_dom::{
    any::AnyDocument,
    v7400::{
        data::mesh::layer::{LayerElementNormalHandle, TypedLayerElementHandle},
        object::{geometry::MeshHandle, model::TypedModelHandle, TypedObjectHandle},
    },
};

use crate::{asset_reader::AssetReader, image_container::ImageContainer, mesh::Scene};

#[derive(Debug)]
pub enum FbxLoadError {
    AssetReadError(io::Error),
    UnsupportedVersion,
}

pub fn load(
    reader: impl Read,
    asset_reader: &AssetReader,
    image_container: &mut ImageContainer,
) -> Result<Scene, FbxLoadError> {
    let reader = std::io::BufReader::new(reader);

    match AnyDocument::from_reader(reader).map_err(|e| todo!())? {
        AnyDocument::V7400(fbx_ver, doc) => {
            for object in doc.objects() {
                match object.get_typed() {
                    TypedObjectHandle::Model(model) => {
                        match model {
                            TypedModelHandle::Mesh(mesh) => {
                                // let geometry = mesh.geometry().unwrap();
                                // let polygon_vertices = geometry.polygon_vertices().unwrap();
                                // let vertices_slice = polygon_vertices.raw_polygon_vertices();
                                // println!("Vertices: {:?}", vertices_slice);

                                // // Iterate over the properties of the geometry
                                // for property in geometry.properties_by_native_typename() {
                                //     // Check if the property is of type LayerElementNormal
                                //     if let Some(normals) = property.get_typed::<LayerElementNormalHandle>() {
                                //         // Get the mapping mode and reference mode of the normals
                                //         let mapping_mode = normals.mapping_mode();
                                //         let reference_mode = normals.reference_mode();

                                //         // Get the raw normal data
                                //         let raw_normals = normals.direct().unwrap();

                                //         println!("Normals: {:?}", raw_normals);
                                //         println!("Mapping Mode: {:?}", mapping_mode);
                                //         println!("Reference Mode: {:?}", reference_mode);
                                //     }
                                // }

                                // // let polygon_vertices = mesh.geometry().unwrap().polygon_vertices().unwrap();
                                // // let vertices_slice = polygon_vertices.raw_polygon_vertices();
                                // // for layer in mesh.geometry().unwrap().layers() {
                                // //     // layer.
                                // // }
                                // let polygon_vertices = mesh.geometry().unwrap().polygon_vertices().unwrap();
                                // let vertices_slice = polygon_vertices.raw_polygon_vertices();
                                // println!("Vertices: {:?}", vertices_slice);
                            }
                            TypedModelHandle::Camera(_) => {}
                            TypedModelHandle::Light(_) => {}
                            TypedModelHandle::LimbNode(_) => {}
                            TypedModelHandle::Null(_) => {}
                            TypedModelHandle::Unknown(_) => {}
                            _ => {}
                        }
                    }
                    TypedObjectHandle::Texture(texture) => {}
                    TypedObjectHandle::Material(material) => {}
                    TypedObjectHandle::Deformer(deformer) => {}
                    TypedObjectHandle::Geometry(_) => {}
                    TypedObjectHandle::NodeAttribute(_) => {}
                    TypedObjectHandle::SubDeformer(_) => {}
                    TypedObjectHandle::Video(_) => {}
                    TypedObjectHandle::Unknown(_) => {}
                    _ => {}
                }
            }
        }
        _ => return Err(FbxLoadError::UnsupportedVersion),
    }

    todo!();
}
