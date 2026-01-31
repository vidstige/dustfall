use bevy::prelude::*;
use bevy::asset::LoadState;
use bevy::log::{Level, LogPlugin};
use bevy::pbr::DirectionalLightShadowMap;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::texture::ImagePlugin;
use bevy::animation::AnimationPlayer;
use bevy::app::PostUpdate;
use bevy::gltf::Gltf;
use bevy::window::PrimaryWindow;
use rand::Rng;
use std::f32::consts::TAU;
use dustfall::solar::{self, Location};

mod heightmap_normal;
mod isometric;
mod texture_atlas;

const GRID_WIDTH: usize = 256;
const GRID_HEIGHT: usize = 256;
// World units are in _meters_
const TILE_SIZE: f32 = 4.0;
const CHUNK_SIZE: usize = 16;
const HEIGHTMAP_PATH: &str = "images/height-map.png";
const ALBEDO_PATH: &str = "images/albedo-map.png";
const HEIGHTMAP_BUMP_SLOPE: f32 = 16.0;
const HEIGHTMAP_BUMP_SCALE: f32 = HEIGHTMAP_BUMP_SLOPE * TILE_SIZE;
const HEIGHTMAP_PATCH_SIZE: usize = 128;
const ASTRONAUT_SCALE: f32 = 0.42;  // Scales to ~1.7m
const ASTRONAUT_WALK_SPEED: f32 = 1.2;
const ASTRONAUT_TURN_SPEED: f32 = 4.0;
const ASTRONAUT_STOP_DISTANCE: f32 = 0.05;
// The astronaut model's forward axis points to +X, so we rotate by -90deg to align with +Z.
const ASTRONAUT_FORWARD_YAW_OFFSET: f32 = -std::f32::consts::FRAC_PI_2;
const DEFAULT_LOCATION: Location = Location {
    latitude: 22.5 * (TAU / 360.0),
    longitude: 137.4 * (TAU / 360.0),
};

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    Loading,
    Running,
}

#[derive(Resource)]
struct TileMap {
    width: usize,
    height: usize,
    tiles: Vec<u32>,
}

#[derive(Resource)]
struct GameAssets {
    heightmap: Handle<Image>,
    albedo: Handle<Image>,
    astronaut_gltf: Handle<Gltf>,
    astronaut_scene: Handle<Scene>,
}

#[derive(Resource)]
#[allow(dead_code)]
struct TerrainAssets {
    atlas: texture_atlas::TextureAtlas,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Astronaut;

#[derive(Component)]
struct LoadingIndicator {
    base_scale: f32,
}

#[derive(Component)]
struct AstronautController {
    target: Vec3,
    speed: f32,
    turn_speed: f32,
    moving: bool,
}

#[derive(Resource, Default)]
struct AstronautAnimations {
    clips: Option<AstronautAnimationClips>,
}

struct AstronautAnimationClips {
    idle: Handle<AnimationClip>,
    walk: Handle<AnimationClip>,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.08)))
        .insert_resource(AstronautAnimations::default())
        .add_state::<AppState>()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                // Bevy doesn't support Step interpolation for glTF animations.
                // Re-export the GLB with Linear interpolation to remove this filter.
                .set(LogPlugin {
                    level: Level::INFO,
                    filter: "wgpu=error,naga=warn,bevy_gltf::loader=error".to_string(),
                }),
        )
        .insert_resource(random_map(GRID_WIDTH, GRID_HEIGHT))
        .add_systems(Startup, (isometric::spawn_iso_camera, load_assets))
        .add_systems(
            OnEnter(AppState::Loading),
            spawn_loading_indicator,
        )
        .add_systems(OnExit(AppState::Loading), despawn_loading_indicator)
        .add_systems(
            Update,
            (
                check_loading_ready,
                animate_loading_indicator,
            )
                .run_if(in_state(AppState::Loading)),
        )
        .add_systems(
            OnEnter(AppState::Running),
            (setup_lighting, spawn_tiles, setup_astronaut),
        )
        .add_systems(
            Update,
            (
                isometric::update_iso_camera,
                init_scene_animations,
                update_sun_light,
                (update_astronaut_movement, update_astronaut_animation_state).chain(),
            )
                .run_if(in_state(AppState::Running)),
        )
        .add_systems(
            PostUpdate,
            remove_cameras::<Astronaut>.run_if(in_state(AppState::Running)),
        )
        .run();
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let heightmap = asset_server.load(HEIGHTMAP_PATH);
    let albedo = asset_server.load(ALBEDO_PATH);
    let astronaut_gltf = asset_server.load("models/astronaut/astronaut-textured.glb");
    let astronaut_scene = asset_server.load("models/astronaut/astronaut-textured.glb#Scene0");
    commands.insert_resource(GameAssets {
        heightmap,
        albedo,
        astronaut_gltf,
        astronaut_scene,
    });
}

fn spawn_loading_indicator(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.5,
        sectors: 32,
        stacks: 16,
    }));
    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.9, 0.9, 0.95),
        emissive: Color::rgb(0.15, 0.15, 0.2),
        unlit: true,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
            ..default()
        },
        LoadingIndicator { base_scale: 1.0 },
    ));
}

fn despawn_loading_indicator(
    mut commands: Commands,
    indicators: Query<Entity, With<LoadingIndicator>>,
) {
    for entity in &indicators {
        commands.entity(entity).despawn_recursive();
    }
}

fn animate_loading_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut Transform, &LoadingIndicator)>,
) {
    let pulse = 1.0 + (time.elapsed_seconds() * 3.0).sin() * 0.15;
    for (mut transform, indicator) in &mut indicators {
        transform.scale = Vec3::splat(indicator.base_scale * pulse);
    }
}

fn check_loading_ready(
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let heightmap_loaded =
        asset_server.get_load_state(&assets.heightmap) == LoadState::Loaded;
    let albedo_loaded =
        asset_server.get_load_state(&assets.albedo) == LoadState::Loaded;
    let astronaut_gltf_loaded =
        asset_server.get_load_state(&assets.astronaut_gltf) == LoadState::Loaded;
    let astronaut_scene_loaded =
        asset_server.get_load_state(&assets.astronaut_scene) == LoadState::Loaded;

    if heightmap_loaded && albedo_loaded && astronaut_gltf_loaded && astronaut_scene_loaded {
        next_state.set(AppState::Running);
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.9, 0.9, 1.0),
        brightness: 0.1,
    });
    commands.insert_resource(DirectionalLightShadowMap { size: 2048 });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 18000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 1.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.9,
            -0.6,
            0.0,
        )),
        ..default()
    });
}

fn update_sun_light(
    time: Res<Time>,
    mut lights: Query<&mut Transform, With<DirectionalLight>>,
) {
    let time_seconds = time.elapsed_seconds();
    let (x, y, z) = solar::solar_direction(&solar::MARS, DEFAULT_LOCATION, time_seconds);
    let sun_dir = Vec3::new(x, y, z);
    let light_dir = -sun_dir.normalize_or_zero();
    let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, light_dir);
    for mut transform in &mut lights {
        transform.rotation = rotation;
    }
}

fn setup_astronaut(mut commands: Commands, assets: Res<GameAssets>) {
    let spawn_translation = Vec3::new(2.0, 0.0, 0.5);
    commands.spawn((
        SceneBundle {
            scene: assets.astronaut_scene.clone(),
            transform: Transform {
                translation: spawn_translation,
                scale: Vec3::splat(ASTRONAUT_SCALE),
                ..default()
            },
            ..default()
        },
        Astronaut,
        AstronautController {
            target: spawn_translation,
            speed: ASTRONAUT_WALK_SPEED,
            turn_speed: ASTRONAUT_TURN_SPEED,
            moving: false,
        },
    ));
}

fn spawn_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    map: Res<TileMap>,
    assets: Res<GameAssets>,
) {
    let heightmap_image = images
        .get(&assets.heightmap)
        .expect("heightmap image not loaded")
        .clone();
    let albedo_image = images
        .get(&assets.albedo)
        .expect("albedo image not loaded")
        .clone();
    assert_eq!(
        heightmap_image.texture_descriptor.size,
        albedo_image.texture_descriptor.size,
        "albedo map must match heightmap dimensions"
    );

    let normal_map = heightmap_normal::build_heightmap_normal_map(
        &heightmap_image,
        HEIGHTMAP_BUMP_SCALE,
        TILE_SIZE,
    );
    let normal_handle = images.add(normal_map);
    let atlas = texture_atlas::TextureAtlas::from_image(
        &heightmap_image,
        HEIGHTMAP_PATCH_SIZE,
        normal_handle,
    );
    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        base_color_texture: Some(assets.albedo.clone()),
        normal_map_texture: Some(atlas.handle.clone()),
        perceptual_roughness: 0.9,
        cull_mode: None,
        ..default()
    });

    for mut mesh in build_grid_meshes(&map, &atlas) {
        let _ = mesh.generate_tangents();
        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: material.clone(),
            ..default()
        });
    }

    commands.insert_resource(TerrainAssets { atlas, material });
}

fn init_scene_animations(
    assets: Res<GameAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut astronaut_animations: ResMut<AstronautAnimations>,
    astronaut_roots: Query<Entity, With<Astronaut>>,
    parents: Query<&Parent>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    let astronaut_entities: Vec<Entity> = astronaut_roots.iter().collect();
    if astronaut_entities.is_empty() {
        return;
    }

    let Some(gltf) = gltfs.get(&assets.astronaut_gltf) else {
        return;
    };
    if astronaut_animations.clips.is_none() {
        let Some(idle) = gltf.named_animations.get("Idle_Breath").cloned() else {
            return;
        };
        let Some(walk) = gltf.named_animations.get("Walk_Loop").cloned() else {
            return;
        };
        astronaut_animations.clips = Some(AstronautAnimationClips {
            idle,
            walk,
        });
    }
    let Some(clips) = astronaut_animations.clips.as_ref() else {
        return;
    };

    for (entity, mut player) in &mut players {
        if is_descendant_of(entity, &astronaut_entities, &parents) {
            player.play(clips.idle.clone());
            player.repeat();
            player.set_speed(1.0);
            player.resume();
        }
    }
}

fn remove_cameras<T: Component>(
    component: Query<Entity, With<T>>,
    parents: Query<&Parent>,
    cameras: Query<Entity, Added<Camera>>,
    mut commands: Commands,
) {
    let roots: Vec<Entity> = component.iter().collect();
    for entity in &cameras {
        if is_descendant_of(entity, &roots, &parents) {
            commands.entity(entity).remove::<Camera>();
            commands.entity(entity).remove::<Camera3d>();
        }
    }
}

fn is_descendant_of(
    mut entity: Entity,
    roots: &[Entity],
    parents: &Query<&Parent>,
) -> bool {
    loop {
        if roots.contains(&entity) {
            return true;
        }
        let Ok(parent) = parents.get(entity) else {
            return false;
        };
        entity = parent.get();
    }
}

fn build_grid_meshes(map: &TileMap, atlas: &texture_atlas::TextureAtlas) -> Vec<Mesh> {
    assert!(
        map.width % CHUNK_SIZE == 0 && map.height % CHUNK_SIZE == 0,
        "map dimensions must be divisible by chunk size"
    );

    let chunks_x = map.width / CHUNK_SIZE;
    let chunks_y = map.height / CHUNK_SIZE;
    let half_w = map.width as f32 * TILE_SIZE * 0.5;
    let half_h = map.height as f32 * TILE_SIZE * 0.5;

    let mut meshes = Vec::with_capacity(chunks_x * chunks_y);
    for chunk_y in 0..chunks_y {
        for chunk_x in 0..chunks_x {
            let mesh = build_chunk_mesh(map, atlas, chunk_x, chunk_y, half_w, half_h);
            meshes.push(mesh);
        }
    }

    meshes
}

fn build_chunk_mesh(
    map: &TileMap,
    atlas: &texture_atlas::TextureAtlas,
    chunk_x: usize,
    chunk_y: usize,
    half_w: f32,
    half_h: f32,
) -> Mesh {
    let mut positions = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * 4);
    let mut normals = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * 4);
    let mut uvs = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * 4);
    let mut indices = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * 6);

    let tile_x_start = chunk_x * CHUNK_SIZE;
    let tile_y_start = chunk_y * CHUNK_SIZE;

    for local_y in 0..CHUNK_SIZE {
        for local_x in 0..CHUNK_SIZE {
            let tile_x = tile_x_start + local_x;
            let tile_y = tile_y_start + local_y;
            let world_x = tile_x as f32 * TILE_SIZE - half_w;
            let world_z = tile_y as f32 * TILE_SIZE - half_h;
            let tile_index = map.tile_index(tile_x, tile_y) as usize;
            let (uv_min, uv_max) = atlas.uv_bounds(tile_index);

            push_tile(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                world_x,
                world_z,
                TILE_SIZE,
                uv_min,
                uv_max,
            );
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}

impl TileMap {
    fn tile_index(&self, x: usize, y: usize) -> u32 {
        self.tiles[y * self.width + x]
    }
}

fn random_map(width: usize, height: usize) -> TileMap {
    let mut tiles = Vec::with_capacity(width * height);
    let mut rng = rand::thread_rng();
    for _y in 0..height {
        for _x in 0..width {
            tiles.push(rng.gen::<u32>());
        }
    }

    TileMap {
        width,
        height,
        tiles,
    }
}

fn push_tile(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
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
    let normal = [0.0, 1.0, 0.0];

    let base = positions.len() as u32;
    positions.extend_from_slice(&[
        [x0, y, z0],
        [x1, y, z0],
        [x1, y, z1],
        [x0, y, z1],
    ]);
    normals.extend_from_slice(&[normal; 4]);
    uvs.extend_from_slice(&[
        [uv_min.x, uv_min.y],
        [uv_max.x, uv_min.y],
        [uv_max.x, uv_max.y],
        [uv_min.x, uv_max.y],
    ]);

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn update_astronaut_movement(
    time: Res<Time>,
    mouse_buttons: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<isometric::IsoCameraTag>>,
    mut astronauts: Query<(&mut Transform, &mut AstronautController), With<Astronaut>>,
) {
    if mouse_buttons.pressed(MouseButton::Left) {
        let window = windows.get_single().ok();
        let cursor_pos = window.and_then(|window| window.cursor_position());
        let camera = camera_query.get_single().ok();
        if let (Some(cursor_pos), Some((camera, camera_transform))) = (cursor_pos, camera) {
            if let Some(world_pos) =
                isometric::cursor_world_on_plane(camera, camera_transform, cursor_pos)
            {
                for (transform, mut controller) in &mut astronauts {
                    controller.target = Vec3::new(
                        world_pos.x,
                        transform.translation.y,
                        world_pos.z,
                    );
                }
            }
        }
    }

    let dt = time.delta_seconds();
    for (mut transform, mut controller) in &mut astronauts {
        let mut to_target = controller.target - transform.translation;
        to_target.y = 0.0;
        let distance = to_target.length();
        if distance <= ASTRONAUT_STOP_DISTANCE || dt <= 0.0 {
            controller.moving = false;
            continue;
        }

        let dir = to_target / distance;
        let target_rot =
            Quat::from_rotation_y(dir.x.atan2(dir.z) + ASTRONAUT_FORWARD_YAW_OFFSET);
        let turn_t = (controller.turn_speed * dt).clamp(0.0, 1.0);
        transform.rotation = transform.rotation.slerp(target_rot, turn_t);

        let travel = (controller.speed * dt).min(distance);
        transform.translation += dir * travel;

        let remaining = distance - travel;
        if remaining <= ASTRONAUT_STOP_DISTANCE {
            transform.translation.x = controller.target.x;
            transform.translation.z = controller.target.z;
            controller.moving = false;
        } else {
            controller.moving = true;
        }
    }
}

fn update_astronaut_animation_state(
    animations: Res<AstronautAnimations>,
    astronauts: Query<(Entity, &AstronautController), With<Astronaut>>,
    parents: Query<&Parent>,
    mut players: Query<(Entity, &mut AnimationPlayer)>,
) {
    let Some(clips) = animations.clips.as_ref() else {
        return;
    };

    for (astronaut_entity, controller) in &astronauts {
        let desired = if controller.moving {
            clips.walk.clone()
        } else {
            clips.idle.clone()
        };
        let roots = [astronaut_entity];
        for (player_entity, mut player) in &mut players {
            if is_descendant_of(player_entity, &roots, &parents) {
                player.play(desired.clone());
                player.repeat();
                player.resume();
            }
        }
    }
}
