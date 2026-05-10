# 🎨 Bevy PBR Material Builder - Complete Project

## 📦 Project Overview

A **comprehensive PBR (Physically Based Rendering) material builder library** for **Bevy 0.18** that uses **WGSL (WebGPU Shading Language)** for shader programming. This library provides a modular, flexible approach to creating PBR materials with support for multiple shaders per texture type and texture export functionality.

## 🎯 Project Goals

✅ **WGSL Shader Library** - Comprehensive collection of PBR shader functions  
✅ **Modular Design** - Each texture type can have its own custom shader  
✅ **Builder Pattern** - Intuitive API for creating PBR materials  
✅ **Texture Export** - Export textures with custom name prefixes  
✅ **Bevy Integration** - Full integration with Bevy's ECS and render pipeline  
✅ **All PBR Texture Types** - Support for all standard PBR pipeline fields  

## 🏗️ Project Structure

```
bevy_pbr_builder/
├── Cargo.toml                    # Project configuration with Bevy 0.18
├── README.md                     # Quick start, overview, and examples
├── LICENSE                       # MIT License
├── PROJECT_SUMMARY.md            # This file
├── docs/
│   ├── PBR_MATERIAL_BUILDER.md    # Full comprehensive documentation
│   └── SUMMARY.md                # Documentation summary and quick reference
└── src/
    ├── lib.rs                    # Library exports and module declarations
    ├── main.rs                   # Example application demonstrating usage
    └── pbr_builder/
        ├── mod.rs                # Main module with PbrMaterialBuilder
        │                           # - Configuration types (PbrMaterialConfig, PbrParameters)
        │                           # - Builder implementation
        │                           # - Plugin setup
        ├── material.rs           # PbrMaterial struct and implementation
        │                           # - Texture and shader management
        │                           # - Helper methods
        ├── shaders.rs            # WGSL Shader Library
        │                           # - Math functions (saturate, lerp, PI constants)
        │                           # - PBR functions (fresnel_schlick, distribution_ggx, etc.)
        │                           # - Texture processing modules for each type
        │                           # - Combined PBR shader
        └── exporter.rs           # Texture export functionality
                                    # - Multiple format support
                                    # - Name prefixing
                                    # - Directory creation
```

## 📚 Documentation Files

### 1. **README.md**
- Quick start guide
- Feature overview
- Basic usage examples
- Project structure
- PBR texture types table
- Material parameter presets
- API reference summary

### 2. **docs/PBR_MATERIAL_BUILDER.md**
- Complete comprehensive documentation
- Detailed API reference
- Architecture explanation
- Shader library deep dive
- Material builder guide
- Texture export guide
- Multiple practical examples
- Best practices
- Troubleshooting guide
- Contributing guidelines

### 3. **docs/SUMMARY.md**
- Quick reference guide
- API method summaries
- Common tasks with code examples
- Shader library contents
- Material parameter presets
- Setup and installation
- Learning resources

## 🔧 Key Components

### 1. PbrTextureType Enum
```rust
pub enum PbrTextureType {
    BaseColor,           // Base color / albedo texture
    MetallicRoughness,   // Metallic (B channel) + Roughness (G channel)
    Normal,              // Normal map texture
    Occlusion,           // Ambient occlusion texture
    Emissive,            // Emissive texture
    Height,              // Height map texture
}
```

**Methods:**
- `prefix() -> &'static str` - Returns file name prefix
- `all() -> &'static [PbrTextureType]` - Returns all texture types

### 2. PbrMaterialConfig
```rust
pub struct PbrMaterialConfig {
    pub name_prefix: String,              // Prefix for exported files
    pub base_color: Option<String>,       // Base color texture path
    pub metallic_roughness: Option<String>, // Metallic-roughness texture path
    pub normal: Option<String>,           // Normal map texture path
    pub occlusion: Option<String>,        // Occlusion texture path
    pub emissive: Option<String>,         // Emissive texture path
    pub height: Option<String>,           // Height map texture path
    pub parameters: PbrParameters,        // Material rendering parameters
}
```

### 3. PbrParameters
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

### 4. PbrMaterial
```rust
#[derive(Asset, TypePath, Debug, Clone)]
pub struct PbrMaterial {
    pub config: PbrMaterialConfig,
    pub textures: HashMap<PbrTextureType, Handle<Image>>,
    pub shaders: HashMap<PbrTextureType, ShaderRef>,
    pub parameters: PbrParameters,
    pub built: bool,
}
```

### 5. PbrMaterialBuilder
```rust
pub struct PbrMaterialBuilder {
    config: PbrMaterialConfig,
    textures: HashMap<PbrTextureType, Handle<Image>>,
    shaders: HashMap<PbrTextureType, ShaderRef>,
    shader_library: PbrShaderLibrary,
}
```

### 6. PbrShaderLibrary
```rust
pub struct PbrShaderLibrary {
    pub functions: HashMap<String, String>,      // Individual shader functions
    pub modules: HashMap<PbrTextureType, String>, // Texture processing modules
    pub combined_shader: String,                  // Complete combined shader
}
```

### 7. PbrMaterialExporter
```rust
pub struct PbrMaterialExporter {
    config: PbrMaterialConfig,
}
```

## 🎨 WGSL Shader Library

### Math Functions
- `PI`, `PI_2`, `PI_4`, `INV_PI`, `INV_PI_2` - Mathematical constants
- `saturate(x: f32) -> f32` - Clamp to [0, 1] range
- `saturate_vec3(x: vec3<f32>) -> vec3<f32>` - Clamp vector to [0, 1] range
- `lerp(a: f32, b: f32, t: f32) -> f32` - Linear interpolation
- `lerp_vec3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32>` - Vector3 interpolation
- `lerp_vec4(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32>` - Vector4 interpolation

### PBR Functions
- `fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32>` - Fresnel-Schlick approximation
- `fresnel_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32>` - Fresnel with roughness
- `distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32` - GGX/Trowbridge-Reitz normal distribution
- `geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32` - Smith's geometry shadowing (GGX)
- `geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32` - Smith's geometry shadowing
- `pbr_brdf(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, h: vec3<f32>, base_color: vec3<f32>, metallic: f32, roughness: f32, f0: vec3<f32>, occlusion: f32) -> vec3<f32>` - Complete PBR BRDF

### Texture Processing Modules

Each texture type has a dedicated module with:
- Input struct with texture, sampler, UV, and parameters
- Output struct with processed data
- Processing function

**Modules:**
- `BASE_COLOR_MODULE` - Base color with sRGB to linear conversion
- `METALLIC_ROUGHNESS_MODULE` - Extracts metallic (B) and roughness (G) channels
- `NORMAL_MODULE` - Normal mapping with tangent space transformation
- `OCCLUSION_MODULE` - Ambient occlusion from R channel
- `EMISSIVE_MODULE` - Emissive color with sRGB to linear conversion
- `HEIGHT_MODULE` - Height map sampling

### Main PBR Shader

The combined shader includes:
- All shader library functions
- All texture processing modules
- Vertex and fragment shaders
- Complete PBR lighting calculation
- Support for all texture types

## 🚀 Usage Examples

### 1. Basic Material Creation
```rust
use bevy::prelude::*;
use bevy_pbr_builder::*;

fn setup_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let config = PbrMaterialConfig {
        name_prefix: "simple_material".to_string(),
        parameters: PbrParameters {
            base_color_factor: [0.8, 0.2, 0.2, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            ..default()
        },
        ..default()
    };

    let builder = PbrMaterialBuilder::new(config)
        .with_texture(PbrTextureType::BaseColor, asset_server.load("textures/albedo.png"));

    let material = builder.build(&asset_server, &mut materials).unwrap();

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube::default())),
        material: materials.add(material),
        ..default()
    });
}
```

### 2. Complete PBR Material
```rust
let config = PbrMaterialConfig {
    name_prefix: "pbr_material".to_string(),
    parameters: PbrParameters {
        base_color_factor: [1.0, 0.75, 0.33, 1.0], // Gold
        metallic_factor: 1.0,
        roughness_factor: 0.1,
        normal_scale: 1.0,
        occlusion_strength: 1.0,
        ..default()
    },
    ..default()
};

let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, asset_server.load("textures/base_color.png"))
    .with_texture(PbrTextureType::MetallicRoughness, asset_server.load("textures/metallic_roughness.png"))
    .with_texture(PbrTextureType::Normal, asset_server.load("textures/normal.png"))
    .with_texture(PbrTextureType::Occlusion, asset_server.load("textures/occlusion.png"))
    .with_texture(PbrTextureType::Emissive, asset_server.load("textures/emissive.png"))
    .with_texture(PbrTextureType::Height, asset_server.load("textures/height.png"));

let material = builder.build(&asset_server, &mut materials).unwrap();
```

### 3. Custom Shader
```rust
let custom_base_color_shader = r#"
@group(1) @binding(0) var base_color_texture: texture_2d<f32>;
@group(1) @binding(1) var base_color_sampler: sampler;

fn process_base_color(input: BaseColorInput) -> BaseColorOutput {
    let color = textureSample(input.texture, input.sampler, input.uv);
    // Custom processing: increase saturation
    let gray = dot(color.rgb, vec3<f32>(0.3, 0.59, 0.11));
    let saturated = mix(vec3<f32>(gray), color.rgb, 1.5);
    return BaseColorOutput {
        color: vec4<f32>(saturated * input.color_factor.rgb, color.a * input.color_factor.a),
        alpha: color.a * input.color_factor.a
    };
}
"#;

let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_texture)
    .with_wgsl_shader(PbrTextureType::BaseColor, custom_base_color_shader);
```

### 4. Texture Export
```rust
let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_texture)
    .with_texture(PbrTextureType::Normal, normal_texture);

// Export all textures
let exported_paths = builder.export_textures(Path::new("exports"), &images)?;

// Files created:
// - exports/pbr_material_base_color.png
// - exports/pbr_material_normal.png

// Or export specific texture
let path = builder.export_texture(
    PbrTextureType::BaseColor,
    Path::new("exports"),
    &images
)?;
```

### 5. Using Default Shaders from Library
```rust
let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_texture)
    .with_texture(PbrTextureType::Normal, normal_texture)
    .with_default_shader(PbrTextureType::BaseColor)  // Use library shader
    .with_default_shader(PbrTextureType::Normal);    // Use library shader
```

### 6. Accessing Shader Library
```rust
let shader_library = PbrShaderLibrary::default();

// Get a specific function
if let Some(fresnel_code) = shader_library.get_function("fresnel_schlick") {
    println!("Fresnel-Schlick function:\n{}", fresnel_code);
}

// Get a texture module
if let Some(normal_module) = shader_library.get_module(PbrTextureType::Normal) {
    println!("Normal module:\n{}", normal_module);
}

// Get the complete combined shader
let combined_shader = shader_library.get_combined_shader();

// Create shader references
let shader_ref = shader_library.create_combined_shader_ref();
let normal_shader = shader_library.create_texture_shader_ref(PbrTextureType::Normal);
```

## 🎯 Material Parameter Presets

### Metallic Materials

#### Gold
```rust
PbrParameters {
    base_color_factor: [1.0, 0.75, 0.33, 1.0],
    metallic_factor: 1.0,
    roughness_factor: 0.1,
    ..default()
}
```

#### Silver
```rust
PbrParameters {
    base_color_factor: [0.9, 0.9, 0.9, 1.0],
    metallic_factor: 1.0,
    roughness_factor: 0.05,
    ..default()
}
```

#### Copper
```rust
PbrParameters {
    base_color_factor: [0.95, 0.6, 0.4, 1.0],
    metallic_factor: 1.0,
    roughness_factor: 0.15,
    ..default()
}
```

#### Iron
```rust
PbrParameters {
    base_color_factor: [0.6, 0.6, 0.65, 1.0],
    metallic_factor: 0.9,
    roughness_factor: 0.3,
    ..default()
}
```

### Dielectric Materials

#### Plastic (Red)
```rust
PbrParameters {
    base_color_factor: [0.8, 0.2, 0.2, 1.0],
    metallic_factor: 0.0,
    roughness_factor: 0.5,
    ..default()
}
```

#### Plastic (Blue)
```rust
PbrParameters {
    base_color_factor: [0.2, 0.2, 0.8, 1.0],
    metallic_factor: 0.0,
    roughness_factor: 0.4,
    ..default()
}
```

#### Ceramic
```rust
PbrParameters {
    base_color_factor: [0.7, 0.7, 0.8, 1.0],
    metallic_factor: 0.0,
    roughness_factor: 0.7,
    ..default()
}
```

#### Glass
```rust
PbrParameters {
    base_color_factor: [0.9, 0.9, 1.0, 0.5],
    metallic_factor: 0.0,
    roughness_factor: 0.0,
    ..default()
}
```

#### Rubber
```rust
PbrParameters {
    base_color_factor: [0.1, 0.1, 0.1, 1.0],
    metallic_factor: 0.0,
    roughness_factor: 0.9,
    ..default()
}
```

## 📊 File Naming Convention

When exporting textures, the library uses the following naming pattern:

```
{name_prefix}_{texture_type}.png
```

### Examples

With `name_prefix = "gold_material"`:
- `gold_material_base_color.png`
- `gold_material_metallic_roughness.png`
- `gold_material_normal.png`
- `gold_material_occlusion.png`
- `gold_material_emissive.png`
- `gold_material_height.png`

## 🔧 Supported Features

### Texture Formats
- ✅ `Rgba8Unorm` - 8-bit normalized RGBA
- ✅ `Rgba8Srgb` - 8-bit sRGB RGBA
- ✅ `Rgba16Unorm` - 16-bit normalized RGBA
- ✅ `Rgba32Float` - 32-bit floating point RGBA

### PBR Features
- ✅ Physically Based Rendering
- ✅ Image-Based Lighting (IBL) ready
- ✅ Metallic-roughness workflow
- ✅ Normal mapping
- ✅ Ambient occlusion
- ✅ Emissive materials
- ✅ Height mapping (parallax ready)

### Shader Features
- ✅ WGSL (WebGPU Shading Language)
- ✅ Modular shader design
- ✅ Custom shaders per texture type
- ✅ Shader library with PBR functions
- ✅ Combined shader generation

### Export Features
- ✅ PNG export
- ✅ Custom name prefixes
- ✅ Automatic directory creation
- ✅ Multiple texture format support
- ✅ Individual texture export
- ✅ Batch texture export

## 🎓 Learning Resources

### WGSL (WebGPU Shading Language)
- [WGSL Specification](https://gpuweb.github.io/gpuweb/wgsl/)
- [WebGPU Shading Language Guide](https://www.w3.org/TR/WGSL/)
- [Bevy WGSL Examples](https://bevyengine.org/examples/shader/)
- [WebGPU MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API)

### PBR (Physically Based Rendering)
- [LearnOpenGL - PBR Theory](https://learnopengl.com/PBR/Theory)
- [LearnOpenGL - PBR Lighting](https://learnopengl.com/PBR/Lighting)
- [Real-Time Rendering Book](https://www.realtimerendering.com/)
- [PBR Guide by Marmoset](https://marmoset.co/posts/physically-based-rendering-and-image-based-lighting/)
- [Substance Painter PBR Guide](https://www.allegorithmic.com/pbr-guide)

### Bevy Engine
- [Bevy Official Documentation](https://bevyengine.org/learn/)
- [Bevy GitHub Repository](https://github.com/bevyengine/bevy)
- [Bevy Examples](https://github.com/bevyengine/bevy/tree/main/examples)
- [Bevy Asset Management](https://bevyengine.org/learn/ecs/resources/#assets)
- [Bevy Shader Book](https://bevyengine.org/learn/shader/)

### Rust
- [Rust Documentation](https://doc.rust-lang.org/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [The Rust Book](https://doc.rust-lang.org/book/)

## 🐛 Troubleshooting

### Common Issues

#### 1. Textures Not Loading
**Symptoms:** Materials appear black or use default colors.

**Solutions:**
- ✅ Verify texture paths are correct
- ✅ Check that texture files exist and are accessible
- ✅ Ensure texture formats are supported
- ✅ Use absolute paths or verify relative paths
- ✅ Check file permissions

#### 2. Shader Compilation Errors
**Symptoms:** Application panics or fails to start with shader errors.

**Solutions:**
- ✅ Check WGSL syntax for errors
- ✅ Verify all required bindings are present
- ✅ Ensure shader functions are properly defined
- ✅ Use the shader library functions as a reference
- ✅ Check for typos in variable names
- ✅ Verify binding indices match

#### 3. Export Failures
**Symptoms:** Texture export fails or produces corrupted files.

**Solutions:**
- ✅ Verify output directory permissions
- ✅ Check that the texture format is supported for export
- ✅ Ensure the texture data is valid
- ✅ Try exporting to a different directory
- ✅ Check disk space

#### 4. Performance Issues
**Symptoms:** Low frame rates with many PBR materials.

**Solutions:**
- ✅ Reduce the number of active materials
- ✅ Use texture atlases for many small textures
- ✅ Optimize shader complexity
- ✅ Use lower resolution textures when appropriate
- ✅ Enable texture compression
- ✅ Use mipmaps for distant objects

#### 5. Missing Textures in Export
**Symptoms:** Not all textures are exported.

**Solutions:**
- ✅ Verify all textures are added to the builder
- ✅ Check that texture handles are valid
- ✅ Ensure textures are loaded before export
- ✅ Check that the texture type is supported

## 🤝 Contributing

### How to Contribute

1. **Fork** the repository on GitHub
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes
4. **Add** tests for new functionality
5. **Update** documentation
6. **Format** your code (`cargo fmt`)
7. **Check** for clippy warnings (`cargo clippy`)
8. **Commit** your changes (`git commit -m 'Add amazing feature'`)
9. **Push** to the branch (`git push origin feature/amazing-feature`)
10. **Submit** a pull request

### Contribution Guidelines

- **Code Style**: Follow Rust and Bevy conventions
- **Documentation**: Document all public APIs with rustdoc
- **Testing**: Add tests for new functionality
- **Examples**: Provide usage examples for new features
- **Performance**: Consider performance implications of changes
- **Commits**: Keep commits atomic and well-described
- **Pull Requests**: Provide clear descriptions and context

### Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## 📄 License

This project is licensed under the **MIT License** - see [LICENSE](LICENSE) for details.

```
MIT License

Copyright (c) 2024 Bevy PBR Material Builder

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 🎉 Version History

### Version 0.1.0 (Current)
- ✅ Initial release
- ✅ WGSL shader library with comprehensive PBR functions
- ✅ PBR material builder with builder pattern
- ✅ Texture export with name prefixes
- ✅ Support for all standard PBR texture types
- ✅ Full Bevy 0.18 integration
- ✅ Complete documentation
- ✅ Example application
- ✅ Error handling
- ✅ Type safety

### Future Versions
- 🔜 Bevy 0.19+ compatibility
- 🔜 Additional shader functions
- 🔜 More texture formats support
- 🔜 Advanced PBR features (clearcoat, anisotropy, etc.)
- 🔜 Performance optimizations
- 🔜 More examples and tutorials

## 📞 Support & Community

- **GitHub Issues**: [https://github.com/your-repo/bevy_pbr_builder/issues](https://github.com/your-repo/bevy_pbr_builder/issues)
- **GitHub Discussions**: [https://github.com/your-repo/bevy_pbr_builder/discussions](https://github.com/your-repo/bevy_pbr_builder/discussions)
- **Bevy Discord**: [https://discord.gg/bevy](https://discord.gg/bevy)
- **Bevy Forums**: [https://bevyengine.org/community/](https://bevyengine.org/community/)

## 🏆 Acknowledgments

- **Bevy Engine**: [https://bevyengine.org/](https://bevyengine.org/)
- **WebGPU**: [https://www.w3.org/TR/webgpu/](https://www.w3.org/TR/webgpu/)
- **WGSL**: [https://gpuweb.github.io/gpuweb/wgsl/](https://gpuweb.github.io/gpuweb/wgsl/)
- **All contributors** who have helped improve this library

---

**Project Status**: ✅ Complete and Ready to Use  
**Bevy Version**: 0.18  
**WGSL Support**: Full  
**Documentation**: Complete  
**Examples**: Included  
**License**: MIT  

**Built with ❤️ using Rust, Bevy, and WGSL**  
**Questions? Issues? Contributions? Open a GitHub issue or pull request!**
