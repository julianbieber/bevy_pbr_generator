//! Per-texture generation functions for Bevy PBR materials.
//!
//! Each function takes normalized UV coordinates (Vec2 in [0.0, 1.0]) and returns a Vec3.
//! Users can edit these functions to customize texture generation.

use glam::{Vec2, Vec3, Vec4};

/// Base color texture (sRGB).
/// Bevy: RGB = color, A = opacity
pub fn base_color_texture(_uv: Vec2) -> Vec4 {
    Vec4::ONE
}

/// Emissive texture (sRGB).
/// Bevy: RGB = emissive color.
pub fn emissive_texture(_uv: Vec2) -> Vec3 {
    Vec3::ONE
}

/// Metallic-roughness texture (linear).
/// Bevy: B(y) = metallic, G(x) = roughness.
pub fn metallic_roughness_texture(_uv: Vec2) -> Vec2 {
    Vec2::new(0.5, 0.0)
}

/// Diffuse transmission texture (linear).
/// Default: 0.0 (no transmission).
/// Bevy: A = transmission.
pub fn diffuse_transmission_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Specular transmission texture (linear).
/// Default: 0.0 (no transmission).
/// Bevy: R = transmission.
pub fn specular_transmission_texture(_uv: Vec2) -> f32 {
    0.0 // x = transmission (stored in red channel)
}

/// Thickness texture (linear).
/// Default: x = 0.0 (no thickness).
/// Bevy: G = thickness.
pub fn thickness_texture(_uv: Vec2) -> Vec2 {
    // Default: no thickness
    Vec2::new(0.0, 0.0) // x = thickness (stored in green channel)
}

// ========== Example implementations (uncomment to use) ==========

// Example: Perlin noise-based base color.
// pub fn base_color_texture(uv: Vec2) -> Vec2 {
//     let noise = perlin_noise(uv, 5.0);
//     let r = (noise + 1.0) / 2.0; // Map from [-1, 1] to [0, 1]
//     Vec2::new(r, r * 0.5)
// }

// Example: Worley noise-based metallic-roughness.
// pub fn metallic_roughness_texture(uv: Vec2) -> Vec2 {
//     let noise = worley_noise(uv, 10.0, 30);
//     let roughness = noise.clamp(0.0, 1.0);
//     let metallic = (perlin_noise(uv, 2.0) + 1.0) / 2.0;
//     Vec2::new(roughness, metallic) // x = roughness, y = metallic
// }

// Example: Combined noise for emissive texture.
// pub fn emissive_texture(uv: Vec2) -> Vec2 {
//     let noise = combined_noise(uv);
//     let intensity = (noise + 1.0) / 2.0;
//     Vec2::new(intensity * 0.1, intensity * 0.05)
// }
