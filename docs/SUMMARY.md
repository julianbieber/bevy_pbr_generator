# Bevy PBR Material Builder - Documentation Summary

## 📚 Complete Documentation

This documentation provides comprehensive information about the Bevy PBR Material Builder library, including:

### 📖 Main Documentation
- **[PBR Material Builder Guide](PBR_MATERIAL_BUILDER.md)** - Complete guide with API reference, examples, and best practices

### 🏗️ Project Structure
```
bevy_pbr_builder/
├── Cargo.toml                    # Project configuration
├── README.md                     # Quick start and overview
├── LICENSE                       # MIT License
├── docs/
│   ├── PBR_MATERIAL_BUILDER.md    # Full documentation
│   └── SUMMARY.md                # This file
└── src/
    ├── lib.rs                    # Library exports
    ├── main.rs                   # Example application
    └── pbr_builder/
        ├── mod.rs                # Main module with PbrMaterialBuilder
        ├── material.rs           # PbrMaterial struct and methods
        ├── shaders.rs            # WGSL Shader Library
        └── exporter.rs           # Texture export functionality
```

## 🚀 Quick Reference

### Key Types
| Type | Purpose |
|------|---------|
| `PbrMaterial` | Main material struct with textures and shaders |
| `PbrMaterialBuilder` | Builder for creating and configuring materials |
| `PbrShaderLibrary` | Library of WGSL shader functions |
| `PbrMaterialExporter` | Export textures with name prefixes |
| `PbrTextureType` | Enum for all PBR texture types |
| `PbrMaterialConfig` | Material configuration |
| `PbrParameters` | Material rendering parameters |

### Texture Types
| Type | Description | File Suffix |
|------|-------------|-------------|
| `BaseColor` | Albedo/Diffuse texture | `_base_color.png` |
| `MetallicRoughness` | Metallic (B) + Roughness (G) | `_metallic_roughness.png` |
| `Normal` | Normal map | `_normal.png` |
| `Occlusion` | Ambient occlusion | `_occlusion.png` |
| `Emissive` | Emissive color | `_emissive.png` |
| `Height` | Height map | `_height.png` |

### Basic Usage Pattern
```rust
// 1. Create configuration
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

// 2. Create builder and add textures
let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_texture)
    .with_texture(PbrTextureType::Normal, normal_texture);

// 3. Build material
let material = builder.build(&asset_server, &mut materials)?;

// 4. Use material
commands.spawn(PbrBundle {
    mesh: meshes.add(Mesh::from(shape::Cube::default())),
    material: materials.add(material),
    ..default()
});

// 5. Export textures (optional)
let exported = builder.export_textures(Path::new("exports"), &images)?;
```

## 📋 API Reference

### PbrMaterialBuilder Methods
- `new(config: PbrMaterialConfig) -> Self`
- `with_texture(texture_type, handle) -> Self`
- `with_wgsl_shader(texture_type, wgsl: &str) -> Self`
- `with_default_shader(texture_type) -> Self`
- `build(asset_server, images) -> Result<PbrMaterial, PbrBuilderError>`
- `export_textures(output_dir, images) -> Result<Vec<PathBuf>, PbrBuilderError>`
- `export_texture(texture_type, output_dir, images) -> Result<PathBuf, PbrBuilderError>`
- `shader_library() -> &PbrShaderLibrary`
- `get_combined_wgsl() -> &str`

### PbrShaderLibrary Methods
- `get_function(name: &str) -> Option<&str>`
- `get_module(texture_type) -> Option<&str>`
- `get_combined_shader() -> &str`
- `create_shader_ref(shader: &str) -> ShaderRef`
- `create_texture_shader_ref(texture_type) -> ShaderRef`
- `create_combined_shader_ref() -> ShaderRef`

### PbrMaterialExporter Methods
- `new(config: PbrMaterialConfig) -> Self`
- `export_image(image, output_path) -> Result<(), PbrBuilderError>`
- `export_all_textures(textures, images, output_dir) -> Result<Vec<PathBuf>, PbrBuilderError>`
- `export_texture(texture_type, image, output_dir) -> Result<PathBuf, PbrBuilderError>`
- `export_with_custom_name(texture_type, image, output_dir, custom_name) -> Result<PathBuf, PbrBuilderError>`

## 🎯 Common Tasks

### Creating a Gold Material
```rust
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

let gold_material = PbrMaterialBuilder::new(gold_config)
    .with_texture(PbrTextureType::BaseColor, gold_albedo)
    .with_texture(PbrTextureType::MetallicRoughness, gold_mr)
    .with_texture(PbrTextureType::Normal, gold_normal)
    .build(&asset_server, &mut materials)?;
```

### Creating a Custom Shader
```rust
let custom_shader = r#"
@group(1) @binding(0) var base_color_texture: texture_2d<f32>;
@group(1) @binding(1) var base_color_sampler: sampler;

fn process_base_color(input: BaseColorInput) -> BaseColorOutput {
    let color = textureSample(input.texture, input.sampler, input.uv);
    // Add custom processing
    let processed = color.rgb * 1.1; // Boost color
    return BaseColorOutput {
        color: vec4<f32>(processed, color.a),
        alpha: color.a
    };
}
"#;

let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_texture)
    .with_wgsl_shader(PbrTextureType::BaseColor, custom_shader);
```

### Exporting Textures
```rust
let builder = PbrMaterialBuilder::new(config)
    .with_texture(PbrTextureType::BaseColor, base_color_texture)
    .with_texture(PbrTextureType::Normal, normal_texture);

// Export all textures
let exported = builder.export_textures(Path::new("exports"), &images)?;

// Files created:
// - exports/my_material_base_color.png
// - exports/my_material_normal.png
```

### Using the Shader Library
```rust
let shader_library = PbrShaderLibrary::default();

// Get a specific function
if let Some(fresnel) = shader_library.get_function("fresnel_schlick") {
    println!("Fresnel function: {}", fresnel);
}

// Get the complete combined shader
let combined = shader_library.get_combined_shader();

// Create shader references
let shader_ref = shader_library.create_combined_shader_ref();
```

## 📊 Shader Library Contents

### Math Functions
- `PI`, `PI_2`, `PI_4`, `INV_PI`, `INV_PI_2` - Mathematical constants
- `saturate(x)` - Clamp to [0, 1] range
- `saturate_vec3(x)` - Clamp vector to [0, 1] range
- `lerp(a, b, t)` - Linear interpolation
- `lerp_vec3(a, b, t)` - Vector3 linear interpolation
- `lerp_vec4(a, b, t)` - Vector4 linear interpolation

### PBR Functions
- `fresnel_schlick(cos_theta, f0)` - Fresnel-Schlick approximation
- `fresnel_schlick_roughness(cos_theta, f0, roughness)` - Fresnel with roughness
- `distribution_ggx(n, h, roughness)` - GGX normal distribution
- `geometry_schlick_ggx(n_dot_v, roughness)` - Smith's geometry (GGX)
- `geometry_smith(n, v, l, roughness)` - Smith's geometry shadowing
- `pbr_brdf(n, v, l, h, base_color, metallic, roughness, f0, occlusion)` - Complete PBR BRDF

### Texture Modules
- `BASE_COLOR_MODULE` - Base color processing
- `METALLIC_ROUGHNESS_MODULE` - Metallic-roughness processing
- `NORMAL_MODULE` - Normal map processing
- `OCCLUSION_MODULE` - Occlusion processing
- `EMISSIVE_MODULE` - Emissive processing
- `HEIGHT_MODULE` - Height map processing

## 🎨 Material Parameter Presets

### Metals
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

### Dielectrics
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

## 🔧 Setup & Installation

### Cargo.toml
```toml
[dependencies]
bevy_pbr_builder = { path = "../bevy_pbr_builder" }
bevy = "0.18"
image = "0.25"
```

### Plugin Setup
```rust
use bevy_pbr_builder::*;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(PbrMaterialBuilderPlugin)
    .run();
```

## 📖 Learning Resources

### WGSL (WebGPU Shading Language)
- [WGSL Specification](https://gpuweb.github.io/gpuweb/wgsl/)
- [WebGPU Shading Language Guide](https://www.w3.org/TR/WGSL/)
- [Bevy WGSL Examples](https://bevyengine.org/examples/shader/)

### PBR (Physically Based Rendering)
- [PBR Theory](https://learnopengl.com/PBR/Theory)
- [PBR Practical](https://learnopengl.com/PBR/Lighting)
- [Real-Time Rendering](https://www.realtimerendering.com/)

### Bevy Engine
- [Bevy Documentation](https://bevyengine.org/learn/)
- [Bevy Examples](https://github.com/bevyengine/bevy/tree/main/examples)
- [Bevy Assets](https://bevyengine.org/learn/ecs/resources/#assets)

## 🐛 Troubleshooting

### Common Issues and Solutions

#### Textures Not Loading
- **Check**: Texture paths are correct
- **Check**: Files exist and are accessible
- **Check**: Texture formats are supported
- **Solution**: Use absolute paths or verify relative paths

#### Shader Compilation Errors
- **Check**: WGSL syntax is valid
- **Check**: All required bindings are present
- **Check**: Functions are properly defined
- **Solution**: Compare with shader library functions

#### Export Failures
- **Check**: Output directory permissions
- **Check**: Texture format is supported for export
- **Check**: Texture data is valid
- **Solution**: Try a different output directory

#### Performance Issues
- **Check**: Number of active materials
- **Solution**: Use texture atlases
- **Solution**: Optimize shader complexity
- **Solution**: Use lower resolution textures
- **Solution**: Enable texture compression

## 🤝 Contributing

### How to Contribute
1. **Fork** the repository
2. **Create** a feature branch
3. **Make** your changes
4. **Add** tests for new functionality
5. **Update** documentation
6. **Submit** a pull request

### Contribution Guidelines
- Follow Rust and Bevy conventions
- Document all public APIs
- Add tests for new functionality
- Provide usage examples
- Consider performance implications
- Keep commits atomic and well-described

## 📄 License

This project is licensed under the **MIT License** - see [LICENSE](../LICENSE) for details.

## 📞 Support

- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share ideas
- **Documentation**: Check the full docs for API details

## 🎉 Changelog

### Version 0.1.0
- Initial release
- WGSL shader library with comprehensive PBR functions
- PBR material builder with builder pattern
- Texture export with name prefixes
- Support for all standard PBR texture types
- Full Bevy 0.18 integration
- Complete documentation
- Example application

---

**Documentation Last Updated**: May 2024  
**Library Version**: 0.1.0  
**Bevy Version**: 0.18  
**WGSL Support**: Full
