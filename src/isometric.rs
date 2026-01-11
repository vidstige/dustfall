use macroquad::prelude::*;

use crate::{TileMap, TILE_WORLD_SIZE};

// Camera pitch tuned so projected tiles appear with a classic 2:1 isometric ratio.
const CAMERA_EYE_OFFSET: (f32, f32, f32) = (-1.0, 0.816_496_6, 1.0);
const CAMERA_DISTANCE_SCALE: f32 = 2.2;

const DRAG_PAN_SCALE: f32 = 0.5;
const TRACKPAD_PAN_SCALE: f32 = 0.012;
const SCROLL_ZOOM_RATE: f32 = 0.1;
pub const INITIAL_ZOOM: f32 = 10.0;
const MIN_ZOOM: f32 = 4.0;
const MAX_ZOOM: f32 = 30.0;

pub struct IsoCamera {
    target: Vec2,
    zoom: f32,
    active_touch_id: Option<u64>,
    last_touch_pos: Option<Vec2>,
}

impl IsoCamera {
    pub fn new(target: Vec2, zoom: f32) -> Self {
        Self {
            target,
            zoom,
            active_touch_id: None,
            last_touch_pos: None,
        }
    }
}

pub fn build_camera(map: &TileMap, camera: &IsoCamera) -> Camera3D {
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

pub fn update_camera(camera: &mut IsoCamera) {
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
