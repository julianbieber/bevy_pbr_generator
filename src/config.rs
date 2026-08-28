//! The command-line surface, and the description of a single texture to be generated.

use clap::{Parser, ValueEnum};
use glam::{Vec2, Vec3, Vec4};
use std::path::PathBuf;

/// Selects which family of texture functions every generator samples from.
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

/// A sample produced by a texture generator. The variant carries the property's
/// natural arity; it is the matching `pack_fn` that decides which RGBA channels
/// those components land in.
#[derive(Debug, Clone, Copy)]
pub enum TextureValue {
    F32(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}

/// Everything needed to produce one output image: what to call it, what to
/// sample per pixel, and how that sample becomes RGBA.
pub struct TextureConfig {
    /// Stem of the output file name; the `.png` extension is added by the caller.
    pub name: &'static str,
    /// Sampled once per pixel with UV coordinates in `[0.0, 1.0]` on both axes.
    pub generator: Box<dyn Fn(Vec2) -> TextureValue>,
    /// Maps a sample onto RGBA. It must accept every variant `generator` can
    /// return — an unhandled variant silently yields whatever its fallback arm
    /// produces. Components outside `[0.0, 1.0]` are clamped when written.
    pub pack_fn: fn(TextureValue) -> [f32; 4],
}

impl TextureConfig {
    pub fn new(
        name: &'static str,
        generator: impl Fn(Vec2) -> TextureValue + 'static,
        pack_fn: fn(TextureValue) -> [f32; 4],
    ) -> Self {
        Self {
            name,
            generator: Box::new(generator),
            pack_fn,
        }
    }
}
