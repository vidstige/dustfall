use bevy::prelude::*;
use bevy::render::render_resource::{
    AddressMode, Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat,
};
use bevy::render::texture::{ImageSampler, TextureFormatPixelInfo};

pub fn build_heightmap_normal_map(
    image: &Image,
    bump_scale: f32,
    world_scale: f32,
) -> Image {
    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;
    let pixel_stride = image.texture_descriptor.format.pixel_size();
    let heightmap_data = &image.data;
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
            heights.push(luma * bump_scale);
        }
    }

    let mut normal_data = Vec::with_capacity(width * height * 8);
    for y in 0..height {
        for x in 0..width {
            let normal = heightmap_normal(&heights, width, height, x, y, world_scale);
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
    world_scale: f32,
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
        (h_r - h_l) / ((x1 - x0) as f32 * world_scale)
    };
    let dz = if y0 == y1 {
        0.0
    } else {
        (h_u - h_d) / ((y1 - y0) as f32 * world_scale)
    };

    Vec3::new(-dx, 1.0, -dz).normalize()
}

fn normal_channel_u16(value: f32) -> u16 {
    let clamped = value.clamp(-1.0, 1.0);
    ((clamped * 0.5 + 0.5) * 65535.0).round() as u16
}
