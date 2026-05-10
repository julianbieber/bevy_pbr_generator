//! Bevy PBR Material Builder Library
//!
//! This library provides a comprehensive PBR (Physically Based Rendering) material builder
//! for Bevy 0.18 that uses WGSL (WebGPU Shading Language) for shader programming.
//!
//! # Features
//!
//! - **Shader Library**: Comprehensive library of WGSL shader functions for PBR rendering
//! - **Material Builder**: Builder pattern for creating PBR materials with multiple textures
//! - **Texture Export**: Export individual texture images with name prefixes
//! - **Multiple Shaders**: Support for different shaders per texture type
//! - **Material Generator**: Procedural PBR material and texture generation
//! - **Bevy Integration**: Full integration with Bevy's ECS and render pipeline
//!
//! # Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_pbr_builder::*;
//!
//! fn setup_pbr_material(
//!     mut commands: Commands,
//!     asset_server: Res<AssetServer>,
//!     mut meshes: ResMut<Assets<Mesh>>,
//!     mut materials: ResMut<Assets<PbrMaterial>>,
//! ) {
//!     // Create PBR material configuration
//!     let config = PbrMaterialConfig {
//!         name_prefix: "gold_material".to_string(),
//!         parameters: PbrParameters {
//!             base_color_factor: [1.0, 0.75, 0.33, 1.0], // Gold color
//!             metallic_factor: 1.0,
//!             roughness_factor: 0.1,
//!             ..default()
//!         },
//!         ..default()
//!     };
//!
//!     // Create PBR material builder
//!     let builder = PbrMaterialBuilder::new(config);
//!
//!     // Load textures
//!     let base_color_texture = asset_server.load("textures/base_color.png");
//!     let metallic_roughness_texture = asset_server.load("textures/metallic_roughness.png");
//!
//!     // Add textures to builder
//!     let builder = builder
//!         .with_texture(PbrTextureType::BaseColor, base_color_texture)
//!         .with_texture(PbrTextureType::MetallicRoughness, metallic_roughness_texture);
//!
//!     // Build the material
//!     let material = builder.build(&asset_server, &mut materials).unwrap();
//!
//!     // Use the material
//!     commands.spawn(PbrBundle {
//!         mesh: meshes.add(Mesh::from(shape::Cube::default())),
//!         material: materials.add(material),
//!         ..default()
//!     });
//! }
//!
//! // Generate procedural materials
//! fn generate_materials() {
//!     let generator = PbrMaterialGenerator::new();
//!     
//!     // Generate a gold material configuration
//!     let gold_config = generator.generate_material_config(PbrMaterialType::Gold);
//!     
//!     // Generate and save textures
//!     let textures = generator.generate_textures(PbrMaterialType::Gold, Path::new("generated")).unwrap();
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::needless_doctest_main)]

pub mod pbr_builder;

pub use pbr_builder::*;

/// Re-export commonly used types
pub use pbr_builder::{
    PbrBuilderError, PbrMaterial, PbrMaterialBuilder, PbrMaterialBuilderPlugin,
    PbrMaterialConfig, PbrMaterialExporter, PbrMaterialGenerator, PbrMaterialType,
    PbrGeneratorConfig, PbrParameters, PbrShaderLibrary, PbrTextureType,
};
