use std::collections::HashMap;
use std::sync::Arc;

use super::assets_reader::AssetsReader;
use super::image::Image;

#[derive(Clone)]
pub struct ImageContainer {
    images: HashMap<String, Arc<Image>>,
}

pub enum ImageContainerError {
    CannotOpenAsset { path: String },
    CannotDecodeAssetAsImage { path: String },
}

impl ImageContainer {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    pub fn get_image(
        &mut self,
        image_path: &str,
        assets_reader: &AssetsReader,
    ) -> Result<Arc<Image>, ImageContainerError> {
        if let Some(image_mut) = self.images.get_mut(image_path) {
            Ok(image_mut.clone())
        } else {
            if let Some(assets_reader) = assets_reader.get_reader(image_path) {
                if let Some(image) = Image::from_reader(assets_reader) {
                    let image = Arc::new(image);
                    self.images.insert(image_path.to_string(), image.clone());

                    Ok(image)
                } else {
                    Err(ImageContainerError::CannotDecodeAssetAsImage {
                        path: image_path.to_string(),
                    })
                }
            } else {
                Err(ImageContainerError::CannotOpenAsset {
                    path: image_path.to_string(),
                })
            }
        }
    }
}
