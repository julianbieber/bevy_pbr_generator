use clap::{Parser, ValueEnum};
use glam::{Vec2, Vec3, Vec4};
use std::path::PathBuf;

/// Texture type for generation.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum TextureType {
    /// Water-like textures (default)
    #[default]
    Water,
    /// Rocky ground textures
    Rocky,
}

/// CLI arguments for texture generation.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Resolution of the generated textures (width and height).
    #[arg(short, long, default_value_t = 1024)]
    pub resolution: u32,

    /// Output directory for the generated textures.
    #[arg(short, long, default_value = "./output")]
    pub output_dir: PathBuf,

    /// Type of texture to generate.
    #[arg(short, long, value_enum, default_value_t = TextureType::Water)]
    pub texture_type: TextureType,
}

/// Texture value enum to support different return types from generators.
#[derive(Debug, Clone, Copy)]
pub enum TextureValue {
    F32(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}

/// Texture configuration for Bevy PBR.
/// Each generator returns its natural type (f32, Vec2, Vec3, Vec4).
/// The pack_fn converts the generator's output to RGBA [f32; 4] based on Bevy's channel mappings.
pub struct TextureConfig {
    pub name: &'static str,
    pub generator: Box<dyn Fn(Vec2) -> TextureValue>,
    #[allow(dead_code)]
    pub is_srgb: bool,
    /// Pack function: converts the generator's TextureValue to RGBA [f32; 4]
    /// based on Bevy's channel mappings for this specific texture.
    pub pack_fn: fn(TextureValue) -> [f32; 4],
}

impl TextureConfig {
    pub fn new(
        name: &'static str,
        generator: impl Fn(Vec2) -> TextureValue + 'static,
        is_srgb: bool,
        pack_fn: fn(TextureValue) -> [f32; 4],
    ) -> Self {
        Self {
            name,
            generator: Box::new(generator),
            is_srgb,
            pack_fn,
        }
    }
}
