use std::convert::TryFrom;

use image::ImageError;

#[derive(Copy, Clone)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Tiff,
    Tga,
    Bmp,
    Ico,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum ColorType {
    L8,
    La8,
    Rgb8,
    Rgba8,
    L16,
    La16,
    Rgb16,
    Rgba16,
    RgbF32,
    RgbaF32,
}

impl TryFrom<image::ColorType> for ColorType {
    type Error = ();

    fn try_from(color_type: image::ColorType) -> Result<Self, Self::Error> {
        match color_type {
            image::ColorType::L8 => Ok(ColorType::L8),
            image::ColorType::La8 => Ok(ColorType::La8),
            image::ColorType::Rgb8 => Ok(ColorType::Rgb8),
            image::ColorType::Rgba8 => Ok(ColorType::Rgba8),
            image::ColorType::L16 => Ok(ColorType::L16),
            image::ColorType::La16 => Ok(ColorType::La16),
            image::ColorType::Rgb16 => Ok(ColorType::Rgb16),
            image::ColorType::Rgba16 => Ok(ColorType::Rgba16),
            image::ColorType::Rgb32F => Ok(ColorType::RgbF32),
            image::ColorType::Rgba32F => Ok(ColorType::RgbaF32),
            _ => Err(()),
        }
    }
}

fn get_dimensions(image: &image::DynamicImage) -> (u32, u32) {
    match image {
        image::DynamicImage::ImageLuma8(image) => image.dimensions(),
        image::DynamicImage::ImageLumaA8(image) => image.dimensions(),
        image::DynamicImage::ImageRgb8(image) => image.dimensions(),
        image::DynamicImage::ImageRgba8(image) => image.dimensions(),
        image::DynamicImage::ImageLuma16(image) => image.dimensions(),
        image::DynamicImage::ImageLumaA16(image) => image.dimensions(),
        image::DynamicImage::ImageRgb16(image) => image.dimensions(),
        image::DynamicImage::ImageRgba16(image) => image.dimensions(),
        _ => unreachable!(),
    }
}

pub enum ImageSaveError {
    ImageError(ImageError),
}

pub struct Image {
    image: image::DynamicImage,
    width: usize,
    height: usize,
    color_type: ColorType,
}

impl Image {
    pub fn new(width: usize, height: usize, color_type: ColorType) -> Self {
        let image = match color_type {
            ColorType::L8 => image::DynamicImage::new_luma8(width as u32, height as u32),
            ColorType::L16 => image::DynamicImage::new_luma16(width as u32, height as u32),
            ColorType::La8 => image::DynamicImage::new_luma_a8(width as u32, height as u32),
            ColorType::La16 => image::DynamicImage::new_luma_a16(width as u32, height as u32),
            ColorType::Rgb8 => image::DynamicImage::new_rgb8(width as u32, height as u32),
            ColorType::Rgba8 => image::DynamicImage::new_rgba8(width as u32, height as u32),
            ColorType::Rgb16 => image::DynamicImage::new_rgb16(width as u32, height as u32),
            ColorType::Rgba16 => image::DynamicImage::new_rgba16(width as u32, height as u32),
            ColorType::RgbF32 => image::DynamicImage::new_rgb32f(width as u32, height as u32),
            ColorType::RgbaF32 => image::DynamicImage::new_rgba32f(width as u32, height as u32),
        };

        Self {
            image,
            width,
            height,
            color_type,
        }
    }

    pub fn from_reader(mut reader: impl std::io::Read) -> Option<Self> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).ok()?;

        let mut image = image::io::Reader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .ok()?
            .decode()
            .ok()?;

        let color_type = if let Ok(color_type) = ColorType::try_from(image.color()) {
            color_type
        } else {
            image = image::DynamicImage::ImageRgba32F(image.into_rgba32f());
            ColorType::RgbaF32
        };

        let (width, height) = get_dimensions(&image);

        Some(Self {
            image,
            width: width as usize,
            height: height as usize,
            color_type,
        })
    }

    pub fn save(
        &self,
        writer: &mut (impl std::io::Write + std::io::Seek),
        format: ImageFormat,
    ) -> Result<(), ImageSaveError> {
        let format = match format {
            ImageFormat::Bmp => image::ImageFormat::Bmp,
            ImageFormat::Gif => image::ImageFormat::Gif,
            ImageFormat::Ico => image::ImageFormat::Ico,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Tga => image::ImageFormat::Tga,
            ImageFormat::Tiff => image::ImageFormat::Tiff,
        };

        self.image
            .write_to(writer, format)
            .map_err(ImageSaveError::ImageError)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn color_type(&self) -> ColorType {
        self.color_type
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.image.as_bytes()
    }

    pub fn color_at_f32(&self, x: usize, y: usize) -> (f32, f32, f32, f32) {
        let x = x.clamp(0, self.width);
        let y = y.clamp(0, self.height);

        if x >= self.width || y >= self.height {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            let x = x as u32;
            let y = y as u32;

            match &self.image {
                image::DynamicImage::ImageLuma8(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0] as f32 / 255.0;
                    let g = r;
                    let b = r;
                    let a = 1.0;
                    (r, g, b, a)
                }
                image::DynamicImage::ImageLumaA8(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0] as f32 / 255.0;
                    let g = r;
                    let b = r;
                    let a = p[1] as f32 / 255.0;
                    (r, g, b, a)
                }
                image::DynamicImage::ImageRgb8(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0] as f32 / 255.0;
                    let g = p[1] as f32 / 255.0;
                    let b = p[2] as f32 / 255.0;
                    let a = 1.0;
                    (r, g, b, a)
                }
                image::DynamicImage::ImageRgba8(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0] as f32 / 255.0;
                    let g = p[1] as f32 / 255.0;
                    let b = p[2] as f32 / 255.0;
                    let a = p[3] as f32 / 255.0;
                    (r, g, b, a)
                }
                image::DynamicImage::ImageLuma16(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0] as f32 / 65535.0;
                    let g = r;
                    let b = r;
                    let a = 1.0;
                    (r, g, b, a)
                }
                image::DynamicImage::ImageLumaA16(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0] as f32 / 65535.0;
                    let g = r;
                    let b = r;
                    let a = p[1] as f32 / 65535.0;
                    (r, g, b, a)
                }
                image::DynamicImage::ImageRgb16(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0] as f32 / 65535.0;
                    let g = p[1] as f32 / 65535.0;
                    let b = p[2] as f32 / 65535.0;
                    let a = 1.0;
                    (r, g, b, a)
                }
                image::DynamicImage::ImageRgba16(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0] as f32 / 65535.0;
                    let g = p[1] as f32 / 65535.0;
                    let b = p[2] as f32 / 65535.0;
                    let a = p[3] as f32 / 65535.0;
                    (r, g, b, a)
                }
                image::DynamicImage::ImageRgb32F(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0];
                    let g = p[1];
                    let b = p[2];
                    let a = p[3];
                    (r, g, b, a)
                }
                image::DynamicImage::ImageRgba32F(image) => {
                    let p = image.get_pixel(x, y);
                    let r = p[0];
                    let g = p[1];
                    let b = p[2];
                    let a = p[3];
                    (r, g, b, a)
                }
                _ => unreachable!(),
            }
        }
    }
}
