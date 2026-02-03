use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::touchpad::TouchpadMagnify;
use bevy::prelude::*;
use bevy::render::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::window::PrimaryWindow;

// Camera pitch tuned so projected tiles appear with a classic 2:1 isometric ratio.
const CAMERA_EYE_OFFSET: (f32, f32, f32) = (-1.0, 0.816_496_6, 1.0);
const CAMERA_DISTANCE_SCALE: f32 = 2.2;

const TRACKPAD_PAN_SCALE: f32 = 0.1;
const SCROLL_ZOOM_RATE: f32 = 0.02;
const MAGNIFY_ZOOM_RATE: f32 = 1.0;
pub const INITIAL_ZOOM: f32 = 10.0;
const MIN_ZOOM: f32 = 4.0;
const MAX_ZOOM: f32 = 32.0;

#[derive(Resource)]
pub struct IsoCamera {
    target: Vec2,
    zoom: f32,
    last_cursor_pos: Option<Vec2>,
}

impl IsoCamera {
    pub fn new(target: Vec2, zoom: f32) -> Self {
        Self {
            target,
            zoom,
            last_cursor_pos: None,
        }
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
    mut scroll_events: EventReader<MouseWheel>,
    mut magnify_events: EventReader<TouchpadMagnify>,
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&Camera, &GlobalTransform, &mut Transform, &mut Projection), With<IsoCameraTag>>,
) {
    let mut scroll_delta = Vec2::ZERO;
    for event in scroll_events.iter() {
        let mut delta = Vec2::new(event.x, event.y);
        if matches!(event.unit, MouseScrollUnit::Line) {
            delta *= 16.0;
        }
        scroll_delta += delta;
    }
    let mut magnify_delta = 0.0;
    for event in magnify_events.iter() {
        magnify_delta += event.0;
    }

    let window = windows.get_single().ok();
    let cursor_pos = window.and_then(|window| window.cursor_position());
    let dragging = mouse_buttons.pressed(MouseButton::Right);
    if !dragging {
        camera.last_cursor_pos = None;
    }

    for (camera_component, camera_transform, mut transform, mut projection) in &mut query {
        if dragging {
            if let (Some(current_pos), Some(last_pos)) = (cursor_pos, camera.last_cursor_pos) {
                if let Some(world_delta) =
                    cursor_pan_delta(camera_component, camera_transform, last_pos, current_pos)
                {
                    camera.target += world_delta;
                }
            }
            camera.last_cursor_pos = cursor_pos;
        }

        if magnify_delta.abs() > 0.0 {
            camera.zoom = (camera.zoom * (1.0 - magnify_delta * MAGNIFY_ZOOM_RATE))
                .clamp(MIN_ZOOM, MAX_ZOOM);
        } else if scroll_delta.length_squared() > 0.0 {
            if zoom_modifier_active(&keys) {
                camera.zoom = (camera.zoom * (1.0 + scroll_delta.y * SCROLL_ZOOM_RATE))
                    .clamp(MIN_ZOOM, MAX_ZOOM);
            } else {
                if let Some(current_pos) = cursor_pos {
                    let scroll_pan = scroll_delta * TRACKPAD_PAN_SCALE;
                    let scaled_pos = current_pos + scroll_pan;
                    if let Some(world_delta) = cursor_pan_delta(
                        camera_component,
                        camera_transform,
                        current_pos,
                        scaled_pos,
                    ) {
                        camera.target += world_delta;
                    }
                }
            }
        }

        let target = Vec3::new(camera.target.x, 0.0, camera.target.y);
        let position = target + iso_eye_direction() * (camera.zoom * CAMERA_DISTANCE_SCALE);
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

fn zoom_modifier_active(keys: &Input<KeyCode>) -> bool {
    keys.pressed(KeyCode::AltLeft)
        || keys.pressed(KeyCode::AltRight)
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
}

fn cursor_pan_delta(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    from: Vec2,
    to: Vec2,
) -> Option<Vec2> {
    let world_from = cursor_world_on_plane(camera, camera_transform, from)?;
    let world_to = cursor_world_on_plane(camera, camera_transform, to)?;
    Some(Vec2::new(world_from.x - world_to.x, world_from.z - world_to.z))
}

pub fn cursor_world_on_plane(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_pos: Vec2,
) -> Option<Vec3> {
    let ray = camera.viewport_to_world(camera_transform, cursor_pos)?;
    if ray.direction.y.abs() < 1e-6 {
        return None;
    }
    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None;
    }
    Some(ray.origin + ray.direction * t)
}
