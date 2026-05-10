//! PBR Material Module
use bevy::prelude::*;
use bevy::render::render_resource::ShaderRef;
use std::collections::HashMap;
use super::{PbrMaterialConfig, PbrParameters, PbrTextureType};

/// Custom PBR Material
#[derive(Asset, TypePath, Debug, Clone)]
pub struct PbrMaterial {
    pub config: PbrMaterialConfig,
    pub textures: HashMap<PbrTextureType, Handle<Image>>,
    pub shaders: HashMap<PbrTextureType, ShaderRef>,
    pub parameters: PbrParameters,
    pub built: bool,
}

impl PbrMaterial {
    pub fn new(config: &PbrMaterialConfig) -> Self {
        Self {
            config: config.clone(),
            textures: HashMap::new(),
            shaders: HashMap::new(),
            parameters: config.parameters.clone(),
            built: false,
        }
    }

    pub fn set_texture(&mut self, texture_type: PbrTextureType, handle: Handle<Image>) {
        self.textures.insert(texture_type, handle);
    }

    pub fn set_shader(&mut self, texture_type: PbrTextureType, shader: ShaderRef) {
        self.shaders.insert(texture_type, shader);
    }

    pub fn get_shader(&self, texture_type: PbrTextureType) -> ShaderRef {
        self.shaders.get(&texture_type).cloned().unwrap_or_else(|| ShaderRef::Default)
    }

    pub fn get_texture(&self, texture_type: PbrTextureType) -> Option<&Handle<Image>> {
        self.textures.get(&texture_type)
    }

    pub fn mark_built(&mut self) {
        self.built = true;
    }

    pub fn is_built(&self) -> bool {
        self.built
    }

    pub fn active_texture_types(&self) -> Vec<PbrTextureType> {
        self.textures.keys().cloned().collect()
    }

    pub fn custom_shader_types(&self) -> Vec<PbrTextureType> {
        self.shaders.keys().cloned().collect()
    }

    pub fn has_texture(&self, texture_type: PbrTextureType) -> bool {
        self.textures.get(&texture_type).is_some()
    }

    pub fn has_shader(&self, texture_type: PbrTextureType) -> bool {
        self.shaders.get(&texture_type).is_some()
    }
}
