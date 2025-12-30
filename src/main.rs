use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;
use macroquad::texture::{load_image, FilterMode, Image};

const GRID_WIDTH: usize = 16;
const GRID_HEIGHT: usize = 16;
const TILE_WIDTH: f32 = 64.0;
const TILE_HEIGHT: f32 = 32.0;
const TILE_VARIANTS: usize = 32;
const ATLAS_COLUMNS: usize = 8;
const ATLAS_ROWS: usize = (TILE_VARIANTS + ATLAS_COLUMNS - 1) / ATLAS_COLUMNS;
const SCROLL_PAN_SPEED: f32 = 4.0;
const DRAG_PAN_SCALE: f32 = 0.45;

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

struct TileAtlas {
    texture: Texture2D,
    columns: usize,
    tile_w: f32,
    tile_h: f32,
    atlas_w: f32,
    atlas_h: f32,
}

impl TileAtlas {
    async fn load() -> Self {
        let mut tile_images = Vec::with_capacity(TILE_VARIANTS);
        for idx in 0..TILE_VARIANTS {
            let path = format!("images/tiles/{}.png", idx);
            let image = load_image(&path)
                .await
                .unwrap_or_else(|e| panic!("Failed to load {}: {}", path, e));
            tile_images.push(image);
        }

        let tile_w = tile_images
            .get(0)
            .map(|img| img.width() as u32)
            .unwrap_or(TILE_WIDTH as u32);
        let tile_h = tile_images
            .get(0)
            .map(|img| img.height() as u32)
            .unwrap_or(TILE_HEIGHT as u32);

        let atlas_w = (ATLAS_COLUMNS as u32) * tile_w;
        let atlas_h = (ATLAS_ROWS as u32) * tile_h;
        let mut atlas_image = Image::gen_image_color(
            atlas_w as u16,
            atlas_h as u16,
            Color::new(0.0, 0.0, 0.0, 0.0),
        );

        for (idx, image) in tile_images.iter().enumerate() {
            let col = (idx % ATLAS_COLUMNS) as u32;
            let row = (idx / ATLAS_COLUMNS) as u32;
            blit_image(&mut atlas_image, image, col * tile_w, row * tile_h);
        }

        let texture = Texture2D::from_image(&atlas_image);
        texture.set_filter(FilterMode::Nearest);

        Self {
            texture,
            columns: ATLAS_COLUMNS,
            tile_w: tile_w as f32,
            tile_h: tile_h as f32,
            atlas_w: atlas_w as f32,
            atlas_h: atlas_h as f32,
        }
    }

    fn texture(&self) -> &Texture2D {
        &self.texture
    }

    fn uv_rect(&self, index: usize) -> Rect {
        let col = (index % self.columns) as f32;
        let row = (index / self.columns) as f32;
        let u0 = (col * self.tile_w) / self.atlas_w;
        let v0 = (row * self.tile_h) / self.atlas_h;
        let u1 = ((col + 1.0) * self.tile_w) / self.atlas_w;
        let v1 = ((row + 1.0) * self.tile_h) / self.atlas_h;

        Rect {
            x: u0,
            y: v0,
            w: u1 - u0,
            h: v1 - v0,
        }
    }
}

fn blit_image(dest: &mut Image, src: &Image, offset_x: u32, offset_y: u32) {
    let width = src.width() as u32;
    let height = src.height() as u32;

    for y in 0..height {
        for x in 0..width {
            let color = src.get_pixel(x, y);
            dest.set_pixel(offset_x + x, offset_y + y, color);
        }
    }
}

struct IsoCamera {
    offset: Vec2,
    active_touch_id: Option<u64>,
    last_touch_pos: Option<Vec2>,
}

struct TileBatch {
    mesh: Mesh,
}

impl TileBatch {
    fn new() -> Self {
        Self {
            mesh: Mesh {
                vertices: Vec::new(),
                indices: Vec::new(),
                texture: None,
            },
        }
    }

    fn begin(&mut self, texture: &Texture2D) {
        self.mesh.vertices.clear();
        self.mesh.indices.clear();
        self.mesh.texture = Some(texture.clone());
    }

    fn push_tile(&mut self, center: Vec2, uv_rect: Rect) {
        if self.mesh.vertices.len() > u16::MAX as usize - 4 {
            return;
        }

        let half_w = TILE_WIDTH * 0.5;
        let half_h = TILE_HEIGHT * 0.5;

        let top_left = center + vec2(-half_w, -half_h);
        let top_right = center + vec2(half_w, -half_h);
        let bottom_right = center + vec2(half_w, half_h);
        let bottom_left = center + vec2(-half_w, half_h);

        let u0 = uv_rect.x;
        let v0 = uv_rect.y;
        let u1 = uv_rect.x + uv_rect.w;
        let v1 = uv_rect.y + uv_rect.h;

        let base = self.mesh.vertices.len() as u16;
        self.mesh.vertices.extend_from_slice(&[
            Vertex::new(top_left.x, top_left.y, 0.0, u0, v0, WHITE),
            Vertex::new(top_right.x, top_right.y, 0.0, u1, v0, WHITE),
            Vertex::new(bottom_right.x, bottom_right.y, 0.0, u1, v1, WHITE),
            Vertex::new(bottom_left.x, bottom_left.y, 0.0, u0, v1, WHITE),
        ]);
        self.mesh
            .indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn draw(&self) {
        if !self.mesh.vertices.is_empty() && !self.mesh.indices.is_empty() {
            draw_mesh(&self.mesh);
        }
    }
}

#[macroquad::main("dustfal")]
async fn main() {
    let map = TileMap::new(GRID_WIDTH, GRID_HEIGHT, 42);
    let atlas = TileAtlas::load().await;
    let mut camera = create_camera(&map);
    let mut batch = TileBatch::new();

    loop {
        update_camera(&mut camera);
        clear_background(Color::from_rgba(15, 18, 27, 255));

        let anchor = vec2(screen_width() * 0.5, screen_height() * 0.4);
        draw_plane(anchor, &map, &atlas, &camera, &mut batch);

        draw_text("Drag mouse/touchpad to pan", 16.0, 34.0, 28.0, WHITE);

        next_frame().await;
    }
}

fn draw_plane(
    anchor: Vec2,
    map: &TileMap,
    atlas: &TileAtlas,
    camera: &IsoCamera,
    batch: &mut TileBatch,
) {
    batch.begin(atlas.texture());
    for y in 0..map.height {
        for x in 0..map.width {
            let center = iso_to_screen(x as f32, y as f32, camera, anchor);
            let tile_index = map.tile_index(x, y);
            let uv = atlas.uv_rect(tile_index);
            batch.push_tile(center, uv);
        }
    }
    batch.draw();
}

fn iso_to_screen(x: f32, y: f32, camera: &IsoCamera, anchor: Vec2) -> Vec2 {
    let iso = iso_coords(x, y);
    (iso - camera.offset) + anchor
}

fn iso_coords(x: f32, y: f32) -> Vec2 {
    vec2((x - y) * TILE_WIDTH * 0.5, (x + y) * TILE_HEIGHT * 0.5)
}

fn create_camera(map: &TileMap) -> IsoCamera {
    let center = vec2(map.width as f32 * 0.5, map.height as f32 * 0.5);
    let iso_center = iso_coords(center.x, center.y);
    IsoCamera {
        offset: iso_center,
        active_touch_id: None,
        last_touch_pos: None,
    }
}

fn update_camera(camera: &mut IsoCamera) {
    let mut pan_delta = Vec2::ZERO;

    if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
        pan_delta += mouse_delta_position();
        camera.active_touch_id = None;
        camera.last_touch_pos = None;
    } else if let Some(touch_delta) = camera_touch_drag_delta(camera) {
        pan_delta += touch_delta;
    } else {
        camera.active_touch_id = None;
        camera.last_touch_pos = None;
    }

    if pan_delta.length_squared() > 0.0 {
        let pixel_delta = vec2(
            -pan_delta.x * screen_width() * 0.5,
            -pan_delta.y * screen_height() * 0.5,
        ) * DRAG_PAN_SCALE;
        camera.offset += pixel_delta;
    }

    let (wheel_x, wheel_y) = mouse_wheel();
    if wheel_x.abs() > 0.0 || wheel_y.abs() > 0.0 {
        camera.offset += vec2(wheel_x, wheel_y) * -SCROLL_PAN_SPEED;
    }
}

fn camera_touch_drag_delta(camera: &mut IsoCamera) -> Option<Vec2> {
    let mut touches = touches_local();
    if touches.is_empty() {
        return None;
    }

    touches.sort_by_key(|touch| touch.id);

    let active = if let Some(id) = camera.active_touch_id {
        touches.into_iter().find(|touch| touch.id == id)
    } else {
        touches.into_iter().find(|touch| {
            matches!(
                touch.phase,
                TouchPhase::Started | TouchPhase::Moved | TouchPhase::Stationary
            )
        })
    };

    let touch = active?;

    match touch.phase {
        TouchPhase::Started => {
            camera.active_touch_id = Some(touch.id);
            camera.last_touch_pos = Some(touch.position);
            None
        }
        TouchPhase::Moved | TouchPhase::Stationary => {
            camera.active_touch_id = Some(touch.id);
            let delta = camera.last_touch_pos.map(|last| last - touch.position);
            camera.last_touch_pos = Some(touch.position);
            delta
        }
        TouchPhase::Ended | TouchPhase::Cancelled => {
            if camera.active_touch_id == Some(touch.id) {
                camera.active_touch_id = None;
                camera.last_touch_pos = None;
            }
            None
        }
    }
}
