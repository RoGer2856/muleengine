use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::Read,
    path::PathBuf,
    sync::Arc,
};

use ab_glyph::{Font as AbGlyphFont, FontVec, InvalidFont, ScaleFont};
use vek::Vec2;

use crate::image::Image;

pub struct GlyphRenderer {
    font: FontVec,
}

#[derive(Clone)]
pub struct RenderedGlyph {
    image: Arc<Image>,
    pixel_scale: usize,
    bounds_min: Vec2<f32>,
    _bounds_max: Vec2<f32>,
    h_advance: f32,
    v_advance: f32,
}

impl RenderedGlyph {
    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    pub fn pixel_scale(&self) -> usize {
        self.pixel_scale
    }

    pub fn h_advance(&self) -> f32 {
        self.h_advance
    }

    pub fn v_advance(&self) -> f32 {
        self.v_advance
    }

    pub fn compute_render_offset_px(&self) -> Vec2<f32> {
        Vec2::new(self.bounds_min.x, self.bounds_min.y)
    }
}

pub struct FontContainer {
    glyph_renderer: GlyphRenderer,
    glyph_images: HashMap<(char, usize), RenderedGlyph>,
}

#[derive(Debug)]
pub enum FontLoadError {
    FileOpen(std::io::Error),
    FileReadError(std::io::Error),
    InvalidFont(InvalidFont),
}

impl GlyphRenderer {
    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, FontLoadError> {
        Ok(Self {
            font: FontVec::try_from_vec(bytes).map_err(FontLoadError::InvalidFont)?,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FontLoadError> {
        Self::from_vec(Vec::from(bytes))
    }

    pub fn from_file(path: PathBuf) -> Result<Self, FontLoadError> {
        let mut file = File::open(path).map_err(FontLoadError::FileOpen)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(FontLoadError::FileReadError)?;

        Self::from_vec(buffer)
    }

    pub fn render_empty(&self, pixel_scale: usize) -> Image {
        Image::from_luma_alpha_u8_closure(pixel_scale, pixel_scale, |_x, _y| (0, 0))
    }

    pub fn render_glyph(&self, chr: char, pixel_scale: usize) -> Option<RenderedGlyph> {
        let pixel_scale_f32 = pixel_scale as f32;

        let glyph_id = self.font.glyph_id(chr);

        let scaled_font = self.font.as_scaled(pixel_scale_f32);
        let h_advance = scaled_font.h_advance(glyph_id);
        let v_advance = scaled_font.v_advance(glyph_id);

        let glyph = glyph_id.with_scale(pixel_scale_f32);
        if let Some(outlined_glyph) = self.font.outline_glyph(glyph) {
            let mut image = self.render_empty(pixel_scale);
            outlined_glyph.draw(|x, y, c| {
                let _ = image.set_color_f32_at(x as usize, y as usize, (1.0, 1.0, 1.0, c));
            });

            let px_bounds = outlined_glyph.px_bounds();

            Some(RenderedGlyph {
                pixel_scale,
                image: Arc::new(image),
                bounds_min: Vec2::new(px_bounds.min.x, px_bounds.min.y),
                _bounds_max: Vec2::new(px_bounds.max.x, px_bounds.max.y),
                h_advance,
                v_advance,
            })
        } else {
            None
        }
    }
}

impl FontContainer {
    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, FontLoadError> {
        Ok(Self {
            glyph_renderer: GlyphRenderer::from_vec(bytes)?,
            glyph_images: HashMap::new(),
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FontLoadError> {
        Ok(Self {
            glyph_renderer: GlyphRenderer::from_bytes(bytes)?,
            glyph_images: HashMap::new(),
        })
    }

    pub fn from_file(path: PathBuf) -> Result<Self, FontLoadError> {
        Ok(Self {
            glyph_renderer: GlyphRenderer::from_file(path)?,
            glyph_images: HashMap::new(),
        })
    }

    pub fn get_rendered_glyph(&mut self, chr: char, pixel_scale: usize) -> Option<RenderedGlyph> {
        match self.glyph_images.entry((chr, pixel_scale)) {
            Entry::Occupied(entry) => Some(entry.get().clone()),
            Entry::Vacant(entry) => self
                .glyph_renderer
                .render_glyph(chr, pixel_scale)
                .map(|rendered_glyph| entry.insert(rendered_glyph).clone()),
        }
    }
}

pub struct HackFontContainer(FontContainer);

impl Default for HackFontContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl HackFontContainer {
    pub fn new() -> Self {
        match FontContainer::from_bytes(include_bytes!("hack-font/Hack-Bold.ttf")) {
            Ok(font_container) => Self(font_container),
            Err(_) => unreachable!(),
        }
    }

    pub fn get_rendered_glyph(&mut self, chr: char, pixel_scale: usize) -> Option<RenderedGlyph> {
        self.0.get_rendered_glyph(chr, pixel_scale)
    }
}
