//! Rasterisation: turning a texture description into an image file on disk.

use crate::config::TextureConfig;
use glam::Vec2;
use image::Rgba;
use image::RgbaImage;
use std::path::PathBuf;

/// Samples `config`'s generator over a `resolution` x `resolution` grid of UVs
/// spanning `[0.0, 1.0]` inclusive on both axes, and writes the packed samples
/// to `path` as an 8-bit RGBA PNG. Channel values outside `[0.0, 1.0]` are
/// clamped rather than wrapped.
///
/// `resolution` must be at least 2: at 1 the UV divisor is zero and every
/// sample is taken at NaN. Panics if the file cannot be written, and the
/// encoding is chosen from `path`'s extension, so it must be `.png`.
pub fn generate_texture(config: &TextureConfig, resolution: u32, path: &PathBuf) {
    let width = resolution;
    let height = resolution;

    let mut img: RgbaImage = image::ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let uv = Vec2::new(
                x as f32 / (width - 1) as f32,
                y as f32 / (height - 1) as f32,
            );

            let tex_value = (config.generator)(uv);

            let rgba = (config.pack_fn)(tex_value);

            let r = (rgba[0].clamp(0.0, 1.0) * 255.0) as u8;
            let g = (rgba[1].clamp(0.0, 1.0) * 255.0) as u8;
            let b = (rgba[2].clamp(0.0, 1.0) * 255.0) as u8;
            let a = (rgba[3].clamp(0.0, 1.0) * 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }

    img.save(path).expect("Failed to save texture");
}
