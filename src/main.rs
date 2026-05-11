//! Bevy PBR Texture Generator
//!
//! Generates PNG textures for Bevy PBR materials with configurable resolution.
//! Users can edit the texture functions in `textures.rs` to customize generation.

use clap::Parser;
use glam::Vec2;
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

/// Texture configuration for Bevy PBR.
#[derive(Debug, Clone)]
struct TextureConfig {
    name: &'static str,
    generator: fn(Vec2) -> Vec2,
    #[allow(dead_code)]
    is_srgb: bool,
    /// Channel mapping: (R, G, B, A) where each is derived from the Vec2 output.
    /// For Vec2 output, we map x and y to specific channels.
    channel_map: fn(Vec2) -> [f32; 4],
}

impl TextureConfig {
    fn new(
        name: &'static str,
        generator: fn(Vec2) -> Vec2,
        is_srgb: bool,
        channel_map: fn(Vec2) -> [f32; 4],
    ) -> Self {
        Self {
            name,
            generator,
            is_srgb,
            channel_map,
        }
    }
}

fn main() {
    let args = Args::parse();

    // Create output directory if it doesn't exist
    if !args.output_dir.exists() {
        fs::create_dir_all(&args.output_dir).expect("Failed to create output directory");
    }

    // Define all textures with their configurations
    // Note: Bevy PBR texture channel mappings:
    // - base_color_texture: RGB = color, A = opacity
    // - emissive_texture: RGB = emissive color
    // - metallic_roughness_texture: B = metallic, G = roughness
    // - diffuse_transmission_texture: A = transmission
    // - specular_transmission_texture: R = transmission
    // - thickness_texture: G = thickness
    let textures: Vec<TextureConfig> = vec![
        // Base color: RGB = color, A = 1.0 (opaque)
        // Vec2.x -> R, Vec2.y -> G, B = 0.0, A = 1.0
        TextureConfig::new(
            "base_color_texture",
            textures::base_color_texture,
            true, // sRGB
            |v| [v.x, v.y, 0.0, 1.0],
        ),
        // Emissive: RGB = emissive color, A = 1.0
        // Vec2.x -> R, Vec2.y -> G, B = 0.0, A = 1.0
        TextureConfig::new(
            "emissive_texture",
            textures::emissive_texture,
            true, // sRGB
            |v| [v.x, v.y, 0.0, 1.0],
        ),
        // Metallic-roughness: B = metallic, G = roughness, R = 0.0, A = 1.0
        // Vec2.x -> roughness (G), Vec2.y -> metallic (B)
        TextureConfig::new(
            "metallic_roughness_texture",
            textures::metallic_roughness_texture,
            false,                    // Linear
            |v| [0.0, v.x, v.y, 1.0], // R=0.0, G=roughness, B=metallic, A=1.0
        ),
        // Diffuse transmission: A = transmission, RGB = 1.0
        // Vec2.x -> transmission (A)
        TextureConfig::new(
            "diffuse_transmission_texture",
            textures::diffuse_transmission_texture,
            false,                    // Linear
            |v| [1.0, 1.0, 1.0, v.x], // R=1.0, G=1.0, B=1.0, A=transmission
        ),
        // Specular transmission: R = transmission, GBA = 1.0
        // Vec2.x -> transmission (R)
        TextureConfig::new(
            "specular_transmission_texture",
            textures::specular_transmission_texture,
            false,                    // Linear
            |v| [v.x, 1.0, 1.0, 1.0], // R=transmission, G=1.0, B=1.0, A=1.0
        ),
        // Thickness: G = thickness, RBA = 1.0
        // Vec2.x -> thickness (G)
        TextureConfig::new(
            "thickness_texture",
            textures::thickness_texture,
            false,                    // Linear
            |v| [1.0, v.x, 1.0, 1.0], // R=1.0, G=thickness, B=1.0, A=1.0
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

            // Generate texture value
            let tex_value = (config.generator)(uv);

            // Map to RGBA channels
            let rgba = (config.channel_map)(tex_value);

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
