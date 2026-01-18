use bevy::prelude::*;

pub struct TileAtlas {
    columns: usize,
    rows: usize,
}

impl TileAtlas {
    pub fn from_image(image: &Image, columns: usize) -> Self {
        assert!(columns > 0, "tile atlas columns must be non-zero");

        let width = image.texture_descriptor.size.width as usize;
        let height = image.texture_descriptor.size.height as usize;
        let tile_width = width / columns;
        assert!(tile_width > 0, "tile atlas width is too small");
        assert!(
            width % columns == 0,
            "tile atlas width must be divisible by columns"
        );
        assert!(
            height % tile_width == 0,
            "tile atlas height must be divisible by tile width"
        );

        let rows = height / tile_width;
        Self { columns, rows }
    }

    pub fn uv_bounds(&self, index: usize) -> (Vec2, Vec2) {
        let tile_index = index % (self.columns * self.rows);
        let column = tile_index % self.columns;
        let row = tile_index / self.columns;
        let u0 = column as f32 / self.columns as f32;
        let v0 = row as f32 / self.rows as f32;
        let u1 = (column + 1) as f32 / self.columns as f32;
        let v1 = (row + 1) as f32 / self.rows as f32;

        (Vec2::new(u0, v0), Vec2::new(u1, v1))
    }
}
