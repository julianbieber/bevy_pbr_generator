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

    if !args.output_dir.exists() {
        fs::create_dir_all(&args.output_dir).expect("Failed to create output directory");
    }

    let textures: Vec<TextureConfig> = vec![
        TextureConfig::new(
            "base_color",
            |uv| TextureValue::Vec4(packed::base_color(uv)),
            true,
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [1.0, 1.0, 1.0, 1.0],
            },
        ),
        TextureConfig::new(
            "normal",
            |uv| TextureValue::Vec3(packed::normal(uv)),
            false,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.5, 0.5, 1.0, 1.0],
            },
        ),
        TextureConfig::new(
            "orm",
            |uv| TextureValue::Vec3(packed::orm(uv)),
            false,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [1.0, 0.5, 0.0, 1.0],
            },
        ),
        TextureConfig::new(
            "emissive",
            |uv| TextureValue::Vec3(packed::emissive(uv)),
            true,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        TextureConfig::new(
            "transmission",
            |uv| TextureValue::Vec4(packed::transmission(uv)),
            false,
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [0.0, 0.0, 0.0, 0.0],
            },
        ),
        TextureConfig::new(
            "specular",
            |uv| TextureValue::Vec4(packed::specular(uv)),
            true,
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [1.0, 1.0, 1.0, 0.0],
            },
        ),
        TextureConfig::new(
            "clearcoat",
            |uv| TextureValue::Vec2(packed::clearcoat(uv)),
            false,
            |v| match v {
                TextureValue::Vec2(v) => [v.x, v.y, 0.0, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        TextureConfig::new(
            "clearcoat_normal",
            |uv| TextureValue::Vec3(packed::clearcoat_normal(uv)),
            false,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.5, 0.5, 1.0, 1.0],
            },
        ),
        TextureConfig::new(
            "anisotropy",
            |uv| TextureValue::Vec3(packed::anisotropy(uv)),
            false,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        TextureConfig::new(
            "depth",
            |uv| TextureValue::F32(packed::depth(uv)),
            false,
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
