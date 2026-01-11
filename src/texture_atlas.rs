use macroquad::prelude::*;

pub struct TileAtlas {
    pub texture: Texture2D,
    columns: usize,
    rows: usize,
    tile_count: usize,
}

impl TileAtlas {
    pub fn uv_bounds(&self, index: usize) -> (Vec2, Vec2) {
        let tile_index = index % self.tile_count;
        let column = tile_index % self.columns;
        let row = tile_index / self.columns;
        let u0 = column as f32 / self.columns as f32;
        let v0 = row as f32 / self.rows as f32;
        let u1 = (column + 1) as f32 / self.columns as f32;
        let v1 = (row + 1) as f32 / self.rows as f32;

        (vec2(u0, v0), vec2(u1, v1))
    }
}

pub async fn load_tile_atlas(path: &str, columns: usize) -> TileAtlas {
    assert!(columns > 0, "tile atlas columns must be non-zero");

    let atlas = load_image(path)
        .await
        .unwrap_or_else(|err| panic!("failed to load {path}: {err}"));
    let width = atlas.width as usize;
    let height = atlas.height as usize;
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
    let tile_count = columns * rows;

    let texture = Texture2D::from_image(&atlas);
    texture.set_filter(FilterMode::Nearest);

    TileAtlas {
        texture,
        columns,
        rows,
        tile_count,
    }
}
