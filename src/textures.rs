//! Per-texture generation functions for Bevy PBR materials.
//!
//! Each function takes normalized UV coordinates (Vec2 in [0.0, 1.0]) and returns
//! its natural type (f32, Vec2, Vec3, or Vec4) based on the PBR property requirements.
//! Users can edit these functions to customize texture generation.

use glam::{Vec2, Vec3, Vec4};

/// Base color texture (sRGB).
/// Bevy: RGB = color, A = opacity
pub fn base_color_texture(_uv: Vec2) -> Vec4 {
    Vec4::new(1.0, 1.0, 1.0, 1.0)
}

/// Emissive texture (sRGB).
/// Bevy: RGB = emissive color.
pub fn emissive_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(1.0, 1.0, 1.0)
}

/// Metallic-roughness texture (linear).
/// Bevy: G = roughness, B = metallic.
/// Returns Vec2 where x = roughness, y = metallic.
pub fn metallic_roughness_texture(_uv: Vec2) -> Vec2 {
    Vec2::new(0.5, 0.0)
}

/// Normal map texture (linear).
/// Bevy: RGB = normal vector (tangent space).
pub fn normal_map_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.5, 0.5, 1.0) // Flat normal in tangent space
}

/// Occlusion texture (linear).
/// Bevy: R = occlusion.
pub fn occlusion_texture(_uv: Vec2) -> f32 {
    1.0 // Default: full occlusion
}

/// Specular texture (linear).
/// Bevy: A = specular.
pub fn specular_texture(_uv: Vec2) -> f32 {
    0.0 // Default: no specular
}

/// Specular tint texture (sRGB).
/// Bevy: RGB = specular tint color.
pub fn specular_tint_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(1.0, 1.0, 1.0) // Default: white tint
}

/// Clearcoat texture (linear).
/// Bevy: R = clearcoat.
pub fn clearcoat_texture(_uv: Vec2) -> f32 {
    0.0 // Default: no clearcoat
}

/// Clearcoat roughness texture (linear).
/// Bevy: G = clearcoat roughness.
pub fn clearcoat_roughness_texture(_uv: Vec2) -> f32 {
    0.0 // Default: smooth clearcoat
}

/// Clearcoat normal texture (linear).
/// Bevy: RGB = clearcoat normal vector.
pub fn clearcoat_normal_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.5, 0.5, 1.0) // Default: flat normal
}

/// Anisotropy texture (linear).
/// Bevy: RG = anisotropy direction, B = anisotropy strength.
pub fn anisotropy_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.0, 0.0, 0.0) // Default: no anisotropy (dir_x, dir_y, strength)
}

/// Depth map / Parallax texture (linear).
/// Bevy: R = parallax depth.
pub fn depth_map(_uv: Vec2) -> f32 {
    0.0 // Default: no parallax
}

/// Diffuse transmission texture (linear).
/// Bevy: A = transmission.
pub fn diffuse_transmission_texture(_uv: Vec2) -> f32 {
    0.0 // Default: no transmission
}

/// Specular transmission texture (linear).
/// Bevy: R = transmission.
pub fn specular_transmission_texture(_uv: Vec2) -> f32 {
    0.0 // Default: no transmission
}

/// Thickness texture (linear).
/// Bevy: G = thickness.
pub fn thickness_texture(_uv: Vec2) -> f32 {
    0.0 // Default: no thickness
}

// ========== Example implementations (uncomment to use) ==========

// Example: Perlin noise-based base color.
// pub fn base_color_texture(uv: Vec2) -> Vec4 {
//     let noise = crate::noise::perlin_noise(uv, 5.0);
//     let r = (noise + 1.0) / 2.0; // Map from [-1, 1] to [0, 1]
//     Vec4::new(r, r * 0.5, 0.0, 1.0)
// }

// Example: Worley noise-based metallic-roughness.
// pub fn metallic_roughness_texture(uv: Vec2) -> Vec2 {
//     let noise = crate::noise::worley_noise(uv, 10.0, 30);
//     let roughness = noise.clamp(0.0, 1.0);
//     let metallic = (crate::noise::perlin_noise(uv, 2.0) + 1.0) / 2.0;
//     Vec2::new(roughness, metallic) // x = roughness, y = metallic
// }

// Example: Combined noise for emissive texture.
// pub fn emissive_texture(uv: Vec2) -> Vec3 {
//     let noise = crate::noise::combined_noise(uv);
//     let intensity = (noise + 1.0) / 2.0;
//     Vec3::new(intensity * 0.1, intensity * 0.05, 0.0)
// }
