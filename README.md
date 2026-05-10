# Bevy PBR Material Builder

[![Crates.io](https://img.shields.io/crates/v/bevy_pbr_builder.svg)](https://crates.io/crates/bevy_pbr_builder)
[![Docs.rs](https://docs.rs/bevy_pbr_builder/badge.svg)](https://docs.rs/bevy_pbr_builder)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A comprehensive PBR (Physically Based Rendering) material builder library for **Bevy 0.18** that uses **WGSL** (WebGPU Shading Language) for shader programming.

## Features

✅ **WGSL Shader Library** - Comprehensive collection of PBR shader functions  
✅ **Modular Design** - Each texture type can have its own custom shader  
✅ **Builder Pattern** - Intuitive API for creating PBR materials  
✅ **Texture Export** - Export textures with custom name prefixes  
✅ **Bevy Integration** - Full integration with Bevy's ECS and render pipeline  
✅ **All PBR Texture Types** - Base Color, Metallic-Roughness, Normal, Occlusion, Emissive, Height  

## Quick Start

### Add to Your Project

```toml
[dependencies]
bevy_pbr_builder = { git = "https://github.com/your-repo/bevy_pbr_builder" }
bevy = "0.18"
image = "0.25"
```

### Basic Usage

```rust
use bevy::prelude::*;
use bevy_pbr_builder::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PbrMaterialBuilderPlugin)
        .add_systems(Startup, setup_pbr_material)
        .run();
}

fn setup_pbr_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    // Create gold material
    let config = PbrMaterialConfig {
        name_prefix: "gold_material".to_string(),
        parameters: PbrParameters {
            base_color_factor: [1.0, 0.75, 0.33, 1.0], // Gold color
            metallic_factor: 1.0,
            roughness_factor: 0.1,
            ..default()
        },
        ..default()
    };

    // Create material with textures
    let builder = PbrMaterialBuilder::new(config)
        .with_texture(PbrTextureType::BaseColor, asset_server.load("textures/base_color.png"))
        .with_texture(PbrTextureType::MetallicRoughness, asset_server.load("textures/metallic_roughness.png"))
        .with_texture(PbrTextureType::Normal, asset_server.load("textures/normal.png"));

    // Build and use the material
    let material = builder.build(&asset_server, &mut materials).unwrap();

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube::default())),
        material: materials.add(material),
        ..default()
    });
}
```

## Documentation

- **[Full Documentation](docs/PBR_MATERIAL_BUILDER.md)** - Complete API reference and guides
- **[Examples](#examples)** - Practical usage examples
- **[Shader Library](#shader-library)** - Available WGSL functions

## Shader Library

The library includes a comprehensive collection of **WGSL shader functions** for PBR rendering:

### Math Functions
- `saturate()` - Clamp values to [0, 1] range
- `lerp()` - Linear interpolation
- `PI` constants - Mathematical constants

### PBR Functions
- `fresnel_schlick()` - Fresnel-Schlick approximation
- `distribution_ggx()` - GGX/Trowbridge-Reitz normal distribution
- `geometry_schlick_ggx()` - Smith's geometry shadowing (GGX)
- `geometry_smith()` - Smith's method for geometry shadowing
- `pbr_brdf()` - Complete PBR BRDF calculation

### Texture Processing Modules
Each PBR texture type has its own processing module:
- **Base Color** - Albedo/diffuse with sRGB to linear conversion
- **Metallic-Roughness** - Extracts metallic (B) and roughness (G) channels
- **Normal** - Normal mapping with tangent space transformation
- **Occlusion** - Ambient occlusion from R channel
- **Emissive** - Emissive color with sRGB to linear conversion
- **Height** - Height map sampling

## Texture Export

Export textures with custom name prefixes:

```rust
// Export all textures
let exported = builder.export_textures(Path::new("exports"), &images)?;

// Files will be named:
// - gold_material_base_color.png
// - gold_material_metallic_roughness.png
// - gold_material_normal.png
// - etc.
```

### Supported Export Formats
- `Rgba8Unorm` - 8-bit normalized RGBA
- `Rgba8Srgb` - 8-bit sRGB RGBA  
- `Rgba16Unorm` - 16-bit normalized RGBA
- `Rgba32Float` - 32-bit floating point RGBA

## Examples

### 1. Multiple Materials

```rust
// Create different materials
let gold_material = PbrMaterialBuilder::new(gold_config)
    .with_texture(PbrTextureType::BaseColor, gold_albedo)
    .with_texture(PbrTextureType::MetallicRoughness, gold_mr)
    .build(&asset_server, &mut materials)?;

let rust_material = PbrMaterialBuilder::new(rust_config)
    .with_texture(PbrTextureType::BaseColor, rust_albedo)
    .with_texture(PbrTextureType::MetallicRoughness, rust_mr)
    .with_texture(PbrTextureType::Normal, rust_normal)
    .with_texture(PbrTextureType::Occlusion, rust_ao)
    .build(&asset_server, &mut materials)?;
```

### 2. Custom Shaders

```rust
let custom_shader = r#"
@group(1) @binding(0) var base_color_texture: texture_2d<f32>;
@group(1) @binding(1) var base_color_sampler: sampler;

fn process_base_color(input: BaseColorInput) -> BaseColorOutput {
    let color = textureSample(input.texture, input.sampler, input.uv);
    // Custom processing here
    return BaseColorOutput {
        color: vec4<f32>(color.rgb * input.color_factor.rgb, color.a),
        alpha: color.a
    };
}
"#;

let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_texture)
    .with_wgsl_shader(PbrTextureType::BaseColor, custom_shader);
```

### 3. Using Default Shaders

```rust
let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_texture)
    .with_default_shader(PbrTextureType::BaseColor); // Uses library shader
```

## Project Structure

```
bevy_pbr_builder/
├── Cargo.toml
├── README.md
├── docs/
│   └── PBR_MATERIAL_BUILDER.md      # Full documentation
├── src/
│   ├── lib.rs                      # Library exports
│   ├── main.rs                     # Example application
│   └── pbr_builder/
│       ├── mod.rs                  # Main module & PbrMaterialBuilder
│       ├── material.rs             # PbrMaterial struct
│       ├── shaders.rs              # WGSL Shader Library
│       └── exporter.rs             # Texture export functionality
```

## PBR Texture Types

| Type | Description | Channels Used | File Suffix |
|------|-------------|---------------|-------------|
| `BaseColor` | Albedo/Diffuse | RGB(A) | `_base_color.png` |
| `MetallicRoughness` | Metallic & Roughness | B=Metallic, G=Roughness | `_metallic_roughness.png` |
| `Normal` | Normal Map | XYZ | `_normal.png` |
| `Occlusion` | Ambient Occlusion | R | `_occlusion.png` |
| `Emissive` | Emissive Color | RGB | `_emissive.png` |
| `Height` | Height Map | R | `_height.png` |

## Material Parameters

```rust
pub struct PbrParameters {
    pub base_color_factor: [f32; 4],      // RGBA color multiplier
    pub metallic_factor: f32,             // Metallic multiplier (0.0 - 1.0)
    pub roughness_factor: f32,             // Roughness multiplier (0.0 - 1.0)
    pub emissive_factor: [f32; 3],         // Emissive color multiplier
    pub normal_scale: f32,                // Normal map intensity
    pub occlusion_strength: f32,          // Occlusion effect strength
    pub alpha_cutoff: f32,                // Alpha cutoff for transparency
    pub double_sided: bool,               // Render both sides of geometry
}
```

## Common Parameter Presets

### Metallic Materials
```rust
// Gold
PbrParameters {
    base_color_factor: [1.0, 0.75, 0.33, 1.0],
    metallic_factor: 1.0,
    roughness_factor: 0.1,
    ..default()
}

// Silver
PbrParameters {
    base_color_factor: [0.9, 0.9, 0.9, 1.0],
    metallic_factor: 1.0,
    roughness_factor: 0.05,
    ..default()
}

// Copper
PbrParameters {
    base_color_factor: [0.95, 0.6, 0.4, 1.0],
    metallic_factor: 1.0,
    roughness_factor: 0.15,
    ..default()
}
```

### Dielectric Materials
```rust
// Plastic (Red)
PbrParameters {
    base_color_factor: [0.8, 0.2, 0.2, 1.0],
    metallic_factor: 0.0,
    roughness_factor: 0.5,
    ..default()
}

// Ceramic
PbrParameters {
    base_color_factor: [0.7, 0.7, 0.8, 1.0],
    metallic_factor: 0.0,
    roughness_factor: 0.7,
    ..default()
}

// Glass
PbrParameters {
    base_color_factor: [0.9, 0.9, 1.0, 0.5],
    metallic_factor: 0.0,
    roughness_factor: 0.0,
    ..default()
}
```

## API Reference

### Main Types

- `PbrMaterial` - The material struct
- `PbrMaterialBuilder` - Builder for creating materials
- `PbrShaderLibrary` - Library of WGSL shader functions
- `PbrMaterialExporter` - Texture export functionality
- `PbrTextureType` - Enum for PBR texture types
- `PbrMaterialConfig` - Material configuration
- `PbrParameters` - Material parameters
- `PbrBuilderError` - Error types

### Plugin

```rust
// Add to your Bevy app
App::new()
    .add_plugins(PbrMaterialBuilderPlugin)
    // ... other plugins
```

## Best Practices

1. **Use Descriptive Names**: Name materials and textures descriptively
2. **Only Load What You Need**: Don't load textures you won't use
3. **Use Texture Compression**: Enable compression for better performance
4. **Test Shaders Isolated**: Test custom shaders before integrating
5. **Handle Errors Gracefully**: Provide fallback materials for missing textures
6. **Monitor Memory**: Be mindful of memory usage with many materials

## Troubleshooting

### Textures Not Loading
- ✅ Verify texture paths are correct
- ✅ Check that texture files exist and are accessible
- ✅ Ensure texture formats are supported
- ✅ Use absolute paths or verify relative paths

### Shader Compilation Errors
- ✅ Check WGSL syntax for errors
- ✅ Verify all required bindings are present
- ✅ Ensure shader functions are properly defined
- ✅ Use the shader library functions as a reference

### Export Failures
- ✅ Verify output directory permissions
- ✅ Check that the texture format is supported for export
- ✅ Ensure the texture data is valid
- ✅ Try exporting to a different directory

## Contributing

Contributions are welcome! Please:

1. Follow Rust and Bevy conventions
2. Document all public APIs
3. Add tests for new functionality
4. Provide usage examples
5. Consider performance implications

## License

This project is licensed under the **MIT License**.

## Version History

- **0.1.0** (Current)
  - Initial release
  - WGSL shader library with comprehensive PBR functions
  - PBR material builder with builder pattern
  - Texture export with name prefixes
  - Support for all standard PBR texture types
  - Full Bevy 0.18 integration

---

**Built with ❤️ using Bevy and WGSL**  
**Questions? Issues? Open a GitHub issue!**
