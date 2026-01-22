use bevy::prelude::*;

pub struct NormalMapAtlas {
    pub handle: Handle<Image>,
    columns: usize,
    rows: usize,
}

impl NormalMapAtlas {
    pub fn from_heightmap(
        image: &Image,
        patch_size: usize,
        handle: Handle<Image>,
    ) -> Self {
        assert!(patch_size > 0, "heightmap patch size must be non-zero");
        let width = image.texture_descriptor.size.width as usize;
        let height = image.texture_descriptor.size.height as usize;
        assert!(
            width % patch_size == 0 && height % patch_size == 0,
            "heightmap size must be divisible by patch size"
        );

        let columns = width / patch_size;
        let rows = height / patch_size;
        assert!(columns > 0 && rows > 0, "heightmap atlas is empty");

        Self {
            handle,
            columns,
            rows,
        }
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
