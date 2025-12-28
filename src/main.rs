use macroquad::prelude::*;

const GRID_WIDTH: usize = 16;
const GRID_HEIGHT: usize = 16;
const TILE_WIDTH: f32 = 64.0;
const TILE_HEIGHT: f32 = 32.0;

struct TileMap {
    width: usize,
    height: usize,
    indices: Vec<u32>,
}

impl TileMap {
    fn new(width: usize, height: usize, seed: u32) -> Self {
        let mut value = seed;
        let mut indices = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            value = value.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            indices.push(value);
        }

        Self {
            width,
            height,
            indices,
        }
    }

    fn tile_index(&self, x: usize, y: usize) -> u32 {
        self.indices[y * self.width + x]
    }
}

#[macroquad::main("Dustfall Isometric Checkered Plane")]
async fn main() {
    let map = TileMap::new(GRID_WIDTH, GRID_HEIGHT, 42);

    loop {
        clear_background(Color::from_rgba(15, 18, 27, 255));

        let anchor = vec2(screen_width() * 0.5, screen_height() * 0.4);
        draw_plane(anchor, &map);

        draw_text(
            "Macroquad checkered plane (press Esc to exit)",
            16.0,
            34.0,
            28.0,
            WHITE,
        );

        next_frame().await;
    }
}

fn draw_plane(anchor: Vec2, map: &TileMap) {
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
            let color_toggle = map.tile_index(x, y) % 2 == 0;
            draw_tile(center, color_toggle);
        }
    }
}

fn draw_tile(center: Vec2, color_toggle: bool) {
    let half_w = TILE_WIDTH * 0.5;
    let half_h = TILE_HEIGHT * 0.5;

    let top = center + vec2(0.0, -half_h);
    let right = center + vec2(half_w, 0.0);
    let bottom = center + vec2(0.0, half_h);
    let left = center + vec2(-half_w, 0.0);

    let light = Color::from_rgba(127, 180, 196, 255);
    let dark = Color::from_rgba(60, 82, 94, 255);
    let tile_color = if color_toggle { light } else { dark };

    draw_triangle(top, right, left, tile_color);
    draw_triangle(bottom, right, left, tile_color);

    let outline = Color::from_rgba(12, 22, 33, 255);
    draw_line(left.x, left.y, top.x, top.y, 1.0, outline);
    draw_line(top.x, top.y, right.x, right.y, 1.0, outline);
    draw_line(right.x, right.y, bottom.x, bottom.y, 1.0, outline);
    draw_line(bottom.x, bottom.y, left.x, left.y, 1.0, outline);
}

fn iso_to_screen(x: f32, y: f32, anchor: Vec2) -> Vec2 {
    let iso = vec2((x - y) * TILE_WIDTH * 0.5, (x + y) * TILE_HEIGHT * 0.5);
    iso + anchor
}
