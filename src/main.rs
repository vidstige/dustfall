use macroquad::prelude::*;
use macroquad::texture::FilterMode;

const GRID_WIDTH: usize = 16;
const GRID_HEIGHT: usize = 16;
const TILE_WIDTH: f32 = 64.0;
const TILE_HEIGHT: f32 = 32.0;
const TILE_VARIANTS: usize = 32;
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

struct IsoCamera {
    offset: Vec2,
    active_touch_id: Option<u64>,
    last_touch_pos: Option<Vec2>,
}

#[macroquad::main("Dustfal")]
async fn main() {
    let map = TileMap::new(GRID_WIDTH, GRID_HEIGHT, 42);
    let tiles = TileSet::load().await;
    let mut camera = create_camera(&map);

    loop {
        update_camera(&mut camera);
        clear_background(Color::from_rgba(15, 18, 27, 255));

        let anchor = vec2(screen_width() * 0.5, screen_height() * 0.4);
        draw_plane(anchor, &map, &tiles, &camera);

        draw_text("Drag mouse/touchpad to pan", 16.0, 34.0, 28.0, WHITE);

        next_frame().await;
    }
}

fn draw_plane(anchor: Vec2, map: &TileMap, tiles: &TileSet, camera: &IsoCamera) {
    let bounds = compute_visible_bounds(map, camera, anchor);
    for diag in bounds.diag_min..=bounds.diag_max {
        let diag = diag as usize;
        let mut x_min = diag.saturating_sub(map.height - 1);
        let mut x_max = diag.min(map.width - 1);
        x_min = x_min.max(bounds.x_min);
        x_max = x_max.min(bounds.x_max);
        if x_min > x_max {
            continue;
        }

        for x in x_min..=x_max {
            let y = diag - x;
            if y < bounds.y_min || y > bounds.y_max {
                continue;
            }

            let center = iso_to_screen(x as f32, y as f32, camera, anchor);
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

fn iso_to_screen(x: f32, y: f32, camera: &IsoCamera, anchor: Vec2) -> Vec2 {
    let iso = iso_coords(x, y);
    (iso - camera.offset) + anchor
}

fn iso_coords(x: f32, y: f32) -> Vec2 {
    vec2((x - y) * TILE_WIDTH * 0.5, (x + y) * TILE_HEIGHT * 0.5)
}

struct VisibleTileBounds {
    x_min: usize,
    x_max: usize,
    y_min: usize,
    y_max: usize,
    diag_min: usize,
    diag_max: usize,
}

fn compute_visible_bounds(map: &TileMap, camera: &IsoCamera, anchor: Vec2) -> VisibleTileBounds {
    let corners = [
        vec2(0.0, 0.0),
        vec2(screen_width(), 0.0),
        vec2(0.0, screen_height()),
        vec2(screen_width(), screen_height()),
    ];

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for corner in corners {
        let iso = screen_to_iso(corner, camera, anchor);
        let tile = iso_to_tile_coords(iso);
        min_x = min_x.min(tile.x);
        max_x = max_x.max(tile.x);
        min_y = min_y.min(tile.y);
        max_y = max_y.max(tile.y);
    }

    if !min_x.is_finite() {
        return VisibleTileBounds {
            x_min: 0,
            x_max: map.width.saturating_sub(1),
            y_min: 0,
            y_max: map.height.saturating_sub(1),
            diag_min: 0,
            diag_max: map.width + map.height - 2,
        };
    }

    let margin = 4.0;
    let width_limit = (map.width.saturating_sub(1)) as f32;
    let height_limit = (map.height.saturating_sub(1)) as f32;

    let x_min = ((min_x - margin).floor()).max(0.0).min(width_limit) as usize;
    let x_max = ((max_x + margin).ceil()).max(0.0).min(width_limit) as usize;
    let y_min = ((min_y - margin).floor()).max(0.0).min(height_limit) as usize;
    let y_max = ((max_y + margin).ceil()).max(0.0).min(height_limit) as usize;

    let diag_min = x_min.saturating_add(y_min);
    let diag_max = (x_max + y_max).min(map.width + map.height - 2);

    VisibleTileBounds {
        x_min,
        x_max,
        y_min,
        y_max,
        diag_min,
        diag_max,
    }
}

fn screen_to_iso(screen: Vec2, camera: &IsoCamera, anchor: Vec2) -> Vec2 {
    screen - anchor + camera.offset
}

fn iso_to_tile_coords(iso: Vec2) -> Vec2 {
    let half_w = TILE_WIDTH * 0.5;
    let half_h = TILE_HEIGHT * 0.5;
    let x = (iso.y / half_h + iso.x / half_w) * 0.5;
    let y = (iso.y / half_h - iso.x / half_w) * 0.5;
    vec2(x, y)
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
