//! Bevy PBR Texture Generator
//!
//! Generates PNG textures for Bevy PBR materials with configurable resolution.
//! Users can edit the texture functions in `textures.rs` to customize generation.

use std::fs;

use clap::Parser;
use config::{Args, TextureConfig, TextureType, TextureValue};
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

    let texture_type = args.texture_type;

    let textures: Vec<TextureConfig> = vec![
        TextureConfig::new(
            "base_color",
            move |uv| TextureValue::Vec4(packed::base_color(uv, texture_type)),
            true,
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [1.0, 1.0, 1.0, 1.0],
            },
        ),
        TextureConfig::new(
            "normal",
            move |uv| TextureValue::Vec3(packed::normal(uv, texture_type)),
            false,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.5, 0.5, 1.0, 1.0],
            },
        ),
        TextureConfig::new(
            "orm",
            move |uv| TextureValue::Vec3(packed::orm(uv, texture_type)),
            false,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [1.0, 0.5, 0.0, 1.0],
            },
        ),
        TextureConfig::new(
            "emissive",
            move |uv| TextureValue::Vec3(packed::emissive(uv, texture_type)),
            true,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        TextureConfig::new(
            "transmission",
            move |uv| TextureValue::Vec4(packed::transmission(uv, texture_type)),
            false,
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [0.0, 0.0, 0.0, 0.0],
            },
        ),
        TextureConfig::new(
            "specular",
            move |uv| TextureValue::Vec4(packed::specular(uv, texture_type)),
            true,
            |v| match v {
                TextureValue::Vec4(v) => [v.x, v.y, v.z, v.w],
                _ => [1.0, 1.0, 1.0, 0.0],
            },
        ),
        TextureConfig::new(
            "clearcoat",
            move |uv| TextureValue::Vec2(packed::clearcoat(uv, texture_type)),
            false,
            |v| match v {
                TextureValue::Vec2(v) => [v.x, v.y, 0.0, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        TextureConfig::new(
            "clearcoat_normal",
            move |uv| TextureValue::Vec3(packed::clearcoat_normal(uv, texture_type)),
            false,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.5, 0.5, 1.0, 1.0],
            },
        ),
        TextureConfig::new(
            "anisotropy",
            move |uv| TextureValue::Vec3(packed::anisotropy(uv, texture_type)),
            false,
            |v| match v {
                TextureValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
        TextureConfig::new(
            "depth",
            move |uv| TextureValue::F32(packed::depth(uv, texture_type)),
            false,
            |v| match v {
                TextureValue::F32(v) => [v, 0.0, 0.0, 1.0],
                _ => [0.0, 0.0, 0.0, 1.0],
            },
        ),
    ];

    let texture_name = match texture_type {
        TextureType::Water => "water",
        TextureType::Rocky => "rocky ground",
    };

    println!(
        "Generating {}x{} {} textures...",
        args.resolution, args.resolution, texture_name
    );

    for config in &textures {
        let path = args.output_dir.join(format!("{}.png", config.name));
        generate_texture(config, args.resolution, &path);
        println!("Generated: {}", path.display());
    }

    println!("Done! Textures saved to: {}", args.output_dir.display());
}
