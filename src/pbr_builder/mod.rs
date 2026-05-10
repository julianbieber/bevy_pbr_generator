//! PBR Material Builder Module
//!
//! This module provides a shader-based PBR material builder that can:
//! - Create PBR materials from multiple texture types
//! - Export individual texture images with name prefixes
//! - Support all standard PBR pipeline fields
//! - Use WGSL (WebGPU Shading Language) for shader programming
//! - Generate procedural PBR materials and textures

use bevy::prelude::*;
use bevy::render::render_resource::ShaderRef;
use std::path::Path;
use std::collections::HashMap;
use thiserror::Error;

pub mod shaders;
pub mod material;
pub mod exporter;
pub mod generator;

pub use material::*;
pub use exporter::*;
pub use shaders::*;
pub use generator::*;

/// PBR Texture types that map to Bevy's StandardMaterial fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PbrTextureType {
    /// Base color / albedo texture (RGB or RGBA)
    BaseColor,
    /// Metallic-Roughness texture (B: Metallic, G: Roughness)
    MetallicRoughness,
    /// Normal map texture (XYZ in tangent space)
    Normal,
    /// Occlusion texture (R channel)
    Occlusion,
    /// Emissive texture (RGB)
    Emissive,
    /// Height map for parallax/occlusion
    Height,
}

impl PbrTextureType {
    /// Get the standard prefix for this texture type
    pub fn prefix(&self) -> &'static str {
        match self {
            PbrTextureType::BaseColor => "base_color",
            PbrTextureType::MetallicRoughness => "metallic_roughness",
            PbrTextureType::Normal => "normal",
            PbrTextureType::Occlusion => "occlusion",
            PbrTextureType::Emissive => "emissive",
            PbrTextureType::Height => "height",
        }
    }

    /// Get the file extension for this texture type
    pub fn extension(&self) -> &'static str {
        "png"
    }

    /// Get all texture types
    pub fn all() -> &'static [PbrTextureType] {
        &[
            PbrTextureType::BaseColor,
            PbrTextureType::MetallicRoughness,
            PbrTextureType::Normal,
            PbrTextureType::Occlusion,
            PbrTextureType::Emissive,
            PbrTextureType::Height,
        ]
    }
}

/// Error type for PBR material building
#[derive(Error, Debug)]
pub enum PbrBuilderError {
    #[error("Texture type not supported: {0}")]
    UnsupportedTextureType(String),
    #[error("Failed to load image: {0}")]
    ImageLoadError(String),
    #[error("Failed to save image: {0}")]
    ImageSaveError(String),
    #[error("Missing required texture: {0}")]
    MissingTexture(String),
    #[error("Shader compilation error: {0}")]
    ShaderError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Image processing error: {0}")]
    ImageProcessingError(String),
}

/// PBR Material configuration
#[derive(Debug, Clone)]
pub struct PbrMaterialConfig {
    /// Name prefix for exported files
    pub name_prefix: String,
    /// Base color texture path
    pub base_color: Option<String>,
    /// Metallic-Roughness texture path
    pub metallic_roughness: Option<String>,
    /// Normal map texture path
    pub normal: Option<String>,
    /// Occlusion texture path
    pub occlusion: Option<String>,
    /// Emissive texture path
    pub emissive: Option<String>,
    /// Height map texture path
    pub height: Option<String>,
    /// Material parameters
    pub parameters: PbrParameters,
}

impl Default for PbrMaterialConfig {
    fn default() -> Self {
        Self {
            name_prefix: "pbr_material".to_string(),
            base_color: None,
            metallic_roughness: None,
            normal: None,
            occlusion: None,
            emissive: None,
            height: None,
            parameters: PbrParameters::default(),
        }
    }
}

/// PBR Material parameters
#[derive(Debug, Clone)]
pub struct PbrParameters {
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub emissive_factor: [f32; 3],
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
}

impl Default for PbrParameters {
    fn default() -> Self {
        Self {
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            emissive_factor: [1.0, 1.0, 1.0],
            normal_scale: 1.0,
            occlusion_strength: 1.0,
            alpha_cutoff: 0.5,
            double_sided: false,
        }
    }
}

/// Main PBR Material Builder
pub struct PbrMaterialBuilder {
    config: PbrMaterialConfig,
    textures: HashMap<PbrTextureType, Handle<Image>>,
    shaders: HashMap<PbrTextureType, ShaderRef>,
    shader_library: PbrShaderLibrary,
}

impl Default for PbrMaterialBuilder {
    fn default() -> Self {
        Self::new(PbrMaterialConfig::default())
    }
}

impl PbrMaterialBuilder {
    /// Create a new PBR material builder
    pub fn new(config: PbrMaterialConfig) -> Self {
        Self {
            config,
            textures: HashMap::new(),
            shaders: HashMap::new(),
            shader_library: PbrShaderLibrary::default(),
        }
    }

    /// Set the configuration
    pub fn with_config(mut self, config: PbrMaterialConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a texture for a specific PBR texture type
    pub fn with_texture(
        mut self,
        texture_type: PbrTextureType,
        image_handle: Handle<Image>,
    ) -> Self {
        self.textures.insert(texture_type, image_handle);
        self
    }

    /// Add multiple textures at once
    pub fn with_textures(
        mut self,
        textures: impl IntoIterator<Item = (PbrTextureType, Handle<Image>)>,
    ) -> Self {
        for (texture_type, handle) in textures {
            self.textures.insert(texture_type, handle);
        }
        self
    }

    /// Add a custom shader for a specific texture type
    pub fn with_shader(
        mut self,
        texture_type: PbrTextureType,
        shader: ShaderRef,
    ) -> Self {
        self.shaders.insert(texture_type, shader);
        self
    }

    /// Add a custom WGSL shader string for a specific texture type
    pub fn with_wgsl_shader(
        mut self,
        texture_type: PbrTextureType,
        wgsl: &str,
    ) -> Self {
        let shader_ref = ShaderRef::from_wgsl(wgsl, None);
        self.shaders.insert(texture_type, shader_ref);
        self
    }

    /// Use the default shader for a specific texture type from the library
    pub fn with_default_shader(
        mut self,
        texture_type: PbrTextureType,
    ) -> Self {
        let shader_ref = self.shader_library.create_texture_shader_ref(texture_type);
        self.shaders.insert(texture_type, shader_ref);
        self
    }

    /// Use default shaders for all texture types
    pub fn with_all_default_shaders(mut self) -> Self {
        for texture_type in PbrTextureType::all() {
            let shader_ref = self.shader_library.create_texture_shader_ref(*texture_type);
            self.shaders.insert(*texture_type, shader_ref);
        }
        self
    }

    /// Build the PBR material
    pub fn build(
        self,
        asset_server: &AssetServer,
        images: &mut ResMut<Assets<Image>>,
    ) -> Result<PbrMaterial, PbrBuilderError> {
        let mut material = PbrMaterial::new(&self.config);

        // Load and process textures
        for (texture_type, handle) in &self.textures {
            material.set_texture(*texture_type, handle.clone());
        }

        // Apply shaders
        for (texture_type, shader) in &self.shaders {
            material.set_shader(*texture_type, shader.clone());
        }

        // If no shaders were specified, use the combined PBR shader
        if self.shaders.is_empty() {
            let combined_shader = self.shader_library.create_combined_shader_ref();
            for texture_type in PbrTextureType::all() {
                material.set_shader(*texture_type, combined_shader.clone());
            }
        }

        Ok(material)
    }

    /// Export all textures with name prefixes
    pub fn export_textures(
        &self,
        output_dir: &Path,
        images: &Assets<Image>,
    ) -> Result<Vec<std::path::PathBuf>, PbrBuilderError> {
        let mut exported_paths = Vec::new();

        // Create output directory if it doesn't exist
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        for (texture_type, handle) in &self.textures {
            if let Some(image) = images.get(handle) {
                let filename = format!(
                    "{}_{}.{}",
                    self.config.name_prefix,
                    texture_type.prefix(),
                    texture_type.extension()
                );
                let output_path = output_dir.join(filename);

                // Export the image
                self.export_image(image, &output_path)?;
                exported_paths.push(output_path);
            }
        }

        Ok(exported_paths)
    }

    /// Export a single texture
    pub fn export_texture(
        &self,
        texture_type: PbrTextureType,
        output_dir: &Path,
        images: &Assets<Image>,
    ) -> Result<std::path::PathBuf, PbrBuilderError> {
        if let Some(handle) = self.textures.get(&texture_type) {
            if let Some(image) = images.get(handle) {
                let filename = format!(
                    "{}_{}.{}",
                    self.config.name_prefix,
                    texture_type.prefix(),
                    texture_type.extension()
                );
                let output_path = output_dir.join(filename);

                // Create output directory if it doesn't exist
                if !output_dir.exists() {
                    std::fs::create_dir_all(output_dir)?;
                }

                self.export_image(image, &output_path)?;
                return Ok(output_path);
            }
        }
        Err(PbrBuilderError::MissingTexture(format!("{:?}", texture_type)))
    }

    fn export_image(
        &self,
        image: &Image,
        output_path: &std::path::Path,
    ) -> Result<(), PbrBuilderError> {
        use image::io::Writer as ImageWriter;
        use image::{ImageBuffer, Rgba};

        // Convert Bevy Image to image crate ImageBuffer
        let width = image.size.x as u32;
        let height = image.size.y as u32;

        match &image.data {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8Srgb => {
                let pixels: Vec<Rgba<u8>> = image
                    .data
                    .chunks(4)
                    .map(|chunk| {
                        Rgba([chunk[0], chunk[1], chunk[2], chunk[3]])
                    })
                    .collect();

                let img_buffer = ImageBuffer::from_vec(width, height, pixels)
                    .ok_or_else(|| {
                        PbrBuilderError::ImageSaveError(
                            "Failed to create image buffer".to_string(),
                        )
                    })?;

                img_buffer
                    .write_to(
                        &mut std::fs::File::create(output_path).map_err(|e| {
                            PbrBuilderError::ImageSaveError(e.to_string())
                        })?,
                        image::ImageOutputFormat::Png,
                    )
                    .map_err(|e| PbrBuilderError::ImageSaveError(e.to_string()))?;
            }
            _ => {
                return Err(PbrBuilderError::ImageSaveError(
                    format!("Unsupported texture format for export: {:?}", image.texture_format),
                ));
            }
        }

        Ok(())
    }

    /// Get the shader library
    pub fn shader_library(&self) -> &PbrShaderLibrary {
        &self.shader_library
    }

    /// Get the combined WGSL shader source
    pub fn get_combined_wgsl(&self) -> &str {
        self.shader_library.get_combined_shader()
    }

    /// Get a specific shader function by name
    pub fn get_shader_function(&self, name: &str) -> Option<&str> {
        self.shader_library.get_function(name)
    }

    /// Get a specific shader module by texture type
    pub fn get_shader_module(&self, texture_type: PbrTextureType) -> Option<&str> {
        self.shader_library.get_module(texture_type)
    }
}

/// Plugin for PBR Material Builder
pub struct PbrMaterialBuilderPlugin;

impl Plugin for PbrMaterialBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PbrMaterial>()
            .register_type::<PbrMaterialConfig>()
            .register_type::<PbrParameters>()
            .register_type::<PbrMaterialGenerator>()
            .register_type::<PbrGeneratorConfig>()
            .add_systems(Startup, setup_pbr_builder);
    }
}

fn setup_pbr_builder() {
    // Setup can be extended as needed
}

/// Helper trait for creating PBR materials with builder pattern
pub trait PbrMaterialBuilderExt {
    /// Create a PBR material builder with default configuration
    fn pbr_builder() -> PbrMaterialBuilder {
        PbrMaterialBuilder::default()
    }

    /// Create a PBR material builder with custom configuration
    fn pbr_builder_with_config(config: PbrMaterialConfig) -> PbrMaterialBuilder {
        PbrMaterialBuilder::new(config)
    }
}

impl PbrMaterialBuilderExt for PbrMaterialBuilder {}
