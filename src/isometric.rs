use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::window::PrimaryWindow;

// Camera pitch tuned so projected tiles appear with a classic 2:1 isometric ratio.
const CAMERA_EYE_OFFSET: (f32, f32, f32) = (-1.0, 0.816_496_6, 1.0);
const CAMERA_DISTANCE_SCALE: f32 = 2.2;

const DRAG_PAN_SCALE: f32 = 0.5;
const TRACKPAD_PAN_SCALE: f32 = 0.012;
const SCROLL_ZOOM_RATE: f32 = 0.1;
pub const INITIAL_ZOOM: f32 = 10.0;
const MIN_ZOOM: f32 = 4.0;
const MAX_ZOOM: f32 = 30.0;

#[derive(Resource)]
pub struct IsoCamera {
    target: Vec2,
    zoom: f32,
}

impl IsoCamera {
    pub fn new(target: Vec2, zoom: f32) -> Self {
        Self { target, zoom }
    }
}

#[derive(Component)]
pub struct IsoCameraTag;

pub fn spawn_iso_camera(mut commands: Commands) {
    let camera = IsoCamera::new(Vec2::ZERO, INITIAL_ZOOM);
    let target = Vec3::new(camera.target.x, 0.0, camera.target.y);
    let position = target + iso_eye_direction() * (camera.zoom * CAMERA_DISTANCE_SCALE);

    commands.insert_resource(camera);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(position).looking_at(target, Vec3::Y),
            projection: OrthographicProjection {
                scale: 1.0,
                scaling_mode: ScalingMode::FixedVertical(INITIAL_ZOOM),
                near: -1000.0,
                far: 1000.0,
                ..default()
            }
            .into(),
            tonemapping: Tonemapping::None,
            ..default()
        },
        IsoCameraTag,
    ));
}

pub fn update_iso_camera(
    mut camera: ResMut<IsoCamera>,
    mut motion_events: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Transform, &mut Projection), With<IsoCameraTag>>,
) {
    let mut pan_delta = Vec2::ZERO;
    for motion in motion_events.iter() {
        pan_delta += motion.delta;
    }

    let mut scroll_delta = Vec2::ZERO;
    for event in scroll_events.iter() {
        let mut delta = Vec2::new(event.x, event.y);
        if matches!(event.unit, MouseScrollUnit::Line) {
            delta *= 16.0;
        }
        scroll_delta += delta;
    }

    if !(mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Right)) {
        pan_delta = Vec2::ZERO;
    }

    let window = windows.get_single().ok();
    let (safe_width, safe_height) = if let Some(window) = window {
        (window.width().max(1.0), window.height().max(1.0))
    } else {
        (1.0, 1.0)
    };
    let aspect = safe_width / safe_height;
    let view_height = camera.zoom;
    let view_width = camera.zoom * aspect;
    let (pan_axis_x, pan_axis_y) = iso_pan_axes();

    if pan_delta.length_squared() > 0.0 {
        let world_delta = pan_axis_x * (pan_delta.x * view_width * 0.5 * DRAG_PAN_SCALE)
            + pan_axis_y * (-pan_delta.y * view_height * 0.5 * DRAG_PAN_SCALE);
        camera.target -= Vec2::new(world_delta.x, world_delta.z);
    }

    if scroll_delta.length_squared() > 0.0 {
        if zoom_modifier_active(&keys) {
            camera.zoom = (camera.zoom * (1.0 - scroll_delta.y * SCROLL_ZOOM_RATE))
                .clamp(MIN_ZOOM, MAX_ZOOM);
        } else {
            let world_delta = pan_axis_x * (scroll_delta.x * view_width * TRACKPAD_PAN_SCALE)
                + pan_axis_y * (-scroll_delta.y * view_height * TRACKPAD_PAN_SCALE);
            camera.target -= Vec2::new(world_delta.x, world_delta.z);
        }
    }

    let target = Vec3::new(camera.target.x, 0.0, camera.target.y);
    let position = target + iso_eye_direction() * (camera.zoom * CAMERA_DISTANCE_SCALE);

    for (mut transform, mut projection) in &mut query {
        transform.translation = position;
        transform.look_at(target, Vec3::Y);
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = 1.0;
            ortho.scaling_mode = ScalingMode::FixedVertical(camera.zoom);
        }
    }
}

fn iso_eye_direction() -> Vec3 {
    Vec3::new(
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

fn zoom_modifier_active(keys: &Input<KeyCode>) -> bool {
    keys.pressed(KeyCode::AltLeft)
        || keys.pressed(KeyCode::AltRight)
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
}
