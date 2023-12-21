use std::{
    convert::TryFrom,
    ops::{Bound, Range, RangeBounds},
};

use image::ImageError;
use vek::Vec2;

#[derive(Debug, Copy, Clone)]
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
#[derive(Debug, Copy, Clone)]
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

#[derive(Debug)]
pub enum ImageSaveError {
    ImageError(ImageError),
}

#[derive(Debug, Clone)]
pub enum ColorSetError {
    IndexOutOfBounds(usize, usize),
    InvalidColorType {
        received: ColorType,
        actual: ColorType,
    },
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
            ColorType::La8 => image::DynamicImage::new_luma_a8(width as u32, height as u32),
            ColorType::L16 => image::DynamicImage::new_luma16(width as u32, height as u32),
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

    pub fn from_luma_u8_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> u8,
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageLuma8(image),
            width,
            height,
            color_type: ColorType::L8,
        }
    }

    pub fn from_luma_alpha_u8_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> (u8, u8),
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::LumaA<u8>, Vec<u8>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c.0, c.1];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageLumaA8(image),
            width,
            height,
            color_type: ColorType::La8,
        }
    }

    pub fn from_luma_u16_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> u16,
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageLuma16(image),
            width,
            height,
            color_type: ColorType::L16,
        }
    }

    pub fn from_luma_alpha_u16_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> (u16, u16),
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::LumaA<u16>, Vec<u16>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c.0, c.1];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageLumaA16(image),
            width,
            height,
            color_type: ColorType::La16,
        }
    }

    pub fn from_rgb_u8_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> (u8, u8, u8),
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c.0, c.1, c.2];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageRgb8(image),
            width,
            height,
            color_type: ColorType::Rgb8,
        }
    }

    pub fn from_rgba_u8_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> (u8, u8, u8, u8),
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c.0, c.1, c.2, c.3];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageRgba8(image),
            width,
            height,
            color_type: ColorType::Rgba8,
        }
    }

    pub fn from_rgb_u16_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> (u16, u16, u16),
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c.0, c.1, c.2];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageRgb16(image),
            width,
            height,
            color_type: ColorType::Rgb16,
        }
    }

    pub fn from_rgba_u16_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> (u16, u16, u16, u16),
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::Rgba<u16>, Vec<u16>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c.0, c.1, c.2, c.3];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageRgba16(image),
            width,
            height,
            color_type: ColorType::Rgba16,
        }
    }

    pub fn from_rgb_f32_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> (f32, f32, f32),
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::Rgb<f32>, Vec<f32>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c.0, c.1, c.2];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageRgb32F(image),
            width,
            height,
            color_type: ColorType::RgbF32,
        }
    }

    pub fn from_rgba_f32_closure(
        width: usize,
        height: usize,
        closure: impl Fn(usize, usize) -> (f32, f32, f32, f32),
    ) -> Self {
        let mut image =
            image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let c = closure(x, y);
                    pixel.0 = [c.0, c.1, c.2, c.3];
                }
            }
        }

        Self {
            image: image::DynamicImage::ImageRgba32F(image),
            width,
            height,
            color_type: ColorType::RgbaF32,
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

    pub fn set_color_luma_u8_at(
        &mut self,
        x: usize,
        y: usize,
        color: u8,
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageLuma8(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::L8,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_luma_alpha_u8_at(
        &mut self,
        x: usize,
        y: usize,
        color: (u8, u8),
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageLumaA8(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color.0, color.1];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::La8,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_luma_u16_at(
        &mut self,
        x: usize,
        y: usize,
        color: u16,
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageLuma16(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::L16,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_luma_alpha_u16_at(
        &mut self,
        x: usize,
        y: usize,
        color: (u16, u16),
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageLumaA16(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color.0, color.1];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::La16,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_rgb_u8_at(
        &mut self,
        x: usize,
        y: usize,
        color: (u8, u8, u8),
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageRgb8(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color.0, color.1, color.2];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::Rgb8,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_rgba_u8_at(
        &mut self,
        x: usize,
        y: usize,
        color: (u8, u8, u8, u8),
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageRgba8(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color.0, color.1, color.2, color.3];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::Rgba8,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_rgb_u16_at(
        &mut self,
        x: usize,
        y: usize,
        color: (u16, u16, u16),
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageRgb16(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color.0, color.1, color.2];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::Rgb16,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_rgba_u16_at(
        &mut self,
        x: usize,
        y: usize,
        color: (u16, u16, u16, u16),
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageRgba16(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color.0, color.1, color.2, color.3];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::Rgba16,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_rgb_f32_at(
        &mut self,
        x: usize,
        y: usize,
        color: (f32, f32, f32),
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageRgb32F(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color.0, color.1, color.2];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::RgbF32,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_rgba_f32_at(
        &mut self,
        x: usize,
        y: usize,
        color: (f32, f32, f32, f32),
    ) -> Result<(), ColorSetError> {
        if let image::DynamicImage::ImageRgba32F(image) = &mut self.image {
            if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                pixel.0 = [color.0, color.1, color.2, color.3];
                Ok(())
            } else {
                Err(ColorSetError::IndexOutOfBounds(x, y))
            }
        } else {
            Err(ColorSetError::InvalidColorType {
                received: ColorType::RgbaF32,
                actual: self.color_type,
            })
        }
    }

    pub fn set_color_f32_at(
        &mut self,
        x: usize,
        y: usize,
        color: (f32, f32, f32, f32),
    ) -> Result<(), ColorSetError> {
        match &mut self.image {
            image::DynamicImage::ImageLuma8(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let luma = ((color.0 + color.1 + color.2) / 3.0 * u8::MAX as f32)
                        .clamp(0.0, u8::MAX as f32) as u8;
                    pixel.0 = [luma];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageLumaA8(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let luma = ((color.0 + color.1 + color.2) / 3.0 * u8::MAX as f32)
                        .clamp(0.0, u8::MAX as f32) as u8;
                    let alpha = (color.3 * u8::MAX as f32).clamp(0.0, u8::MAX as f32) as u8;
                    pixel.0 = [luma, alpha];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageLuma16(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let luma = ((color.0 + color.1 + color.2) / 3.0 * u16::MAX as f32)
                        .clamp(0.0, u16::MAX as f32) as u16;
                    pixel.0 = [luma];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageLumaA16(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let luma = ((color.0 + color.1 + color.2) / 3.0 * u16::MAX as f32)
                        .clamp(0.0, u16::MAX as f32) as u16;
                    let alpha = (color.3 * u16::MAX as f32).clamp(0.0, u16::MAX as f32) as u16;
                    pixel.0 = [luma, alpha];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageRgb8(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let r = (color.0 * u8::MAX as f32).clamp(0.0, u8::MAX as f32) as u8;
                    let g = (color.1 * u8::MAX as f32).clamp(0.0, u8::MAX as f32) as u8;
                    let b = (color.2 * u8::MAX as f32).clamp(0.0, u8::MAX as f32) as u8;
                    pixel.0 = [r, g, b];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageRgba8(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let r = (color.0 * u8::MAX as f32).clamp(0.0, u8::MAX as f32) as u8;
                    let g = (color.1 * u8::MAX as f32).clamp(0.0, u8::MAX as f32) as u8;
                    let b = (color.2 * u8::MAX as f32).clamp(0.0, u8::MAX as f32) as u8;
                    let a = (color.3 * u8::MAX as f32).clamp(0.0, u8::MAX as f32) as u8;
                    pixel.0 = [r, g, b, a];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageRgb16(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let r = (color.0 * u16::MAX as f32).clamp(0.0, u16::MAX as f32) as u16;
                    let g = (color.1 * u16::MAX as f32).clamp(0.0, u16::MAX as f32) as u16;
                    let b = (color.2 * u16::MAX as f32).clamp(0.0, u16::MAX as f32) as u16;
                    pixel.0 = [r, g, b];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageRgba16(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let r = (color.0 * u16::MAX as f32).clamp(0.0, u16::MAX as f32) as u16;
                    let g = (color.1 * u16::MAX as f32).clamp(0.0, u16::MAX as f32) as u16;
                    let b = (color.2 * u16::MAX as f32).clamp(0.0, u16::MAX as f32) as u16;
                    let a = (color.3 * u16::MAX as f32).clamp(0.0, u16::MAX as f32) as u16;
                    pixel.0 = [r, g, b, a];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageRgb32F(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let r = color.0;
                    let g = color.1;
                    let b = color.2;
                    pixel.0 = [r, g, b];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            image::DynamicImage::ImageRgba32F(image) => {
                if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                    let r = color.0;
                    let g = color.1;
                    let b = color.2;
                    let a = color.3;
                    pixel.0 = [r, g, b, a];
                    Ok(())
                } else {
                    Err(ColorSetError::IndexOutOfBounds(x, y))
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn color_f32_at(&self, x: usize, y: usize) -> Option<(f32, f32, f32, f32)> {
        let x = x as u32;
        let y = y as u32;

        match &self.image {
            image::DynamicImage::ImageLuma8(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0] as f32 / 255.0;
                let g = r;
                let b = r;
                let a = 1.0;
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageLumaA8(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0] as f32 / 255.0;
                let g = r;
                let b = r;
                let a = p[1] as f32 / 255.0;
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageRgb8(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0] as f32 / 255.0;
                let g = p[1] as f32 / 255.0;
                let b = p[2] as f32 / 255.0;
                let a = 1.0;
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageRgba8(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0] as f32 / 255.0;
                let g = p[1] as f32 / 255.0;
                let b = p[2] as f32 / 255.0;
                let a = p[3] as f32 / 255.0;
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageLuma16(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0] as f32 / 65535.0;
                let g = r;
                let b = r;
                let a = 1.0;
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageLumaA16(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0] as f32 / 65535.0;
                let g = r;
                let b = r;
                let a = p[1] as f32 / 65535.0;
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageRgb16(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0] as f32 / 65535.0;
                let g = p[1] as f32 / 65535.0;
                let b = p[2] as f32 / 65535.0;
                let a = 1.0;
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageRgba16(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0] as f32 / 65535.0;
                let g = p[1] as f32 / 65535.0;
                let b = p[2] as f32 / 65535.0;
                let a = p[3] as f32 / 65535.0;
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageRgb32F(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0];
                let g = p[1];
                let b = p[2];
                let a = p[3];
                Some((r, g, b, a))
            }
            image::DynamicImage::ImageRgba32F(image) => {
                let p = image.get_pixel_checked(x, y)?;
                let r = p[0];
                let g = p[1];
                let b = p[2];
                let a = p[3];
                Some((r, g, b, a))
            }
            _ => unreachable!(),
        }
    }

    pub fn copy_image_to_self(
        &mut self,
        self_offset: Vec2<isize>,
        source: &Image,
        source_range: Range<Vec2<isize>>,
    ) {
        let mut self_position = Vec2::new(0, 0);

        let source_start = match source_range.start_bound() {
            Bound::Excluded(value) => *value,
            Bound::Included(value) => value + Vec2::new(1, 1),
            Bound::Unbounded => Vec2::new(0, 0),
        };

        let source_end = match source_range.end_bound() {
            Bound::Excluded(value) => *value,
            Bound::Included(value) => value + Vec2::new(1, 1),
            Bound::Unbounded => Vec2::new(self.width as isize, self.height as isize),
        };

        for source_y in source_start.y..source_end.y {
            for source_x in source_start.x..source_end.x {
                if source_x < 0 && source_y < 0 && self_position.x < 0 && self_position.y < 0 {
                    continue;
                }

                if let Some(color) = source.color_f32_at(source_x as usize, source_y as usize) {
                    let self_position = self_offset + self_position;
                    let _ = self.set_color_f32_at(
                        self_position.x as usize,
                        self_position.y as usize,
                        color,
                    );
                }

                self_position.x += 1;
            }

            self_position.x = 0;
            self_position.y += 1;
        }
    }

    pub fn blend_image_to_self(
        &mut self,
        self_offset: Vec2<isize>,
        source: &Image,
        source_range: impl RangeBounds<Vec2<isize>>,
    ) {
        let mut self_position = Vec2::new(0, 0);

        let source_start = match source_range.start_bound() {
            Bound::Excluded(value) => *value,
            Bound::Included(value) => value + Vec2::new(1, 1),
            Bound::Unbounded => Vec2::new(0, 0),
        };

        let source_end = match source_range.end_bound() {
            Bound::Excluded(value) => *value,
            Bound::Included(value) => value + Vec2::new(1, 1),
            Bound::Unbounded => Vec2::new(self.width as isize, self.height as isize),
        };

        for source_y in source_start.y..source_end.y {
            for source_x in source_start.x..source_end.x {
                if source_x < 0 && source_y < 0 && self_position.x < 0 && self_position.y < 0 {
                    continue;
                }

                if let Some(source_color) =
                    source.color_f32_at(source_x as usize, source_y as usize)
                {
                    let self_position = self_offset + self_position;
                    if let Some(self_color) =
                        self.color_f32_at(self_position.x as usize, self_position.y as usize)
                    {
                        let self_opacity = self_color.3;
                        let self_transparency = 1.0 - self_opacity;

                        let source_opacity = source_color.3;
                        let source_transparency = 1.0 - source_opacity;

                        let red =
                            source_color.0 * source_opacity + self_color.0 * source_transparency;
                        let green =
                            source_color.1 * source_opacity + self_color.1 * source_transparency;
                        let blue =
                            source_color.2 * source_opacity + self_color.2 * source_transparency;
                        let transparency = self_transparency * source_transparency;

                        let _ = self.set_color_f32_at(
                            self_position.x as usize,
                            self_position.y as usize,
                            (red, green, blue, 1.0 - transparency),
                        );
                    }
                }

                self_position.x += 1;
            }

            self_position.x = 0;
            self_position.y += 1;
        }
    }
}
