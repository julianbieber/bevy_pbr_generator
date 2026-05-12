//! Bevy PBR Texture Generator
//!
//! Generates PNG textures for Bevy PBR materials with configurable resolution.
//! Users can edit the texture functions in `textures.rs` to customize generation.

use clap::Parser;
use glam::{Vec2, Vec3, Vec4};
use image::{ImageBuffer, Rgba};
use std::fs;
use std::path::PathBuf;

use image::RgbaImage;

mod noise;
mod textures;

/// CLI arguments for texture generation.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Resolution of the generated textures (width and height).
    #[arg(short, long, default_value_t = 1024)]
    resolution: u32,

    /// Output directory for the generated textures.
    #[arg(short, long, default_value = "./output")]
    output_dir: PathBuf,
}

/// Texture value enum to support different return types from generators.
#[derive(Debug, Clone, Copy)]
enum TextureValue {
    F32(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}

/// Texture configuration for Bevy PBR.
/// Each generator returns its natural type (f32, Vec2, Vec3, Vec4).
/// The pack_fn converts the generator's output to RGBA [f32; 4] based on Bevy's channel mappings.
#[derive(Debug, Clone, Copy)]
struct TextureConfig {
    name: &'static str,
    generator: fn(Vec2) -> TextureValue,
    #[allow(dead_code)]
    is_srgb: bool,
    /// Pack function: converts the generator's TextureValue to RGBA [f32; 4]
    /// based on Bevy's channel mappings for this specific texture.
    pack_fn: fn(TextureValue) -> [f32; 4],
}

impl TextureConfig {
    fn new(
        name: &'static str,
        generator: fn(Vec2) -> TextureValue,
        is_srgb: bool,
        pack_fn: fn(TextureValue) -> [f32; 4],
    ) -> Self {
        Self {
            name,
            generator,
            is_srgb,
            pack_fn,
        }
    }
}

/// Packed texture generator functions.
/// These combine individual PBR properties into optimally packed textures
/// according to BEVY_STANDARD_MATERIAL_TEXTURE_PACKING.md.
mod packed {
    use super::*;

    /// Base color + Opacity (sRGB)
    /// RGB = base_color, A = opacity
    pub fn base_color(uv: Vec2) -> Vec4 {
        textures::base_color_texture(uv)
    }

    /// Normal map (Linear)
    /// RGB = normal vector
    pub fn normal(uv: Vec2) -> Vec3 {
        textures::normal_map_texture(uv)
    }

    /// ORM: Occlusion, Roughness, Metallic (Linear)
    /// R = occlusion, G = roughness, B = metallic
    pub fn orm(uv: Vec2) -> Vec3 {
        let occlusion = textures::occlusion_texture(uv);
        let mr = textures::metallic_roughness_texture(uv);
        Vec3::new(occlusion, mr.x, mr.y) // R=occlusion, G=roughness, B=metallic
    }

    /// Emissive (sRGB)
    /// RGB = emissive color
    pub fn emissive(uv: Vec2) -> Vec3 {
        textures::emissive_texture(uv)
    }

    /// Transmission (Linear)
    /// R = specular_transmission, G = thickness, A = diffuse_transmission
    pub fn transmission(uv: Vec2) -> Vec4 {
        let specular_trans = textures::specular_transmission_texture(uv);
        let thickness = textures::thickness_texture(uv);
        let diffuse_trans = textures::diffuse_transmission_texture(uv);
        Vec4::new(specular_trans, thickness, 0.0, diffuse_trans)
    }

    /// Specular + Specular Tint (Linear for specular, sRGB for tint)
    /// RGB = specular_tint, A = specular
    pub fn specular(uv: Vec2) -> Vec4 {
        let tint = textures::specular_tint_texture(uv);
        let specular = textures::specular_texture(uv);
        Vec4::new(tint.x, tint.y, tint.z, specular)
    }

    /// Clearcoat (Linear)
    /// R = clearcoat, G = clearcoat_roughness
    pub fn clearcoat(uv: Vec2) -> Vec2 {
        let clearcoat = textures::clearcoat_texture(uv);
        let roughness = textures::clearcoat_roughness_texture(uv);
        Vec2::new(clearcoat, roughness)
    }

    /// Clearcoat Normal (Linear)
    /// RGB = clearcoat normal vector
    pub fn clearcoat_normal(uv: Vec2) -> Vec3 {
        textures::clearcoat_normal_texture(uv)
    }

    /// Anisotropy (Linear)
    /// RG = direction, B = strength
    pub fn anisotropy(uv: Vec2) -> Vec3 {
        textures::anisotropy_texture(uv)
    }

    /// Depth / Parallax (Linear)
    /// R = parallax depth
    pub fn depth(uv: Vec2) -> f32 {
        textures::depth_map(uv)
    }
}

fn main() {
    let args = Args::parse();

    // Create output directory if it doesn't exist
    if !args.output_dir.exists() {
        fs::create_dir_all(&args.output_dir).expect("Failed to create output directory");
    }

    // Packed textures following BEVY_STANDARD_MATERIAL_TEXTURE_PACKING.md
    // These combine multiple PBR properties into single textures for optimal packing.
    let textures: Vec<TextureConfig> = vec![
        // Base: RGB = base_color, A = opacity (sRGB)
        TextureConfig::new(
            "base_color",
            |uv| TextureValue::Vec4(packed::base_color(uv)),
            true, // sRGB
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [1.0, 1.0, 1.0, 1.0],
            },
        ),
        // Normal: RGB = normal vector (Linear)
        TextureConfig::new(
            "normal",
            |uv| TextureValue::Vec3(packed::normal(uv)),
            false, // Linear
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.5, 0.5, 1.0, 1.0],
            },
        ),
        // ORM: R = occlusion, G = roughness, B = metallic (Linear)
        TextureConfig::new(
            "orm",
            |uv| TextureValue::Vec3(packed::orm(uv)),
            false, // Linear
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [1.0, 0.5, 0.0, 1.0],
            },
        ),
        // Emissive: RGB = emissive color (sRGB)
        TextureConfig::new(
            "emissive",
            |uv| TextureValue::Vec3(packed::emissive(uv)),
            true, // sRGB
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        // Transmission: R = specular_transmission, G = thickness, B = 0, A = diffuse_transmission (Linear)
        TextureConfig::new(
            "transmission",
            |uv| TextureValue::Vec4(packed::transmission(uv)),
            false, // Linear
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [0.0, 0.0, 0.0, 0.0],
            },
        ),
        // Specular: RGB = specular_tint, A = specular (Linear for alpha channel)
        TextureConfig::new(
            "specular",
            |uv| TextureValue::Vec4(packed::specular(uv)),
            true, // sRGB for tint colors, but specular is linear - using true as tint dominates
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [1.0, 1.0, 1.0, 0.0],
            },
        ),
        // Clearcoat: R = clearcoat, G = clearcoat_roughness (Linear)
        TextureConfig::new(
            "clearcoat",
            |uv| TextureValue::Vec2(packed::clearcoat(uv)),
            false, // Linear
            |v| match v {
                TextureValue::Vec2(v) => [v.x, v.y, 0.0, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        // Clearcoat Normal: RGB = clearcoat normal (Linear)
        TextureConfig::new(
            "clearcoat_normal",
            |uv| TextureValue::Vec3(packed::clearcoat_normal(uv)),
            false, // Linear
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.5, 0.5, 1.0, 1.0],
            },
        ),
        // Anisotropy: R = dir_x, G = dir_y, B = strength (Linear)
        TextureConfig::new(
            "anisotropy",
            |uv| TextureValue::Vec3(packed::anisotropy(uv)),
            false, // Linear
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        // Depth: R = parallax depth (Linear)
        TextureConfig::new(
            "depth",
            |uv| TextureValue::F32(packed::depth(uv)),
            false, // Linear
            |v| match v {
                TextureValue::F32(v) => [v, 0.0, 0.0, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
    ];

    println!(
        "Generating {}x{} textures...",
        args.resolution, args.resolution
    );

    for config in &textures {
        let path = args.output_dir.join(format!("{}.png", config.name));
        generate_texture(config, args.resolution, &path);
        println!("Generated: {}", path.display());
    }

    println!("Done! Textures saved to: {}", args.output_dir.display());
}

/// Generate a single texture and save it as a PNG.
fn generate_texture(config: &TextureConfig, resolution: u32, path: &PathBuf) {
    let width = resolution;
    let height = resolution;

    // Create a new RGBA image buffer
    let mut img: RgbaImage = ImageBuffer::new(width, height);

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
