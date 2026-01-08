use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

const GRID_WIDTH: usize = 16;
const GRID_HEIGHT: usize = 16;
const TILE_WORLD_SIZE: f32 = 1.0;

#[macroquad::main("dustfal")]
async fn main() {
    let grid_mesh = build_grid_mesh();

    loop {
        // Clear in screen space first
        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

        let iso_camera = Camera3D {
            position: vec3(30.0, 30.0, 30.0),
            target: vec3(0.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
            projection: Projection::Orthographics,
            fovy: 10.0,
            ..Default::default()
        };
        set_camera(&iso_camera);

        draw_mesh(&grid_mesh);

        set_default_camera();
        draw_text("Top-down checkerboard", 10.0, 28.0, 28.0, WHITE);

        next_frame().await;
    }
}

fn build_grid_mesh() -> Mesh {
    let mut vertices = Vec::with_capacity(GRID_WIDTH * GRID_HEIGHT * 4);
    let mut indices = Vec::with_capacity(GRID_WIDTH * GRID_HEIGHT * 6);

    let half_w = GRID_WIDTH as f32 * TILE_WORLD_SIZE * 0.5;
    let half_h = GRID_HEIGHT as f32 * TILE_WORLD_SIZE * 0.5;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let world_x = x as f32 * TILE_WORLD_SIZE - half_w;
            let world_z = y as f32 * TILE_WORLD_SIZE - half_h;
            let color = if (x + y) % 2 == 0 {
                Color::new(0.85, 0.1, 0.75, 1.0)
            } else {
                Color::new(0.06, 0.06, 0.08, 1.0)
            };

            push_tile(
                &mut vertices,
                &mut indices,
                world_x,
                world_z,
                TILE_WORLD_SIZE,
                color,
            );
        }
    }

    Mesh {
        vertices,
        indices,
        texture: None,
    }
}

fn push_tile(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    world_x: f32,
    world_z: f32,
    size: f32,
    color: Color,
) {
    let x0 = world_x;
    let z0 = world_z;
    let x1 = world_x + size;
    let z1 = world_z + size;
    let y = 0.0;

    let base = vertices.len() as u16;
    vertices.extend_from_slice(&[
        Vertex::new(x0, y, z0, 0.0, 0.0, color),
        Vertex::new(x1, y, z0, 1.0, 0.0, color),
        Vertex::new(x1, y, z1, 1.0, 1.0, color),
        Vertex::new(x0, y, z1, 0.0, 1.0, color),
    ]);
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
