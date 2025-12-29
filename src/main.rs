use macroquad::prelude::*;
use macroquad::texture::FilterMode;

const GRID_WIDTH: usize = 16;
const GRID_HEIGHT: usize = 16;
const TILE_WIDTH: f32 = 64.0;
const TILE_HEIGHT: f32 = 32.0;
const TILE_VARIANTS: usize = 32;

struct TileMap {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}

impl TileMap {
    fn new(width: usize, height: usize, seed: u32) -> Self {
        let mut value = seed;
        let mut indices = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            value = value.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            indices.push((value % TILE_VARIANTS as u32) as u8);
        }

        Self {
            width,
            height,
            indices,
        }
    }

    fn tile_index(&self, x: usize, y: usize) -> usize {
        self.indices[y * self.width + x] as usize
    }
}

struct TileSet {
    textures: Vec<Texture2D>,
}

impl TileSet {
    async fn load() -> Self {
        let mut textures = Vec::with_capacity(TILE_VARIANTS);
        for idx in 0..TILE_VARIANTS {
            let path = format!("images/tiles/{}.png", idx);
            let texture = load_texture(&path)
                .await
                .unwrap_or_else(|e| panic!("Failed to load {}: {}", path, e));
            texture.set_filter(FilterMode::Nearest);
            textures.push(texture);
        }
        Self { textures }
    }

    fn texture(&self, index: usize) -> &Texture2D {
        &self.textures[index]
    }
}

#[macroquad::main("Dustfall Isometric Checkered Plane")]
async fn main() {
    let map = TileMap::new(GRID_WIDTH, GRID_HEIGHT, 42);
    let tiles = TileSet::load().await;

    loop {
        clear_background(Color::from_rgba(15, 18, 27, 255));

        let anchor = vec2(screen_width() * 0.5, screen_height() * 0.4);
        draw_plane(anchor, &map, &tiles);

        draw_text(
            "Macroquad textured plane (press Esc to exit)",
            16.0,
            34.0,
            28.0,
            WHITE,
        );

        next_frame().await;
    }
}

fn draw_plane(anchor: Vec2, map: &TileMap, tiles: &TileSet) {
    let diag_count = map.width + map.height - 1;
    for diag in 0..diag_count {
        let x_min = diag.saturating_sub(map.height - 1);
        let x_max = diag.min(map.width - 1);
        if x_min > x_max {
            continue;
        }

        for x in x_min..=x_max {
            let y = diag - x;
            let center = iso_to_screen(x as f32, y as f32, anchor);
            let tile_index = map.tile_index(x, y);
            draw_tile(center, tile_index, tiles);
        }
    }
}

fn draw_tile(center: Vec2, tile_index: usize, tiles: &TileSet) {
    let texture = tiles.texture(tile_index);
    let top_left = center - vec2(TILE_WIDTH * 0.5, TILE_HEIGHT * 0.5);
    draw_texture_ex(
        texture,
        top_left.x,
        top_left.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(TILE_WIDTH, TILE_HEIGHT)),
            ..Default::default()
        },
    );
}

fn iso_to_screen(x: f32, y: f32, anchor: Vec2) -> Vec2 {
    let iso = vec2((x - y) * TILE_WIDTH * 0.5, (x + y) * TILE_HEIGHT * 0.5);
    iso + anchor
}
