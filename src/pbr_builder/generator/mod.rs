//! PBR Material Generator Module
//!
//! This module provides procedural PBR material generation capabilities.
//! It can generate textures, create material configurations, and build
//! complete PBR materials from scratch.

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use image::{ImageBuffer, Rgba};
use std::path::Path;

use super::{PbrBuilderError, PbrMaterialConfig, PbrParameters, PbrTextureType};

/// PBR Material Generator
/// 
/// This generator can create PBR materials with procedural textures
/// or generate material configurations for use with the builder.
pub struct PbrMaterialGenerator {
    /// Default configuration for generated materials
    pub default_config: PbrGeneratorConfig,
}

/// Configuration for the PBR material generator
#[derive(Debug, Clone)]
pub struct PbrGeneratorConfig {
    /// Default texture size for generated textures
    pub texture_size: (u32, u32),
    /// Default name prefix for generated materials
    pub name_prefix: String,
    /// Default parameters for generated materials
    pub default_parameters: PbrParameters,
    /// Whether to generate all texture types
    pub generate_all_textures: bool,
}

impl Default for PbrGeneratorConfig {
    fn default() -> Self {
        Self {
            texture_size: (512, 512),
            name_prefix: "generated".to_string(),
            default_parameters: PbrParameters::default(),
            generate_all_textures: true,
        }
    }
}

impl PbrMaterialGenerator {
    /// Create a new PBR material generator
    pub fn new() -> Self {
        Self {
            default_config: PbrGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration
    pub fn with_config(config: PbrGeneratorConfig) -> Self {
        Self { default_config: config }
    }

    /// Generate a complete PBR material configuration
    pub fn generate_material_config(
        &self,
        material_type: PbrMaterialType,
    ) -> PbrMaterialConfig {
        let name_prefix = format!("{}_{}", self.default_config.name_prefix, material_type.prefix());
        
        PbrMaterialConfig {
            name_prefix,
            parameters: self.get_parameters_for_type(material_type),
            ..default()
        }
    }

    /// Generate all textures for a material type
    pub fn generate_textures(
        &self,
        material_type: PbrMaterialType,
        output_dir: &Path,
    ) -> Result<Vec<std::path::PathBuf>, PbrBuilderError> {
        let mut generated_paths = Vec::new();
        
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }
        
        let name_prefix = format!("{}_{}", self.default_config.name_prefix, material_type.prefix());
        
        if self.default_config.generate_all_textures {
            for texture_type in PbrTextureType::all() {
                let texture = self.generate_texture(material_type, *texture_type);
                let filename = format!("{}_{}.png", name_prefix, texture_type.prefix());
                let output_path = output_dir.join(filename);
                
                self.save_texture(&texture, &output_path)?;
                generated_paths.push(output_path);
            }
        }
        
        Ok(generated_paths)
    }

    /// Generate a single texture for a material type
    pub fn generate_texture(
        &self,
        material_type: PbrMaterialType,
        texture_type: PbrTextureType,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let width = self.default_config.texture_size.0;
        let height = self.default_config.texture_size.1;
        
        match texture_type {
            PbrTextureType::BaseColor => {
                self.generate_base_color_texture(material_type, width, height)
            }
            PbrTextureType::MetallicRoughness => {
                self.generate_metallic_roughness_texture(material_type, width, height)
            }
            PbrTextureType::Normal => {
                self.generate_normal_texture(material_type, width, height)
            }
            PbrTextureType::Occlusion => {
                self.generate_occlusion_texture(material_type, width, height)
            }
            PbrTextureType::Emissive => {
                self.generate_emissive_texture(material_type, width, height)
            }
            PbrTextureType::Height => {
                self.generate_height_texture(material_type, width, height)
            }
        }
    }

    /// Generate base color texture
    fn generate_base_color_texture(
        &self,
        material_type: PbrMaterialType,
        width: u32,
        height: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = ImageBuffer::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let color = material_type.base_color();
                let pixel = Rgba([
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    255,
                ]);
                image.put_pixel(x, y, pixel);
            }
        }
        
        image
    }

    /// Generate metallic-roughness texture
    fn generate_metallic_roughness_texture(
        &self,
        material_type: PbrMaterialType,
        width: u32,
        height: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = ImageBuffer::new(width, height);
        let params = self.get_parameters_for_type(material_type);
        
        for y in 0..height {
            for x in 0..width {
                // Metallic in B channel, Roughness in G channel
                let metallic = (params.metallic_factor * 255.0) as u8;
                let roughness = (params.roughness_factor * 255.0) as u8;
                
                // Add some variation based on position
                let variation = self.get_noise_value(x, y, width, height);
                let metallic_var = (metallic as f32 * (0.9 + variation * 0.2)).clamp(0.0, 255.0) as u8;
                let roughness_var = (roughness as f32 * (0.9 + variation * 0.2)).clamp(0.0, 255.0) as u8;
                
                let pixel = Rgba([0, roughness_var, metallic_var, 255]);
                image.put_pixel(x, y, pixel);
            }
        }
        
        image
    }

    /// Generate normal texture
    fn generate_normal_texture(
        &self,
        material_type: PbrMaterialType,
        width: u32,
        height: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = ImageBuffer::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                // Generate normal map from noise
                let nx = self.get_noise_value(x, y, width, height) * 2.0 - 1.0;
                let ny = self.get_noise_value(x + 100, y + 100, width, height) * 2.0 - 1.0;
                
                // Convert to 0-255 range and center at 128
                let nx_8 = ((nx * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                let ny_8 = ((ny * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                
                let pixel = Rgba([nx_8, ny_8, 255, 255]);
                image.put_pixel(x, y, pixel);
            }
        }
        
        image
    }

    /// Generate occlusion texture
    fn generate_occlusion_texture(
        &self,
        material_type: PbrMaterialType,
        width: u32,
        height: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = ImageBuffer::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                // Generate occlusion based on noise (darker in corners/edges)
                let dist_x = (x as f32 / width as f32).abs();
                let dist_y = (y as f32 / height as f32).abs();
                let edge_factor = 1.0 - (dist_x * dist_y).sqrt();
                
                let noise = self.get_noise_value(x, y, width, height);
                let occlusion = ((edge_factor * 0.7 + noise * 0.3) * 255.0).clamp(0.0, 255.0) as u8;
                
                let pixel = Rgba([occlusion, occlusion, occlusion, 255]);
                image.put_pixel(x, y, pixel);
            }
        }
        
        image
    }

    /// Generate emissive texture
    fn generate_emissive_texture(
        &self,
        material_type: PbrMaterialType,
        width: u32,
        height: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = ImageBuffer::new(width, height);
        let emissive_color = material_type.emissive_color();
        
        for y in 0..height {
            for x in 0..width {
                let intensity = if material_type.has_emission() {
                    (self.get_noise_value(x, y, width, height) * 0.5 + 0.5) * 255.0
                } else {
                    0.0
                };
                
                let pixel = Rgba([
                    (emissive_color[0] * intensity.clamp(0.0, 255.0)) as u8,
                    (emissive_color[1] * intensity.clamp(0.0, 255.0)) as u8,
                    (emissive_color[2] * intensity.clamp(0.0, 255.0)) as u8,
                    255,
                ]);
                image.put_pixel(x, y, pixel);
            }
        }
        
        image
    }

    /// Generate height texture
    fn generate_height_texture(
        &self,
        material_type: PbrMaterialType,
        width: u32,
        height: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = ImageBuffer::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let height_value = (self.get_noise_value(x, y, width, height) * 255.0).clamp(0.0, 255.0) as u8;
                let pixel = Rgba([height_value, height_value, height_value, 255]);
                image.put_pixel(x, y, pixel);
            }
        }
        
        image
    }

    /// Save texture to file
    fn save_texture(
        &self,
        texture: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        output_path: &Path,
    ) -> Result<(), PbrBuilderError> {
        use image::io::Writer as ImageWriter;
        
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        texture.write_to(
            &mut std::fs::File::create(output_path).map_err(|e| {
                PbrBuilderError::ImageSaveError(e.to_string())
            })?,
            image::ImageOutputFormat::Png,
        ).map_err(|e| PbrBuilderError::ImageSaveError(e.to_string()))?;
        
        Ok(())
    }

    /// Get parameters for a specific material type
    fn get_parameters_for_type(&self, material_type: PbrMaterialType) -> PbrParameters {
        match material_type {
            PbrMaterialType::Gold => PbrParameters {
                base_color_factor: [1.0, 0.75, 0.33, 1.0],
                metallic_factor: 1.0,
                roughness_factor: 0.1,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::Silver => PbrParameters {
                base_color_factor: [0.9, 0.9, 0.9, 1.0],
                metallic_factor: 1.0,
                roughness_factor: 0.05,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::Copper => PbrParameters {
                base_color_factor: [0.95, 0.6, 0.4, 1.0],
                metallic_factor: 1.0,
                roughness_factor: 0.15,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::RustyMetal => PbrParameters {
                base_color_factor: [0.7, 0.3, 0.1, 1.0],
                metallic_factor: 0.8,
                roughness_factor: 0.6,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::PlasticRed => PbrParameters {
                base_color_factor: [0.8, 0.2, 0.2, 1.0],
                metallic_factor: 0.0,
                roughness_factor: 0.5,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::PlasticBlue => PbrParameters {
                base_color_factor: [0.2, 0.2, 0.8, 1.0],
                metallic_factor: 0.0,
                roughness_factor: 0.4,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::Ceramic => PbrParameters {
                base_color_factor: [0.7, 0.7, 0.8, 1.0],
                metallic_factor: 0.0,
                roughness_factor: 0.7,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::Glass => PbrParameters {
                base_color_factor: [0.9, 0.9, 1.0, 0.5],
                metallic_factor: 0.0,
                roughness_factor: 0.0,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::Wood => PbrParameters {
                base_color_factor: [0.6, 0.4, 0.2, 1.0],
                metallic_factor: 0.0,
                roughness_factor: 0.8,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::Stone => PbrParameters {
                base_color_factor: [0.5, 0.5, 0.5, 1.0],
                metallic_factor: 0.0,
                roughness_factor: 0.9,
                ..self.default_config.default_parameters.clone()
            },
            PbrMaterialType::Glowing => PbrParameters {
                base_color_factor: [0.3, 0.8, 0.3, 1.0],
                emissive_factor: [0.5, 1.0, 0.5],
                metallic_factor: 0.0,
                roughness_factor: 0.3,
                ..self.default_config.default_parameters.clone()
            },
        }
    }

    /// Simple noise function for procedural texture generation
    fn get_noise_value(&self, x: u32, y: u32, width: u32, height: u32) -> f32 {
        // Simple pseudo-random noise based on position
        let nx = x as f32 / width as f32;
        let ny = y as f32 / height as f32;
        
        // Use a simple hash function
        let seed = (x * 12345 + y * 67891) % 100000;
        let hash = ((seed as f32).sin() * 10000.0).fract();
        
        // Combine with position for better distribution
        (nx * 0.1 + ny * 0.1 + hash).fract()
    }
}

/// Types of PBR materials that can be generated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbrMaterialType {
    Gold,
    Silver,
    Copper,
    RustyMetal,
    PlasticRed,
    PlasticBlue,
    Ceramic,
    Glass,
    Wood,
    Stone,
    Glowing,
}

impl PbrMaterialType {
    /// Get the prefix for this material type
    pub fn prefix(&self) -> &'static str {
        match self {
            PbrMaterialType::Gold => "gold",
            PbrMaterialType::Silver => "silver",
            PbrMaterialType::Copper => "copper",
            PbrMaterialType::RustyMetal => "rusty_metal",
            PbrMaterialType::PlasticRed => "plastic_red",
            PbrMaterialType::PlasticBlue => "plastic_blue",
            PbrMaterialType::Ceramic => "ceramic",
            PbrMaterialType::Glass => "glass",
            PbrMaterialType::Wood => "wood",
            PbrMaterialType::Stone => "stone",
            PbrMaterialType::Glowing => "glowing",
        }
    }

    /// Get the base color for this material type
    pub fn base_color(&self) -> [f32; 3] {
        match self {
            PbrMaterialType::Gold => [1.0, 0.75, 0.33],
            PbrMaterialType::Silver => [0.9, 0.9, 0.9],
            PbrMaterialType::Copper => [0.95, 0.6, 0.4],
            PbrMaterialType::RustyMetal => [0.7, 0.3, 0.1],
            PbrMaterialType::PlasticRed => [0.8, 0.2, 0.2],
            PbrMaterialType::PlasticBlue => [0.2, 0.2, 0.8],
            PbrMaterialType::Ceramic => [0.7, 0.7, 0.8],
            PbrMaterialType::Glass => [0.9, 0.9, 1.0],
            PbrMaterialType::Wood => [0.6, 0.4, 0.2],
            PbrMaterialType::Stone => [0.5, 0.5, 0.5],
            PbrMaterialType::Glowing => [0.3, 0.8, 0.3],
        }
    }

    /// Get the emissive color for this material type
    pub fn emissive_color(&self) -> [f32; 3] {
        match self {
            PbrMaterialType::Glowing => [0.5, 1.0, 0.5],
            _ => [0.0, 0.0, 0.0],
        }
    }

    /// Check if this material type has emission
    pub fn has_emission(&self) -> bool {
        matches!(self, PbrMaterialType::Glowing)
    }

    /// Get all material types
    pub fn all() -> &'static [PbrMaterialType] {
        &[
            PbrMaterialType::Gold,
            PbrMaterialType::Silver,
            PbrMaterialType::Copper,
            PbrMaterialType::RustyMetal,
            PbrMaterialType::PlasticRed,
            PbrMaterialType::PlasticBlue,
            PbrMaterialType::Ceramic,
            PbrMaterialType::Glass,
            PbrMaterialType::Wood,
            PbrMaterialType::Stone,
            PbrMaterialType::Glowing,
        ]
    }
}

impl Default for PbrMaterialGenerator {
    fn default() -> Self {
        Self::new()
    }
}
