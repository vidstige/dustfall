use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

mod engine;
mod isometric;
mod texture_atlas;

use engine::{add_human, add_moxie, Engine, Gas, Volume};
use isometric::{build_camera, update_camera, IsoCamera, INITIAL_ZOOM};
use texture_atlas::{load_tile_atlas, TileAtlas};

const GRID_WIDTH: usize = 256;
const GRID_HEIGHT: usize = 256;
const TILE_WORLD_SIZE: f32 = 1.0;
const CHUNK_SIZE: usize = 16;
const TILE_ATLAS_COLUMNS: usize = 8;

struct TileMap {
    width: usize,
    height: usize,
    tiles: Vec<u8>,
}

fn thin_atmosphere(total_moles: i32) -> Gas {
    // The reported composition is a volume (molar) ratio, so we treat it as mole fractions.
    const CO2_PARTS_PER_10K: i32 = 9_532; // 95.32%
    const O2_PARTS_PER_10K: i32 = 13; // 1.3 permille
    let co2 = total_moles * CO2_PARTS_PER_10K / 10_000;
    let o2 = total_moles * O2_PARTS_PER_10K / 10_000;
    Gas { o2, co2, co: 0 }
}

#[macroquad::main("Dustfall")]
async fn main() {
    let mut engine = Engine::new(Volume::new(1000), thin_atmosphere(10_000));
    let root = engine.root();
    let base = engine.add_container(root, Volume::new(100), Gas {
        o2: 2000,
        co2: 8000,
        co: 0,
    });
    add_human(&mut engine, base, 3);
    add_moxie(&mut engine, base, 2, 1, 2);

    let map = checker_board(GRID_WIDTH, GRID_HEIGHT);
    let tile_atlas = load_tile_atlas("images/topdown.png", TILE_ATLAS_COLUMNS).await;

    let mut camera_state = IsoCamera::new(Vec2::ZERO, INITIAL_ZOOM);

    let grid_meshes = build_grid_meshes(&map, &tile_atlas);

    loop {
        // Clear in screen space first
        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

        engine.tick();

        update_camera(&mut camera_state);

        let iso_camera = build_camera(&map, &camera_state);
        set_camera(&iso_camera);

        for mesh in &grid_meshes {
            draw_mesh(mesh);
        }

        set_default_camera();
        draw_text(
            "Drag or scroll to pan (hold Alt for zoom)",
            10.0,
            28.0,
            24.0,
            WHITE,
        );

        next_frame().await;
    }
}

fn build_grid_meshes(map: &TileMap, atlas: &TileAtlas) -> Vec<Mesh> {
    assert!(
        map.width % CHUNK_SIZE == 0 && map.height % CHUNK_SIZE == 0,
        "map dimensions must be divisible by chunk size"
    );

    let chunks_x = map.width / CHUNK_SIZE;
    let chunks_y = map.height / CHUNK_SIZE;
    let half_w = map.width as f32 * TILE_WORLD_SIZE * 0.5;
    let half_h = map.height as f32 * TILE_WORLD_SIZE * 0.5;

    let mut meshes = Vec::with_capacity(chunks_x * chunks_y);
    for chunk_y in 0..chunks_y {
        for chunk_x in 0..chunks_x {
            meshes.push(build_chunk_mesh(
                map, atlas, chunk_x, chunk_y, half_w, half_h,
            ));
        }
    }

    meshes
}

fn build_chunk_mesh(
    map: &TileMap,
    atlas: &TileAtlas,
    chunk_x: usize,
    chunk_y: usize,
    half_w: f32,
    half_h: f32,
) -> Mesh {
    let mut vertices = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * 4);
    let mut indices = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * 6);

    let tile_x_start = chunk_x * CHUNK_SIZE;
    let tile_y_start = chunk_y * CHUNK_SIZE;

    for local_y in 0..CHUNK_SIZE {
        for local_x in 0..CHUNK_SIZE {
            let tile_x = tile_x_start + local_x;
            let tile_y = tile_y_start + local_y;
            let world_x = tile_x as f32 * TILE_WORLD_SIZE - half_w;
            let world_z = tile_y as f32 * TILE_WORLD_SIZE - half_h;
            let tile_index = map.tile_index(tile_x, tile_y) as usize;
            let (uv_min, uv_max) = atlas.uv_bounds(tile_index);

            push_tile(
                &mut vertices,
                &mut indices,
                world_x,
                world_z,
                TILE_WORLD_SIZE,
                uv_min,
                uv_max,
            );
        }
    }

    Mesh {
        vertices,
        indices,
        texture: Some(atlas.texture.clone()),
    }
}

impl TileMap {
    fn tile_index(&self, x: usize, y: usize) -> u8 {
        self.tiles[y * self.width + x]
    }
}

fn checker_board(width: usize, height: usize) -> TileMap {
    let mut tiles = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let tile_index = if (x + y) % 2 == 0 { 0 } else { 1 };
            tiles.push(tile_index);
        }
    }

    TileMap {
        width,
        height,
        tiles,
    }
}

fn push_tile(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    world_x: f32,
    world_z: f32,
    size: f32,
    uv_min: Vec2,
    uv_max: Vec2,
) {
    let x0 = world_x;
    let z0 = world_z;
    let x1 = world_x + size;
    let z1 = world_z + size;
    let y = 0.0;

    let base = vertices.len() as u16;
    vertices.extend_from_slice(&[
        Vertex::new(x0, y, z0, uv_min.x, uv_min.y, WHITE),
        Vertex::new(x1, y, z0, uv_max.x, uv_min.y, WHITE),
        Vertex::new(x1, y, z1, uv_max.x, uv_max.y, WHITE),
        Vertex::new(x0, y, z1, uv_min.x, uv_max.y, WHITE),
    ]);
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
