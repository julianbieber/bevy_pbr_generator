//! Selection of the per-material texture functions, and the packing of their
//! outputs into the channel layout Bevy's `StandardMaterial` expects.
//!
//! The layout each function commits to is derived in
//! BEVY_STANDARD_MATERIAL_TEXTURE_PACKING.md.

use crate::config::TextureType;
use crate::textures;
use glam::Vec2;

/// Base colour in RGB, opacity in A. sRGB.
pub fn base_color(uv: Vec2, texture_type: TextureType) -> glam::Vec4 {
    match texture_type {
        TextureType::Water => textures::base_color_texture(uv),
        TextureType::Rocky => textures::rocky_base_color_texture(uv),
    }
}

/// Tangent-space normal in RGB, each component remapped from `[-1.0, 1.0]` to
/// `[0.0, 1.0]`. Linear.
pub fn normal(uv: Vec2, texture_type: TextureType) -> glam::Vec3 {
    match texture_type {
        TextureType::Water => textures::normal_map_texture(uv),
        TextureType::Rocky => textures::rocky_normal_map_texture(uv),
    }
}

/// Occlusion in R, roughness in G, metallic in B. Linear.
pub fn orm(uv: Vec2, texture_type: TextureType) -> glam::Vec3 {
    match texture_type {
        TextureType::Water => {
            let occlusion = textures::occlusion_texture(uv);
            let mr = textures::metallic_roughness_texture(uv);
            glam::Vec3::new(occlusion, mr.x, mr.y)
        }
        TextureType::Rocky => {
            let occlusion = textures::rocky_occlusion_texture(uv);
            let mr = textures::rocky_metallic_roughness_texture(uv);
            glam::Vec3::new(occlusion, mr.x, mr.y)
        }
    }
}

/// Emissive colour in RGB. sRGB.
pub fn emissive(uv: Vec2, texture_type: TextureType) -> glam::Vec3 {
    match texture_type {
        TextureType::Water => textures::emissive_texture(uv),
        TextureType::Rocky => textures::rocky_emissive_texture(uv),
    }
}

/// Specular transmission in R, thickness in G, diffuse transmission in A; B is
/// unused and always 0.0. Linear.
pub fn transmission(uv: Vec2, texture_type: TextureType) -> glam::Vec4 {
    match texture_type {
        TextureType::Water => {
            let specular_trans = textures::specular_transmission_texture(uv);
            let thickness = textures::thickness_texture(uv);
            let diffuse_trans = textures::diffuse_transmission_texture(uv);
            glam::Vec4::new(specular_trans, thickness, 0.0, diffuse_trans)
        }
        TextureType::Rocky => {
            let specular_trans = textures::rocky_specular_transmission_texture(uv);
            let thickness = textures::rocky_thickness_texture(uv);
            let diffuse_trans = textures::rocky_diffuse_transmission_texture(uv);
            glam::Vec4::new(specular_trans, thickness, 0.0, diffuse_trans)
        }
    }
}

/// Specular tint in RGB (sRGB), specular strength in A (linear).
pub fn specular(uv: Vec2, texture_type: TextureType) -> glam::Vec4 {
    match texture_type {
        TextureType::Water => {
            let tint = textures::specular_tint_texture(uv);
            let specular = textures::specular_texture(uv);
            glam::Vec4::new(tint.x, tint.y, tint.z, specular)
        }
        TextureType::Rocky => {
            let tint = textures::rocky_specular_tint_texture(uv);
            let specular = textures::rocky_specular_texture(uv);
            glam::Vec4::new(tint.x, tint.y, tint.z, specular)
        }
    }
}

/// Clearcoat strength in R, clearcoat roughness in G. Linear.
pub fn clearcoat(uv: Vec2, texture_type: TextureType) -> glam::Vec2 {
    match texture_type {
        TextureType::Water => {
            let clearcoat = textures::clearcoat_texture(uv);
            let roughness = textures::clearcoat_roughness_texture(uv);
            glam::Vec2::new(clearcoat, roughness)
        }
        TextureType::Rocky => {
            let clearcoat = textures::rocky_clearcoat_texture(uv);
            let roughness = textures::rocky_clearcoat_roughness_texture(uv);
            glam::Vec2::new(clearcoat, roughness)
        }
    }
}

/// Tangent-space clearcoat normal in RGB, remapped to `[0.0, 1.0]`. Linear.
pub fn clearcoat_normal(uv: Vec2, texture_type: TextureType) -> glam::Vec3 {
    match texture_type {
        TextureType::Water => textures::clearcoat_normal_texture(uv),
        TextureType::Rocky => textures::rocky_clearcoat_normal_texture(uv),
    }
}

/// Anisotropy direction in RG, strength in B. Linear.
pub fn anisotropy(uv: Vec2, texture_type: TextureType) -> glam::Vec3 {
    match texture_type {
        TextureType::Water => textures::anisotropy_texture(uv),
        TextureType::Rocky => textures::rocky_anisotropy_texture(uv),
    }
}

/// Parallax depth, written to R. Linear.
pub fn depth(uv: Vec2, texture_type: TextureType) -> f32 {
    match texture_type {
        TextureType::Water => textures::depth_map(uv),
        TextureType::Rocky => textures::rocky_depth_map(uv),
    }
}
