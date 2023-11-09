use super::image::Image;

pub struct HeightMap {
    height_map: Vec<Vec<f32>>,
    mask_map: Vec<Vec<u8>>,
    column_count: usize,
    row_count: usize,
}

impl HeightMap {
    pub fn from_images(
        relief_map_image: &Image,
        mask_map_image: Option<&Image>,
    ) -> Result<Self, &'static str> {
        #![allow(clippy::needless_range_loop)]

        if relief_map_image.width() == 0 || relief_map_image.height() == 0 {
            Err("the dimensions of the relief maps are 0")
        } else {
            let mut height_map = Vec::new();

            let mut v = Vec::new();
            v.resize(relief_map_image.height(), 0.0);
            height_map.resize(relief_map_image.width(), v);

            for x in 0..relief_map_image.width() {
                for y in 0..relief_map_image.height() {
                    height_map[x][y] = relief_map_image.color_at_f32(x, y).0;
                }
            }

            let mut mask_map = Vec::new();
            let mut v = Vec::new();
            v.resize(relief_map_image.height(), 1);
            mask_map.resize(relief_map_image.width(), v);

            if let Some(mask_map_image) = mask_map_image {
                if relief_map_image.width() != mask_map_image.width()
                    || relief_map_image.height() != mask_map_image.height()
                {
                    return Err("the dimensions of relief and mask map are not matching");
                } else {
                    for x in 0..mask_map_image.width() {
                        for y in 0..mask_map_image.height() {
                            mask_map[x][y] = (mask_map_image.color_at_f32(x, y).0 * 255.0) as u8;
                        }
                    }
                }
            }

            Ok(Self {
                height_map,
                mask_map,
                column_count: relief_map_image.width(),
                row_count: relief_map_image.height(),
            })
        }
    }

    pub fn get_column_count(&self) -> usize {
        self.column_count
    }

    pub fn get_row_count(&self) -> usize {
        self.row_count
    }

    pub fn get_height_map(&self) -> &Vec<Vec<f32>> {
        &self.height_map
    }

    pub fn get_mask_map(&self) -> &Vec<Vec<u8>> {
        &self.mask_map
    }

    pub fn get_height_at(&self, x: f32, y: f32) -> f32 {
        // todo!("the four points are not always on the same plane")
        /*
         *
         *    A----------B
         *    |   |      |
         *    |   |      |
         *   AC---P------BD
         *    |   |      |
         *    C----------D
         *
         */
        let min_x = (x as isize).clamp(0, self.column_count as isize) as usize;
        let max_x = (x as isize + 1).clamp(0, self.column_count as isize) as usize;

        let min_y = (y as isize).clamp(0, self.row_count as isize) as usize;
        let max_y = (y as isize + 1).clamp(0, self.row_count as isize) as usize;

        let a = self.height_map[min_x][max_y];
        let b = self.height_map[max_x][max_y];
        let c = self.height_map[min_x][min_y];
        let d = self.height_map[max_x][min_y];

        let q_vertical = y - min_y as f32;
        let q_horizontal = x - min_x as f32;

        let ac = c * q_vertical + a * (1.0 - q_vertical);
        let bd = d * q_vertical + b * (1.0 - q_vertical);

        ac * q_horizontal + bd * (1.0 - q_horizontal)
    }
}
