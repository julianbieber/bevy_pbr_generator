//! Bevy PBR Texture Generator
//!
//! Generates PNG textures for Bevy PBR materials with configurable resolution.
//! Users can edit the texture functions in `textures.rs` to customize generation.

use std::fs;

use clap::Parser;
use config::{Args, TextureConfig, TextureValue};
use generator::generate_texture;

mod config;
mod generator;
mod noise;
mod packed;
mod textures;

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
