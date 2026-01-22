use bevy::prelude::*;
use bevy::log::{Level, LogPlugin};
use bevy::pbr::DirectionalLightShadowMap;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_resource::{
    AddressMode, Extent3d, FilterMode, PrimitiveTopology, SamplerDescriptor, TextureDimension,
    TextureFormat,
};
use bevy::render::texture::{ImagePlugin, ImageSampler, TextureFormatPixelInfo};
use bevy::animation::AnimationPlayer;
use bevy::app::PostUpdate;
use bevy::gltf::Gltf;

mod isometric;

const GRID_WIDTH: usize = 256;
const GRID_HEIGHT: usize = 256;
const TILE_WORLD_SIZE: f32 = 0.5;
const CHUNK_SIZE: usize = 16;
const HEIGHTMAP_PATH: &str = "images/height-map.png";
const HEIGHTMAP_BUMP_SCALE: f32 = 8.0;
const HEIGHTMAP_PATCH_SIZE: usize = 32;

#[derive(Resource)]
struct TileMap {
    width: usize,
    height: usize,
    tiles: Vec<u32>,
}

#[derive(Resource)]
struct HeightmapHandle(Handle<Image>);

#[derive(Resource)]
struct AstronautAssets {
    gltf: Handle<Gltf>,
}

#[derive(Component)]
struct Astronaut;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.08)))
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
        .insert_resource(checker_board(GRID_WIDTH, GRID_HEIGHT))
        .add_systems(
            Startup,
            (
                isometric::spawn_iso_camera,
                setup_heightmap,
                setup_lighting,
                setup_astronaut,
            ),
        )
        .add_systems(
            Update,
            (
                isometric::update_iso_camera,
                spawn_tiles_when_ready,
                init_scene_animations,
            ),
        )
        .add_systems(PostUpdate, remove_astronaut_cameras)
        .run();
}

fn setup_heightmap(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load(HEIGHTMAP_PATH);
    commands.insert_resource(HeightmapHandle(handle));
}

fn setup_lighting(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.9, 0.9, 1.0),
        brightness: 0.35,
    });
    commands.insert_resource(DirectionalLightShadowMap { size: 2048 });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: true,
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

fn setup_astronaut(mut commands: Commands, asset_server: Res<AssetServer>) {
    let gltf = asset_server.load("models/astronaut/astronaut-textured.glb");
    let scene = asset_server.load("models/astronaut/astronaut-textured.glb#Scene0");
    commands.insert_resource(AstronautAssets { gltf });
    commands.spawn((
        SceneBundle {
            scene,
            transform: Transform {
                translation: Vec3::new(2.0, 0.0, 0.5),
                scale: Vec3::splat(0.5),
                ..default()
            },
            ..default()
        },
        Astronaut,
    ));
}

fn spawn_tiles_when_ready(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    map: Res<TileMap>,
    heightmap_handle: Res<HeightmapHandle>,
    mut spawned: Local<bool>,
) {
    if *spawned {
        return;
    }

    let (heightmap_width, heightmap_height, pixel_stride, heightmap_data) = {
        let image = match images.get(&heightmap_handle.0) {
            Some(image) => image,
            None => return,
        };
        let pixel_stride = image.texture_descriptor.format.pixel_size();
        (
            image.texture_descriptor.size.width as usize,
            image.texture_descriptor.size.height as usize,
            pixel_stride,
            image.data.clone(),
        )
    };

    let normal_map = build_heightmap_normal_map(
        heightmap_width,
        heightmap_height,
        pixel_stride,
        &heightmap_data,
    );
    let normal_handle = images.add(normal_map);
    assert!(HEIGHTMAP_PATCH_SIZE > 0, "heightmap patch size must be non-zero");
    assert!(
        heightmap_width % HEIGHTMAP_PATCH_SIZE == 0
            && heightmap_height % HEIGHTMAP_PATCH_SIZE == 0,
        "heightmap size must be divisible by patch size"
    );
    let tiles_per_row = heightmap_width / HEIGHTMAP_PATCH_SIZE;
    let tiles_per_col = heightmap_height / HEIGHTMAP_PATCH_SIZE;
    let uv_patch = Vec2::new(
        HEIGHTMAP_PATCH_SIZE as f32 / heightmap_width as f32,
        HEIGHTMAP_PATCH_SIZE as f32 / heightmap_height as f32,
    );
    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.62, 0.6, 0.56),
        normal_map_texture: Some(normal_handle),
        perceptual_roughness: 0.9,
        cull_mode: None,
        ..default()
    });

    for mut mesh in build_grid_meshes(&map, tiles_per_row, tiles_per_col, uv_patch) {
        let _ = mesh.generate_tangents();
        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: material.clone(),
            ..default()
        });
    }

    *spawned = true;
}

fn init_scene_animations(
    astronaut_assets: Res<AstronautAssets>,
    gltfs: Res<Assets<Gltf>>,
    astronaut_roots: Query<Entity, With<Astronaut>>,
    parents: Query<&Parent>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    let astronaut_entities: Vec<Entity> = astronaut_roots.iter().collect();
    if astronaut_entities.is_empty() {
        return;
    }

    let Some(gltf) = gltfs.get(&astronaut_assets.gltf) else {
        return;
    };
    let animation = gltf
        .named_animations
        .get("Idle_Breath")
        .cloned()
        .or_else(|| gltf.animations.first().cloned());
    let Some(animation) = animation else {
        return;
    };

    for (entity, mut player) in &mut players {
        if is_descendant_of(entity, &astronaut_entities, &parents) {
            player.play(animation.clone());
            player.repeat();
            player.set_speed(1.0);
            player.resume();
        }
    }
}

fn remove_astronaut_cameras(
    astronaut_roots: Query<Entity, With<Astronaut>>,
    parents: Query<&Parent>,
    cameras: Query<Entity, Added<Camera>>,
    mut commands: Commands,
) {
    let roots: Vec<Entity> = astronaut_roots.iter().collect();
    if roots.is_empty() {
        return;
    }

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

fn build_grid_meshes(
    map: &TileMap,
    tiles_per_row: usize,
    tiles_per_col: usize,
    uv_patch: Vec2,
) -> Vec<Mesh> {
    assert!(
        map.width % CHUNK_SIZE == 0 && map.height % CHUNK_SIZE == 0,
        "map dimensions must be divisible by chunk size"
    );
    assert!(tiles_per_row > 0 && tiles_per_col > 0, "heightmap atlas is empty");

    let chunks_x = map.width / CHUNK_SIZE;
    let chunks_y = map.height / CHUNK_SIZE;
    let half_w = map.width as f32 * TILE_WORLD_SIZE * 0.5;
    let half_h = map.height as f32 * TILE_WORLD_SIZE * 0.5;

    let mut meshes = Vec::with_capacity(chunks_x * chunks_y);
    for chunk_y in 0..chunks_y {
        for chunk_x in 0..chunks_x {
            meshes.push(build_chunk_mesh(
                map,
                tiles_per_row,
                tiles_per_col,
                chunk_x,
                chunk_y,
                half_w,
                half_h,
                uv_patch,
            ));
        }
    }

    meshes
}

fn build_chunk_mesh(
    map: &TileMap,
    tiles_per_row: usize,
    tiles_per_col: usize,
    chunk_x: usize,
    chunk_y: usize,
    half_w: f32,
    half_h: f32,
    uv_patch: Vec2,
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
            let world_x = tile_x as f32 * TILE_WORLD_SIZE - half_w;
            let world_z = tile_y as f32 * TILE_WORLD_SIZE - half_h;
            let tile_index = map.tile_index(tile_x, tile_y) as usize;
            let patch_x = tile_index % tiles_per_row;
            let patch_y = (tile_index / tiles_per_row) % tiles_per_col;
            let uv_min = Vec2::new(
                patch_x as f32 * uv_patch.x,
                patch_y as f32 * uv_patch.y,
            );
            let uv_max = uv_min + uv_patch;

            push_tile(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                world_x,
                world_z,
                TILE_WORLD_SIZE,
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

fn build_heightmap_normal_map(
    width: usize,
    height: usize,
    pixel_stride: usize,
    heightmap_data: &[u8],
) -> Image {
    assert!(pixel_stride >= 1, "heightmap texture must be uncompressed");
    assert!(
        heightmap_data.len() >= width * height * pixel_stride,
        "heightmap data does not match image dimensions"
    );

    let mut heights = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) * pixel_stride;
            let r = heightmap_data[index] as f32 / 255.0;
            let g = if pixel_stride > 1 {
                heightmap_data[index + 1] as f32 / 255.0
            } else {
                r
            };
            let b = if pixel_stride > 2 {
                heightmap_data[index + 2] as f32 / 255.0
            } else {
                r
            };
            let luma = (r + g + b) / 3.0;
            heights.push(luma * HEIGHTMAP_BUMP_SCALE);
        }
    }

    let mut normal_data = Vec::with_capacity(width * height * 8);
    for y in 0..height {
        for x in 0..width {
            let normal = heightmap_normal(&heights, width, height, x, y);
            normal_data.extend_from_slice(&normal_channel_u16(normal.x).to_le_bytes());
            normal_data.extend_from_slice(&normal_channel_u16(normal.y).to_le_bytes());
            normal_data.extend_from_slice(&normal_channel_u16(normal.z).to_le_bytes());
            normal_data.extend_from_slice(&u16::MAX.to_le_bytes());
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        normal_data,
        TextureFormat::Rgba16Unorm,
    );
    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..default()
    });
    image
}

fn heightmap_normal(
    heights: &[f32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
) -> Vec3 {
    if width == 0 || height == 0 {
        return Vec3::Y;
    }

    let x0 = x.saturating_sub(1);
    let x1 = (x + 1).min(width - 1);
    let y0 = y.saturating_sub(1);
    let y1 = (y + 1).min(height - 1);

    let h_l = heights[y * width + x0];
    let h_r = heights[y * width + x1];
    let h_d = heights[y0 * width + x];
    let h_u = heights[y1 * width + x];

    let dx = if x0 == x1 {
        0.0
    } else {
        (h_r - h_l) / ((x1 - x0) as f32 * TILE_WORLD_SIZE)
    };
    let dz = if y0 == y1 {
        0.0
    } else {
        (h_u - h_d) / ((y1 - y0) as f32 * TILE_WORLD_SIZE)
    };

    Vec3::new(-dx, 1.0, -dz).normalize()
}

fn normal_channel_u16(value: f32) -> u16 {
    let clamped = value.clamp(-1.0, 1.0);
    ((clamped * 0.5 + 0.5) * 65535.0).round() as u16
}
