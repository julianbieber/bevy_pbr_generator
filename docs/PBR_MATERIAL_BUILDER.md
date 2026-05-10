# Bevy PBR Material Builder Documentation

A comprehensive PBR (Physically Based Rendering) material builder library for Bevy 0.18 that uses WGSL (WebGPU Shading Language) for shader programming.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Shader Library](#shader-library)
- [Material Builder](#material-builder)
- [Texture Export](#texture-export)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

The Bevy PBR Material Builder provides a flexible and modular approach to creating PBR materials in Bevy. It leverages WGSL (WebGPU Shading Language) to offer fine-grained control over shader behavior for each texture type, while also providing a comprehensive library of pre-built PBR shader functions.

## Features

### Core Features
- **WGSL Shader Library**: Comprehensive collection of PBR shader functions
- **Modular Design**: Each texture type can have its own shader
- **Builder Pattern**: Intuitive API for creating PBR materials
- **Texture Export**: Export textures with custom name prefixes
- **Bevy Integration**: Full integration with Bevy's ECS and render pipeline

### Supported PBR Texture Types
- **Base Color**: Albedo/diffuse texture (RGB or RGBA)
- **Metallic-Roughness**: Combined metallic (B channel) and roughness (G channel) texture
- **Normal**: Normal map texture (XYZ in tangent space)
- **Occlusion**: Ambient occlusion texture (R channel)
- **Emissive**: Emissive texture (RGB)
- **Height**: Height map for parallax/occlusion effects

## Quick Start

### Adding to Your Project

Add the following to your `Cargo.toml`:

```toml
[dependencies]
bevy_pbr_builder = { path = "path/to/bevy_pbr_builder" }
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
    // Create PBR material configuration
    let config = PbrMaterialConfig {
        name_prefix: "my_material".to_string(),
        parameters: PbrParameters {
            base_color_factor: [1.0, 0.5, 0.3, 1.0],
            metallic_factor: 0.8,
            roughness_factor: 0.2,
            ..default()
        },
        ..default()
    };

    // Create material builder
    let builder = PbrMaterialBuilder::new(config);

    // Load and add textures
    let builder = builder
        .with_texture(PbrTextureType::BaseColor, asset_server.load("textures/albedo.png"))
        .with_texture(PbrTextureType::MetallicRoughness, asset_server.load("textures/metallic_roughness.png"))
        .with_texture(PbrTextureType::Normal, asset_server.load("textures/normal.png"));

    // Build the material
    let material = builder.build(&asset_server, &mut materials).unwrap();

    // Spawn entity with PBR material
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube::default())),
        material: materials.add(material),
        ..default()
    });
}
```

## Architecture

### Module Structure

```
bevy_pbr_builder
├── pbr_builder
│   ├── mod.rs          # Main module with PbrMaterialBuilder
│   ├── material.rs     # PbrMaterial struct and implementation
│   ├── shaders.rs      # WGSL Shader Library
│   └── exporter.rs     # Texture export functionality
└── lib.rs              # Library exports
```

### Core Components

#### 1. PbrMaterial
The main material struct that stores:
- Configuration (`PbrMaterialConfig`)
- Texture handles (`HashMap<PbrTextureType, Handle<Image>>`)
- Shader references (`HashMap<PbrTextureType, ShaderRef>`)
- Material parameters (`PbrParameters`)

#### 2. PbrMaterialBuilder
Builder struct that provides:
- Texture management
- Shader assignment
- Material construction
- Texture export

#### 3. PbrShaderLibrary
Comprehensive library of WGSL shader functions:
- Math utilities
- PBR calculations
- Texture processing modules
- Combined shader generation

#### 4. PbrMaterialExporter
Handles texture export with:
- Multiple format support
- Name prefixing
- Directory creation

## Shader Library

### Overview

The shader library provides a comprehensive collection of WGSL functions for PBR rendering. All functions are designed to work together seamlessly and follow the WebGPU shading language specification.

### Available Functions

#### Math Functions

```wgsl
// Constants
const PI: f32 = 3.14159265359;
const PI_2: f32 = PI * 2.0;
const PI_4: f32 = PI * 4.0;
const INV_PI: f32 = 1.0 / PI;
const INV_PI_2: f32 = 1.0 / (PI * 2.0);

// Clamping
fn saturate(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + (b - a) * t;
}
```

#### PBR Functions

```wgsl
// Fresnel-Schlick approximation
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32>

// GGX/Trowbridge-Reitz normal distribution
fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32

// Smith's geometry shadowing
fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32

// Complete PBR BRDF
fn pbr_brdf(
    n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, h: vec3<f32>,
    base_color: vec3<f32>, metallic: f32, roughness: f32,
    f0: vec3<f32>, occlusion: f32
) -> vec3<f32>
```

### Texture Processing Modules

Each texture type has its own module with input/output structs and processing function:

#### Base Color Module
```wgsl
struct BaseColorInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    color_factor: vec4<f32>,
};

struct BaseColorOutput {
    color: vec4<f32>,
    alpha: f32,
};

fn process_base_color(input: BaseColorInput) -> BaseColorOutput
```

#### Metallic-Roughness Module
```wgsl
struct MetallicRoughnessInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
};

struct MetallicRoughnessOutput {
    metallic: f32,
    roughness: f32,
};

fn process_metallic_roughness(input: MetallicRoughnessInput) -> MetallicRoughnessOutput
```

#### Normal Module
```wgsl
struct NormalInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    normal_scale: f32,
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
    normal: vec3<f32>,
};

struct NormalOutput {
    normal: vec3<f32>,
};

fn process_normal(input: NormalInput) -> NormalOutput
```

### Using the Shader Library

```rust
use bevy_pbr_builder::*;

// Access the shader library
let shader_library = PbrShaderLibrary::default();

// Get a specific function
if let Some(fresnel_code) = shader_library.get_function("fresnel_schlick") {
    println!("Fresnel-Schlick function:\n{}", fresnel_code);
}

// Get a texture module
if let Some(base_color_module) = shader_library.get_module(PbrTextureType::BaseColor) {
    println!("Base Color module:\n{}", base_color_module);
}

// Get the complete combined shader
let combined_shader = shader_library.get_combined_shader();

// Create shader references
let shader_ref = shader_library.create_combined_shader_ref();
let texture_shader = shader_library.create_texture_shader_ref(PbrTextureType::Normal);
```

## Material Builder

### Creating Materials

#### Basic Material
```rust
let config = PbrMaterialConfig::default();
let builder = PbrMaterialBuilder::new(config);
let material = builder.build(&asset_server, &mut materials)?;
```

#### Material with Textures
```rust
let config = PbrMaterialConfig {
    name_prefix: "rusty_metal".to_string(),
    ..default()
};

let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_handle)
    .with_texture(PbrTextureType::MetallicRoughness, mr_handle)
    .with_texture(PbrTextureType::Normal, normal_handle);

let material = builder.build(&asset_server, &mut materials)?;
```

#### Material with Custom Shaders
```rust
let custom_shader = r#"
@group(1) @binding(0) var my_texture: texture_2d<f32>;
@group(1) @binding(1) var my_sampler: sampler;

fn my_custom_processing(uv: vec2<f32>) -> vec4<f32> {
    // Custom shader logic
    return textureSample(my_texture, my_sampler, uv);
}
"#;

let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_handle)
    .with_wgsl_shader(PbrTextureType::BaseColor, custom_shader);
```

#### Material with Default Shaders from Library
```rust
let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_handle)
    .with_default_shader(PbrTextureType::BaseColor);
```

### Material Parameters

The `PbrParameters` struct controls the material's appearance:

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

### Parameter Presets

#### Metallic Materials (Gold, Silver, Copper)
```rust
PbrParameters {
    base_color_factor: [1.0, 0.75, 0.33, 1.0],  // Gold
    metallic_factor: 1.0,
    roughness_factor: 0.1,
    ..default()
}
```

#### Dielectric Materials (Plastic, Ceramic)
```rust
PbrParameters {
    base_color_factor: [0.8, 0.2, 0.2, 1.0],   // Red plastic
    metallic_factor: 0.0,
    roughness_factor: 0.5,
    ..default()
}
```

#### Glossy Materials (Painted Metal, Lacquer)
```rust
PbrParameters {
    base_color_factor: [0.3, 0.3, 0.8, 1.0],   // Blue painted metal
    metallic_factor: 0.5,
    roughness_factor: 0.2,
    ..default()
}
```

## Texture Export

### Exporting Textures

The PBR Material Builder can export textures with custom name prefixes:

```rust
let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_handle)
    .with_texture(PbrTextureType::Normal, normal_handle);

// Export all textures
let exported_paths = builder.export_textures(Path::new("exports"), &images)?;

// Export specific texture
let path = builder.export_texture(PbrTextureType::BaseColor, Path::new("exports"), &images)?;
```

### File Naming Convention

Textures are exported with the following naming pattern:

```
{name_prefix}_{texture_type}.png
```

Examples:
- `gold_material_base_color.png`
- `gold_material_metallic_roughness.png`
- `gold_material_normal.png`
- `gold_material_occlusion.png`
- `gold_material_emissive.png`
- `gold_material_height.png`

### Supported Texture Formats

The exporter supports the following Bevy texture formats:
- `Rgba8Unorm` - 8-bit normalized RGBA
- `Rgba8Srgb` - 8-bit sRGB RGBA
- `Rgba16Unorm` - 16-bit normalized RGBA
- `Rgba32Float` - 32-bit floating point RGBA

### Custom Export

```rust
let exporter = PbrMaterialExporter::new(config);

// Export with custom name
exporter.export_with_custom_name(
    PbrTextureType::BaseColor,
    &image,
    Path::new("custom_exports"),
    "special_material"
)?;

// Export all texture types
exporter.export_all_texture_types(&images, &textures, Path::new("all_textures"))?;
```

## API Reference

### Structs

#### PbrMaterial
```rust
pub struct PbrMaterial {
    pub config: PbrMaterialConfig,
    pub textures: HashMap<PbrTextureType, Handle<Image>>,
    pub shaders: HashMap<PbrTextureType, ShaderRef>,
    pub parameters: PbrParameters,
    pub built: bool,
}
```

**Methods:**
- `new(config: &PbrMaterialConfig) -> Self`
- `set_texture(texture_type: PbrTextureType, handle: Handle<Image>)`
- `set_shader(texture_type: PbrTextureType, shader: ShaderRef)`
- `get_shader(texture_type: PbrTextureType) -> ShaderRef`
- `get_texture(texture_type: PbrTextureType) -> Option<&Handle<Image>>`
- `mark_built()`
- `is_built() -> bool`
- `active_texture_types() -> Vec<PbrTextureType>`
- `custom_shader_types() -> Vec<PbrTextureType>`
- `has_texture(texture_type: PbrTextureType) -> bool`
- `has_shader(texture_type: PbrTextureType) -> bool`

#### PbrMaterialBuilder
```rust
pub struct PbrMaterialBuilder {
    config: PbrMaterialConfig,
    textures: HashMap<PbrTextureType, Handle<Image>>,
    shaders: HashMap<PbrTextureType, ShaderRef>,
    shader_library: PbrShaderLibrary,
}
```

**Methods:**
- `new(config: PbrMaterialConfig) -> Self`
- `with_texture(texture_type: PbrTextureType, image_handle: Handle<Image>) -> Self`
- `with_wgsl_shader(texture_type: PbrTextureType, wgsl: &str) -> Self`
- `with_default_shader(texture_type: PbrTextureType) -> Self`
- `build(asset_server: &AssetServer, images: &mut ResMut<Assets<Image>>) -> Result<PbrMaterial, PbrBuilderError>`
- `export_textures(output_dir: &Path, images: &Assets<Image>) -> Result<Vec<PathBuf>, PbrBuilderError>`
- `export_texture(texture_type: PbrTextureType, output_dir: &Path, images: &Assets<Image>) -> Result<PathBuf, PbrBuilderError>`
- `shader_library() -> &PbrShaderLibrary`
- `get_combined_wgsl() -> &str`

#### PbrShaderLibrary
```rust
pub struct PbrShaderLibrary {
    pub functions: HashMap<String, String>,
    pub modules: HashMap<PbrTextureType, String>,
    pub combined_shader: String,
}
```

**Methods:**
- `get_function(name: &str) -> Option<&str>`
- `get_module(texture_type: PbrTextureType) -> Option<&str>`
- `get_combined_shader() -> &str`
- `create_shader_ref(shader: &str) -> ShaderRef`
- `create_texture_shader_ref(texture_type: PbrTextureType) -> ShaderRef`
- `create_combined_shader_ref() -> ShaderRef`

#### PbrMaterialExporter
```rust
pub struct PbrMaterialExporter {
    config: PbrMaterialConfig,
}
```

**Methods:**
- `new(config: PbrMaterialConfig) -> Self`
- `export_image(image: &Image, output_path: &Path) -> Result<(), PbrBuilderError>`
- `export_all_textures(textures: &HashMap<PbrTextureType, Handle<Image>>, images: &Assets<Image>, output_dir: &Path) -> Result<Vec<PathBuf>, PbrBuilderError>`
- `export_texture(texture_type: PbrTextureType, image: &Image, output_dir: &Path) -> Result<PathBuf, PbrBuilderError>`
- `export_with_custom_name(texture_type: PbrTextureType, image: &Image, output_dir: &Path, custom_name: &str) -> Result<PathBuf, PbrBuilderError>`

### Enums

#### PbrTextureType
```rust
pub enum PbrTextureType {
    BaseColor,
    MetallicRoughness,
    Normal,
    Occlusion,
    Emissive,
    Height,
}
```

**Methods:**
- `prefix() -> &'static str` - Returns the texture type prefix for file naming
- `all() -> &'static [PbrTextureType]` - Returns all texture types

#### PbrBuilderError
```rust
pub enum PbrBuilderError {
    UnsupportedTextureType(String),
    ImageLoadError(String),
    ImageSaveError(String),
    MissingTexture(String),
    ShaderError(String),
    IoError(std::io::Error),
}
```

### Plugins

#### PbrMaterialBuilderPlugin
```rust
pub struct PbrMaterialBuilderPlugin;
```

Registers the PBR material types with Bevy's ECS.

## Examples

### Complete PBR Material Setup

```rust
use bevy::prelude::*;
use bevy_pbr_builder::*;

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    // Create gold material
    let gold_config = PbrMaterialConfig {
        name_prefix: "gold".to_string(),
        parameters: PbrParameters {
            base_color_factor: [1.0, 0.75, 0.33, 1.0],
            metallic_factor: 1.0,
            roughness_factor: 0.1,
            ..default()
        },
        ..default()
    };

    let gold_builder = PbrMaterialBuilder::new(gold_config)
        .with_texture(PbrTextureType::BaseColor, asset_server.load("textures/gold_albedo.png"))
        .with_texture(PbrTextureType::MetallicRoughness, asset_server.load("textures/gold_mr.png"))
        .with_texture(PbrTextureType::Normal, asset_server.load("textures/gold_normal.png"));

    let gold_material = gold_builder.build(&asset_server, &mut materials).unwrap();

    // Create rusty metal material
    let rust_config = PbrMaterialConfig {
        name_prefix: "rusty_metal".to_string(),
        parameters: PbrParameters {
            base_color_factor: [0.7, 0.3, 0.1, 1.0],
            metallic_factor: 0.8,
            roughness_factor: 0.6,
            ..default()
        },
        ..default()
    };

    let rust_builder = PbrMaterialBuilder::new(rust_config)
        .with_texture(PbrTextureType::BaseColor, asset_server.load("textures/rust_albedo.png"))
        .with_texture(PbrTextureType::MetallicRoughness, asset_server.load("textures/rust_mr.png"))
        .with_texture(PbrTextureType::Normal, asset_server.load("textures/rust_normal.png"))
        .with_texture(PbrTextureType::Occlusion, asset_server.load("textures/rust_ao.png"));

    let rust_material = rust_builder.build(&asset_server, &mut materials).unwrap();

    // Spawn objects with different materials
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube::default())),
        material: materials.add(gold_material),
        transform: Transform::from_xyz(-2.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube::default())),
        material: materials.add(rust_material),
        transform: Transform::from_xyz(2.0, 0.0, 0.0),
        ..default()
    });

    // Setup camera and lighting
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            range: 10.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 3.0, 0.0),
        ..default()
    });
}
```

### Custom Shader Example

```rust
use bevy::prelude::*;
use bevy_pbr_builder::*;

fn setup_custom_shader_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    // Custom WGSL shader for base color with edge highlighting
    let edge_highlight_shader = r#"
    @group(1) @binding(0) var base_color_texture: texture_2d<f32>;
    @group(1) @binding(1) var base_color_sampler: sampler;

    fn process_base_color_with_edges(input: BaseColorInput) -> BaseColorOutput {
        let sampled_color = textureSample(input.texture, input.sampler, input.uv);
        
        // Convert to linear space
        let linear_color = vec3<f32>(
            sampled_color.r * sampled_color.r,
            sampled_color.g * sampled_color.g,
            sampled_color.b * sampled_color.b
        );
        
        // Add edge detection (simple Sobel filter)
        let tex_size = textureDimensions(input.texture);
        let uv_size = vec2<f32>(1.0 / f32(tex_size.x), 1.0 / f32(tex_size.y));
        
        let color_left = textureSample(input.texture, input.sampler, input.uv + vec2<f32>(-uv_size.x, 0.0)).rgb;
        let color_right = textureSample(input.texture, input.sampler, input.uv + vec2<f32>(uv_size.x, 0.0)).rgb;
        let color_up = textureSample(input.texture, input.sampler, input.uv + vec2<f32>(0.0, uv_size.y)).rgb;
        let color_down = textureSample(input.texture, input.sampler, input.uv + vec2<f32>(0.0, -uv_size.y)).rgb;
        
        let edge_x = length(color_right - color_left);
        let edge_y = length(color_up - color_down);
        let edge_intensity = (edge_x + edge_y) * 2.0;
        
        // Apply edge highlight
        let final_color = linear_color + vec3<f32>(edge_intensity * 0.5);
        
        return BaseColorOutput {
            color: vec4<f32>(final_color * input.color_factor.rgb, sampled_color.a * input.color_factor.a),
            alpha: sampled_color.a * input.color_factor.a
        };
    }
    "#;

    let config = PbrMaterialConfig {
        name_prefix: "edge_highlight".to_string(),
        ..default()
    };

    let builder = PbrMaterialBuilder::new(config)
        .with_texture(PbrTextureType::BaseColor, asset_server.load("textures/base.png"))
        .with_wgsl_shader(PbrTextureType::BaseColor, edge_highlight_shader);

    let material = builder.build(&asset_server, &mut materials).unwrap();

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube::default())),
        material: materials.add(material),
        ..default()
    });
}
```

### Texture Export Example

```rust
use bevy::prelude::*;
use bevy_pbr_builder::*;
use std::path::Path;

fn export_material_textures(
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
) {
    let config = PbrMaterialConfig {
        name_prefix: "exported_material".to_string(),
        ..default()
    };

    let builder = PbrMaterialBuilder::new(config)
        .with_texture(PbrTextureType::BaseColor, asset_server.load("textures/albedo.png"))
        .with_texture(PbrTextureType::MetallicRoughness, asset_server.load("textures/mr.png"))
        .with_texture(PbrTextureType::Normal, asset_server.load("textures/normal.png"))
        .with_texture(PbrTextureType::Occlusion, asset_server.load("textures/ao.png"));

    // Export all textures
    if let Ok(exported) = builder.export_textures(Path::new("exports"), &images) {
        println!("Exported {} textures:", exported.len());
        for path in exported {
            println!("  - {}", path.display());
        }
    }

    // Or export with custom exporter
    let exporter = PbrMaterialExporter::new(config);
    let textures = builder.get_all_textures();
    
    if let Ok(exported) = exporter.export_all_textures(textures, &images, Path::new("custom_exports")) {
        println!("Custom export completed: {:?}", exported);
    }
}
```

## Best Practices

### 1. Material Organization
- Use descriptive name prefixes for materials
- Group related textures together
- Consider using a consistent naming convention for texture files

### 2. Performance Optimization
- Only load textures that are actually needed
- Use appropriate texture compression formats
- Consider texture atlases for many small textures
- Use mipmaps for textures viewed from a distance

### 3. Shader Development
- Test shaders in isolation before integrating
- Use the shader library functions when possible
- Keep shaders modular and reusable
- Document custom shader parameters

### 4. Error Handling
- Handle missing textures gracefully
- Provide fallback materials
- Validate shader compilation
- Check for texture format compatibility

### 5. Memory Management
- Unload unused textures
- Use weak references when appropriate
- Monitor memory usage with many materials

## Troubleshooting

### Common Issues

#### 1. Textures Not Loading
**Symptoms:** Materials appear black or use default colors.

**Solutions:**
- Verify texture paths are correct
- Check that texture files exist and are accessible
- Ensure texture formats are supported
- Use absolute paths or verify relative paths

#### 2. Shader Compilation Errors
**Symptoms:** Application panics or fails to start with shader errors.

**Solutions:**
- Check WGSL syntax for errors
- Verify all required bindings are present
- Ensure shader functions are properly defined
- Use the shader library functions as a reference

#### 3. Export Failures
**Symptoms:** Texture export fails or produces corrupted files.

**Solutions:**
- Verify output directory permissions
- Check that the texture format is supported for export
- Ensure the texture data is valid
- Try exporting to a different directory

#### 4. Performance Issues
**Symptoms:** Low frame rates with many PBR materials.

**Solutions:**
- Reduce the number of active materials
- Use texture atlases
- Optimize shader complexity
- Use lower resolution textures when appropriate
- Enable texture compression

### Debugging Tools

#### Logging
```rust
// Enable Bevy's logging
App::new()
    .add_plugins(LogPlugin::default())
    .add_plugins(DefaultPlugins)
    // ...
```

#### Shader Validation
```rust
// Check if a shader is valid
let shader_ref = ShaderRef::from_wgsl(wgsl_code, None);
// Bevy will log compilation errors
```

#### Texture Inspection
```rust
// Check loaded textures
for (handle, image) in &images {
    println!("Texture: {:?}, Size: {}x{}, Format: {:?}", 
             handle, image.size.x, image.size.y, image.texture_format);
}
```

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Code Style**: Follow Rust and Bevy conventions
2. **Documentation**: Document all public APIs
3. **Testing**: Add tests for new functionality
4. **Examples**: Provide usage examples
5. **Performance**: Consider performance implications

## License

This project is licensed under the MIT License.

## Version History

- **0.1.0**: Initial release
  - WGSL shader library
  - PBR material builder
  - Texture export functionality
  - Support for all standard PBR texture types
