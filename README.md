# Bevy PBR Texture Generator

A command-line tool for generating PBR (Physically Based Rendering) textures compatible with [Bevy Engine](https://bevyengine.org/).

## Features

- Generates 6 standard PBR textures as PNG files
- Configurable resolution via CLI
- Manual implementations of 5 noise algorithms for texture generation
- Customizable per-texture functions
- Correct channel mappings for Bevy's `StandardMaterial`

## Installation

```bash
git clone https://github.com/julianbieber/bevy_pbr_generator.git
cd bevy_pbr_generator
cargo install --path .
```

## Usage

```bash
# Generate 1024x1024 textures to ./output (default)
bevy_pbr_generator

# Custom resolution and output directory
bevy_pbr_generator --resolution 512 --output-dir ./textures

# Show help
bevy_pbr_generator --help
```

### CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `-r, --resolution` | Texture width and height in pixels | `1024` |
| `-o, --output-dir` | Output directory for PNG files | `./output` |

## Generated Textures

| Texture | Color Space | Channel Usage | Bevy Field |
|---------|-------------|---------------|------------|
| `base_color_texture.png` | sRGB | RGB = color, A = 1.0 | `base_color_texture` |
| `emissive_texture.png` | sRGB | RGB = emissive color, A = 1.0 | `emissive_texture` |
| `metallic_roughness_texture.png` | Linear | **G = roughness**, **B = metallic**, R = 0, A = 1.0 | `metallic_roughness_texture` |
| `diffuse_transmission_texture.png` | Linear | **A = transmission**, RGB = 1.0 | `diffuse_transmission_texture` |
| `specular_transmission_texture.png` | Linear | **R = transmission**, GBA = 1.0 | `specular_transmission_texture` |
| `thickness_texture.png` | Linear | **G = thickness**, RBA = 1.0 | `thickness_texture` |

## Customization

Edit the texture generation functions in `src/textures.rs`. Each function receives normalized UV coordinates (`Vec2` in `[0.0, 1.0]`) and returns a `Vec2` for channel values.

### Example: Perlin Noise Base Color

```rust
use crate::noise::perlin_noise;

pub fn base_color_texture(uv: Vec2) -> Vec2 {
    let noise = perlin_noise(uv, 5.0);
    let r = (noise + 1.0) / 2.0; // Map from [-1, 1] to [0, 1]
    Vec2::new(r, r * 0.5)
}
```

### Example: Metallic-Roughness with Worley Noise

```rust
use crate::noise::{perlin_noise, worley_noise};

pub fn metallic_roughness_texture(uv: Vec2) -> Vec2 {
    let roughness = worley_noise(uv, 10.0, 30).clamp(0.0, 1.0);
    let metallic = (perlin_noise(uv, 2.0) + 1.0) / 2.0;
    Vec2::new(roughness, metallic) // x = roughness (G), y = metallic (B)
}
```

## Available Noise Functions

All noise functions are in `src/noise.rs`:

| Function | Description |
|----------|-------------|
| `white_noise(uv: Vec2) -> f32` | Random value per pixel |
| `value_noise(uv: Vec2, scale: f32) -> f32` | Interpolated white noise |
| `perlin_noise(uv: Vec2, scale: f32) -> f32` | Perlin noise |
| `simplex_noise(uv: Vec2, scale: f32) -> f32` | Simplex noise (2D) |
| `worley_noise(uv: Vec2, scale: f32, points: usize) -> f32` | Worley noise (F1) |
| `fbm(uv: Vec2, scale: f32, octaves: usize, lacunarity: f32, persistence: f32, noise_fn: fn(Vec2, f32) -> f32) -> f32` | Fractional Brownian Motion |
| `combined_noise(uv: Vec2) -> f32` | Combined noise (example) |

## Bevy Integration

Load the generated textures in Bevy with the correct color space:

```rust
use bevy::prelude::*;

fn setup_textures(asset_server: Res<AssetServer>) {
    // sRGB textures (base_color, emissive)
    let base_color = asset_server.load("textures/base_color_texture.png");
    let emissive = asset_server.load("textures/emissive_texture.png");
    
    // Linear textures - MUST set is_srgb = false
    let metallic_roughness = asset_server.load_with_settings(
        "textures/metallic_roughness_texture.png",
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );
    let diffuse_transmission = asset_server.load_with_settings(
        "textures/diffuse_transmission_texture.png",
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );
    let specular_transmission = asset_server.load_with_settings(
        "textures/specular_transmission_texture.png",
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );
    let thickness = asset_server.load_with_settings(
        "textures/thickness_texture.png",
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );
}
```

## License

MIT License - see [LICENSE](LICENSE) for details.
