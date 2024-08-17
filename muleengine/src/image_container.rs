use std::collections::HashMap;
use std::sync::Arc;

use super::asset_reader::AssetReader;
use super::image::Image;

pub struct ImageContainer {
    images: HashMap<String, Arc<Image>>,
}

#[derive(Debug, Clone)]
pub enum ImageContainerError {
    CannotOpenAsset { path: String },
    CannotDecodeAssetAsImage { path: String },
}

impl Default for ImageContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageContainer {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    pub fn get_image(
        &mut self,
        image_path: impl AsRef<str>,
        asset_reader: &AssetReader,
    ) -> Result<Arc<Image>, ImageContainerError> {
        if let Some(image) = self.images.get(image_path.as_ref()) {
            Ok(image.clone())
        } else if let Some(asset_reader) = asset_reader.get_reader(image_path.as_ref()) {
            if let Some(image) = Image::from_reader(asset_reader) {
                let image = Arc::new(image);
                self.images
                    .insert(image_path.as_ref().to_string(), image.clone());

                Ok(image)
            } else {
                Err(ImageContainerError::CannotDecodeAssetAsImage {
                    path: image_path.as_ref().to_string(),
                })
            }
        } else {
            Err(ImageContainerError::CannotOpenAsset {
                path: image_path.as_ref().to_string(),
            })
        }
    }
}
