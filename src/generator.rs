//! Texture generation utilities

use crate::config::TextureConfig;
use glam::Vec2;
use image::Rgba;
use image::RgbaImage;
use std::path::PathBuf;

/// Generate a single texture and save it as a PNG.
pub fn generate_texture(config: &TextureConfig, resolution: u32, path: &PathBuf) {
    let width = resolution;
    let height = resolution;

    // Create a new RGBA image buffer
    let mut img: RgbaImage = image::ImageBuffer::new(width, height);

    // Generate pixels
    for y in 0..height {
        for x in 0..width {
            // Normalized UV coordinates [0.0, 1.0]
            let uv = Vec2::new(
                x as f32 / (width - 1) as f32,
                y as f32 / (height - 1) as f32,
            );

            // Generate texture value (wrapped in TextureValue enum)
            let tex_value = (config.generator)(uv);

            // Pack to RGBA channels using the texture-specific pack function
            let rgba = (config.pack_fn)(tex_value);

            // Convert to u8 (0-255)
            let r = (rgba[0].clamp(0.0, 1.0) * 255.0) as u8;
            let g = (rgba[1].clamp(0.0, 1.0) * 255.0) as u8;
            let b = (rgba[2].clamp(0.0, 1.0) * 255.0) as u8;
            let a = (rgba[3].clamp(0.0, 1.0) * 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }

    // Save as PNG
    // Note: The `image` crate doesn't support ICC profiles, so we rely on:
    // - sRGB textures: saved as-is (PNG is sRGB by default)
    // - Linear textures: saved as-is (user must load with `is_srgb: false` in Bevy)
    img.save(path).expect("Failed to save texture");
}
