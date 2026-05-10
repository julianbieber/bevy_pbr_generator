//! PBR Material Exporter
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use std::path::Path;
use super::{PbrBuilderError, PbrMaterialConfig, PbrTextureType};

/// PBR Material Exporter
pub struct PbrMaterialExporter {
    config: PbrMaterialConfig,
}

impl PbrMaterialExporter {
    pub fn new(config: PbrMaterialConfig) -> Self {
        Self { config }
    }

    pub fn export_image(&self, image: &Image, output_path: &Path) -> Result<(), PbrBuilderError> {
        use image::io::Writer as ImageWriter;
        use image::{ImageBuffer, Rgba};

        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let width = image.size.x as u32;
        let height = image.size.y as u32;

        let pixels: Vec<Rgba<u8>> = match &image.data {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8Srgb => {
                image.data.chunks(4).map(|chunk| Rgba([chunk[0], chunk[1], chunk[2], chunk[3]])).collect()
            }
            TextureFormat::Rgba16Unorm => {
                image.data.chunks(8).map(|chunk| {
                    let r = (u16::from_le_bytes([chunk[0], chunk[1]]) >> 8) as u8;
                    let g = (u16::from_le_bytes([chunk[2], chunk[3]]) >> 8) as u8;
                    let b = (u16::from_le_bytes([chunk[4], chunk[5]]) >> 8) as u8;
                    let a = (u16::from_le_bytes([chunk[6], chunk[7]]) >> 8) as u8;
                    Rgba([r, g, b, a])
                }).collect()
            }
            TextureFormat::Rgba32Float => {
                image.data.chunks(16).map(|chunk| {
                    let r = (f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]).clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]).clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]).clamp(0.0, 1.0) * 255.0) as u8;
                    let a = (f32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]).clamp(0.0, 1.0) * 255.0) as u8;
                    Rgba([r, g, b, a])
                }).collect()
            }
            _ => return Err(PbrBuilderError::ImageSaveError(format!("Unsupported texture format: {:?}", image.texture_format))),
        };

        let img_buffer = ImageBuffer::from_vec(width, height, pixels)
            .ok_or_else(|| PbrBuilderError::ImageSaveError("Failed to create image buffer".to_string()))?;

        img_buffer.write_to(
            &mut std::fs::File::create(output_path).map_err(|e| PbrBuilderError::ImageSaveError(e.to_string()))?,
            image::ImageOutputFormat::Png,
        ).map_err(|e| PbrBuilderError::ImageSaveError(e.to_string()))?;

        Ok(())
    }

    pub fn export_all_textures(
        &self,
        textures: &std::collections::HashMap<PbrTextureType, Handle<Image>>,
        images: &Assets<Image>,
        output_dir: &Path,
    ) -> Result<Vec<std::path::PathBuf>, PbrBuilderError> {
        let mut exported_paths = Vec::new();
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }
        for (texture_type, handle) in textures {
            if let Some(image) = images.get(handle) {
                let filename = format!("{}_{}.png", self.config.name_prefix, texture_type.prefix());
                let output_path = output_dir.join(filename);
                self.export_image(image, &output_path)?;
                exported_paths.push(output_path);
            }
        }
        Ok(exported_paths)
    }
}
