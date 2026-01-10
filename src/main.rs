use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

const GRID_WIDTH: usize = 256;
const GRID_HEIGHT: usize = 256;
const TILE_WORLD_SIZE: f32 = 1.0;
const CHUNK_SIZE: usize = 16;
const TILE_TEXTURE_COUNT: usize = 32;

struct TileMap {
    width: usize,
    height: usize,
    tiles: Vec<u8>,
}

#[macroquad::main("Dustfall")]
async fn main() {
    let map = checker_board(GRID_WIDTH, GRID_HEIGHT);
    let _tile_textures = load_tile_textures(TILE_TEXTURE_COUNT).await;

    let mut camera_state = IsoCamera {
        target: Vec2::ZERO,
        zoom: INITIAL_ZOOM,
        active_touch_id: None,
        last_touch_pos: None,
    };

    let grid_meshes = build_grid_meshes(&map);

    loop {
        // Clear in screen space first
        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

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

fn build_grid_meshes(map: &TileMap) -> Vec<Mesh> {
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
            meshes.push(build_chunk_mesh(map, chunk_x, chunk_y, half_w, half_h));
        }
    }

    meshes
}

fn build_chunk_mesh(
    map: &TileMap,
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
            let tile_index = map.tile_index(tile_x, tile_y);
            let color = match tile_index {
                0 => Color::new(0.85, 0.1, 0.75, 1.0),
                _ => Color::new(0.06, 0.06, 0.08, 1.0),
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

async fn load_tile_textures(count: usize) -> Vec<Texture2D> {
    let mut textures = Vec::with_capacity(count);
    for index in 0..count {
        let path = format!("images/tiles/{}.png", index);
        let texture = load_texture(&path)
            .await
            .unwrap_or_else(|err| panic!("failed to load {path}: {err}"));
        texture.set_filter(FilterMode::Nearest);
        textures.push(texture);
    }

    textures
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

// Camera pitch tuned so projected tiles appear with a classic 2:1 isometric ratio.
const CAMERA_EYE_OFFSET: (f32, f32, f32) = (-1.0, 0.816_496_6, 1.0);
const CAMERA_DISTANCE_SCALE: f32 = 2.2;

fn build_camera(map: &TileMap, camera: &IsoCamera) -> Camera3D {
    let longest_side = (map.width.max(map.height) as f32 * TILE_WORLD_SIZE).max(1.0);
    let target = vec3(camera.target.x, 0.0, camera.target.y);
    let view_dir = iso_eye_direction();
    let distance = camera.zoom * CAMERA_DISTANCE_SCALE;
    let position = target + view_dir * distance;

    Camera3D {
        position,
        target,
        up: vec3(0.0, 1.0, 0.0),
        projection: Projection::Orthographics,
        fovy: camera.zoom,
        z_near: 0.05,
        z_far: longest_side * 10.0,
        ..Default::default()
    }
}

fn iso_eye_direction() -> Vec3 {
    vec3(
        CAMERA_EYE_OFFSET.0,
        CAMERA_EYE_OFFSET.1,
        CAMERA_EYE_OFFSET.2,
    )
    .normalize()
}

fn iso_camera_forward() -> Vec3 {
    -iso_eye_direction()
}

fn iso_pan_axes() -> (Vec3, Vec3) {
    plane_axes_from_forward(iso_camera_forward())
}

fn plane_axes_from_forward(forward: Vec3) -> (Vec3, Vec3) {
    const EPS: f32 = 1e-6;

    let plane_normal = Vec3::Y;
    let mut right = forward.cross(plane_normal);
    if right.length_squared() < EPS {
        right = Vec3::X;
    }

    // Keep movement parallel to the plane.
    let mut planar_right = right - plane_normal * right.dot(plane_normal);
    if planar_right.length_squared() < EPS {
        planar_right = Vec3::X;
    }
    planar_right = planar_right.normalize();

    let planar_forward = plane_normal.cross(planar_right).normalize();
    (planar_right, planar_forward)
}

struct IsoCamera {
    target: Vec2,
    zoom: f32,
    active_touch_id: Option<u64>,
    last_touch_pos: Option<Vec2>,
}
const DRAG_PAN_SCALE: f32 = 0.5;
const TRACKPAD_PAN_SCALE: f32 = 0.012;
const SCROLL_ZOOM_RATE: f32 = 0.1;
const INITIAL_ZOOM: f32 = 10.0;
const MIN_ZOOM: f32 = 4.0;
const MAX_ZOOM: f32 = 30.0;

fn update_camera(camera: &mut IsoCamera) {
    let mut pan_delta = Vec2::ZERO;

    if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
        pan_delta += mouse_delta_position();
        camera.active_touch_id = None;
        camera.last_touch_pos = None;
    } else if let Some(touch_delta) = touch_pan_delta(camera) {
        pan_delta += touch_delta;
    } else {
        camera.active_touch_id = None;
        camera.last_touch_pos = None;
    }

    let safe_width = screen_width().max(1.0);
    let safe_height = screen_height().max(1.0);
    let aspect = safe_width / safe_height;
    let view_height = camera.zoom;
    let view_width = camera.zoom * aspect;
    let (pan_axis_x, pan_axis_y) = iso_pan_axes();

    if pan_delta.length_squared() > 0.0 {
        let world_delta = pan_axis_x * (pan_delta.x * view_width * 0.5 * DRAG_PAN_SCALE)
            + pan_axis_y * (-pan_delta.y * view_height * 0.5 * DRAG_PAN_SCALE);
        camera.target -= vec2(world_delta.x, world_delta.z);
    }

    let (scroll_x, scroll_y) = mouse_wheel();
    if scroll_x.abs() > f32::EPSILON || scroll_y.abs() > f32::EPSILON {
        if zoom_modifier_active() {
            camera.zoom =
                (camera.zoom * (1.0 - scroll_y * SCROLL_ZOOM_RATE)).clamp(MIN_ZOOM, MAX_ZOOM);
        } else {
            let world_delta = pan_axis_x * (scroll_x * view_width * TRACKPAD_PAN_SCALE)
                + pan_axis_y * (-scroll_y * view_height * TRACKPAD_PAN_SCALE);
            camera.target -= vec2(world_delta.x, world_delta.z);
        }
    }
}

fn zoom_modifier_active() -> bool {
    is_key_down(KeyCode::LeftAlt)
        || is_key_down(KeyCode::RightAlt)
        || is_key_down(KeyCode::LeftControl)
        || is_key_down(KeyCode::RightControl)
}

fn touch_pan_delta(camera: &mut IsoCamera) -> Option<Vec2> {
    let mut touches = touches_local();
    if touches.is_empty() {
        return None;
    }

    touches.sort_by_key(|t| t.id);

    let current = if let Some(id) = camera.active_touch_id {
        touches.into_iter().find(|t| t.id == id)
    } else {
        touches
            .into_iter()
            .find(|t| matches!(t.phase, TouchPhase::Started | TouchPhase::Moved))
    };

    let touch = current?;

    match touch.phase {
        TouchPhase::Started => {
            camera.active_touch_id = Some(touch.id);
            camera.last_touch_pos = Some(touch.position);
            None
        }
        TouchPhase::Moved => {
            camera.active_touch_id = Some(touch.id);
            let delta = camera.last_touch_pos.map(|last| last - touch.position);
            camera.last_touch_pos = Some(touch.position);
            delta
        }
        TouchPhase::Ended | TouchPhase::Cancelled | TouchPhase::Stationary => {
            if camera.active_touch_id == Some(touch.id) {
                camera.active_touch_id = None;
                camera.last_touch_pos = None;
            }
            None
        }
    }
}
