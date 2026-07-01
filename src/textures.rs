//! Per-texture generation functions for Bevy PBR materials.
//!
//! Each function takes normalized UV coordinates (Vec2 in [0.0, 1.0]) and returns
//! its natural type (f32, Vec2, Vec3, or Vec4) based on the PBR property requirements.
//! Users can edit these functions to customize texture generation.
//!
//! # Water Texture Theme
//!
//! The default implementations create water-like textures using layered noise.
//! Water parameters can be tuned by adjusting the constants at the top of each function.

use glam::{Vec2, Vec3, Vec4};

// ========== Water Texture Parameters ==========

/// Deep ocean blue - the primary water color
const WATER_DEEP_COLOR: Vec3 = Vec3::new(0.02, 0.15, 0.40);
/// Shallow water color with teal/green tint
const WATER_SHALLOW_COLOR: Vec3 = Vec3::new(0.05, 0.35, 0.55);
/// Wave crest color (foam/white caps)
const WATER_CREST_COLOR: Vec3 = Vec3::new(0.7, 0.85, 0.95);

/// Scale for large wave patterns
const WAVE_SCALE: f32 = 8.0;
/// Scale for small ripples
const RIPPLE_SCALE: f32 = 25.0;
/// Frequency multiplier for wave distortion
const WAVE_FREQUENCY: f32 = 2.0;
/// Number of wave octaves for detail
const WAVE_OCTAVES: usize = 3;

/// Base roughness for calm water
const WATER_BASE_ROUGHNESS: f32 = 0.15;
/// Roughness increase at wave crests
const WATER_CREST_ROUGHNESS: f32 = 0.40;

/// Base transmission for clear water
const WATER_DIFFUSE_TRANSMISSION: f32 = 0.90;
/// Specular transmission amount
const WATER_SPECULAR_TRANSMISSION: f32 = 0.50;
/// Water thickness (affects absorption)
const WATER_THICKNESS: f32 = 0.10;

/// Base emissive intensity for subtle glow
const WATER_EMISSIVE_INTENSITY: f32 = 0.05;

// ========== Water Texture Functions ==========

/// Base color texture (sRGB).
/// Bevy: RGB = color, A = opacity
///
/// Creates water color with:
/// - Deep blue base
/// - Shallow water variation via noise
/// - Wave crest highlights
/// - Subtle depth-based color blending
pub fn base_color_texture(uv: Vec2) -> Vec4 {
    // Large-scale depth variation
    let depth_noise = crate::noise::perlin_noise(uv, WAVE_SCALE * 0.5);
    let depth_factor = (depth_noise + 1.0) / 2.0; // [0, 1]

    // Small-scale ripple detail
    let ripple_noise = crate::noise::perlin_noise(uv, RIPPLE_SCALE);
    let ripple_factor = (ripple_noise + 1.0) / 2.0;

    // Wave crest detection (high frequency noise for foam)
    let crest_noise = crate::noise::simplex_noise(uv * WAVE_SCALE, WAVE_FREQUENCY);
    let crest_factor = ((crest_noise + 1.0) / 2.0).powi(2); // Sharper crests

    // Blend between deep and shallow water based on depth
    let base_color = WATER_DEEP_COLOR.lerp(WATER_SHALLOW_COLOR, depth_factor);

    // Add crest highlights
    let crest_color = WATER_CREST_COLOR.lerp(base_color, 0.7);
    let final_color = base_color.lerp(crest_color, crest_factor * 0.3);

    // Add subtle ripple variation
    let ripple_variation = Vec3::splat(0.02) * (ripple_factor - 0.5);

    Vec4::new(
        (final_color.x + ripple_variation.x).clamp(0.0, 1.0),
        (final_color.y + ripple_variation.y).clamp(0.0, 1.0),
        (final_color.z + ripple_variation.z).clamp(0.0, 1.0),
        1.0, // Full opacity
    )
}

/// Emissive texture (sRGB).
/// Bevy: RGB = emissive color.
///
/// Creates subtle blue-cyan glow, stronger at wave crests
/// (simulating bioluminescence or light refraction).
pub fn emissive_texture(uv: Vec2) -> Vec3 {
    // Use wave crest noise for emissive variation
    let crest_noise = crate::noise::simplex_noise(uv * WAVE_SCALE, WAVE_FREQUENCY);
    let crest_factor = ((crest_noise + 1.0) / 2.0).powi(2);

    // Deep water glow (bioluminescence effect)
    let deep_glow = Vec3::new(0.1, 0.3, 0.6) * WATER_EMISSIVE_INTENSITY;

    // Crest glow (foam/moonlight reflection)
    let crest_glow = Vec3::new(0.7, 0.85, 0.95) * WATER_EMISSIVE_INTENSITY * 1.5;

    // Blend based on crest factor
    deep_glow.lerp(crest_glow, crest_factor)
}

/// Metallic-roughness texture (linear).
/// Bevy: G = roughness, B = metallic.
/// Returns Vec2 where x = roughness, y = metallic.
///
/// Water is non-metallic (dielectric) with variable roughness.
/// Roughness increases at wave crests and with surface disturbance.
pub fn metallic_roughness_texture(uv: Vec2) -> Vec2 {
    // Base roughness from wave patterns
    let wave_noise = crate::noise::fbm(
        uv,
        WAVE_SCALE,
        WAVE_OCTAVES,
        2.0,
        0.5,
        crate::noise::perlin_noise,
    );
    let wave_factor = (wave_noise + 1.0) / 2.0; // [0, 1]

    // Crest roughness (foam areas are rougher)
    let crest_noise = crate::noise::simplex_noise(uv * WAVE_SCALE * 1.5, WAVE_FREQUENCY * 2.0);
    let crest_factor = ((crest_noise + 1.0) / 2.0).powi(3); // Very sharp crests

    // Calculate roughness: base + wave variation + crest boost
    let roughness = WATER_BASE_ROUGHNESS
        + (WATER_CREST_ROUGHNESS - WATER_BASE_ROUGHNESS) * crest_factor
        + (WATER_CREST_ROUGHNESS - WATER_BASE_ROUGHNESS) * wave_factor * 0.3;

    // Water is non-metallic
    Vec2::new(roughness.clamp(0.0, 1.0), 0.0)
}

/// Normal map texture (linear).
/// Bevy: RGB = normal vector (tangent space).
///
/// Creates water surface normals with:
/// - Large wave undulations
/// - Small ripples
/// - Directional wave patterns
pub fn normal_map_texture(uv: Vec2) -> Vec3 {
    // Normal mapping: calculate surface normal from height map
    const EPSILON: f32 = 1.0 / 1024.0;

    // Sample height at neighboring points
    let height_x_plus = crate::noise::perlin_noise(uv + Vec2::new(EPSILON, 0.0), WAVE_SCALE * 0.8)
        * 0.5
        + crate::noise::perlin_noise(uv + Vec2::new(EPSILON, 0.0), WAVE_SCALE * 1.5) * 0.3
        + crate::noise::perlin_noise(uv + Vec2::new(EPSILON, 0.0), RIPPLE_SCALE) * 0.2
        + ((uv.x + EPSILON) * WAVE_SCALE * 0.5 + uv.y * WAVE_SCALE * 0.5).sin() * 0.15;

    let height_x_minus =
        crate::noise::perlin_noise(uv + Vec2::new(-EPSILON, 0.0), WAVE_SCALE * 0.8) * 0.5
            + crate::noise::perlin_noise(uv + Vec2::new(-EPSILON, 0.0), WAVE_SCALE * 1.5) * 0.3
            + crate::noise::perlin_noise(uv + Vec2::new(-EPSILON, 0.0), RIPPLE_SCALE) * 0.2
            + ((uv.x - EPSILON) * WAVE_SCALE * 0.5 + uv.y * WAVE_SCALE * 0.5).sin() * 0.15;

    let height_y_plus = crate::noise::perlin_noise(uv + Vec2::new(0.0, EPSILON), WAVE_SCALE * 0.8)
        * 0.5
        + crate::noise::perlin_noise(uv + Vec2::new(0.0, EPSILON), WAVE_SCALE * 1.5) * 0.3
        + crate::noise::perlin_noise(uv + Vec2::new(0.0, EPSILON), RIPPLE_SCALE) * 0.2
        + (uv.x * WAVE_SCALE * 0.5 + (uv.y + EPSILON) * WAVE_SCALE * 0.5).sin() * 0.15;

    let height_y_minus =
        crate::noise::perlin_noise(uv + Vec2::new(0.0, -EPSILON), WAVE_SCALE * 0.8) * 0.5
            + crate::noise::perlin_noise(uv + Vec2::new(0.0, -EPSILON), WAVE_SCALE * 1.5) * 0.3
            + crate::noise::perlin_noise(uv + Vec2::new(0.0, -EPSILON), RIPPLE_SCALE) * 0.2
            + (uv.x * WAVE_SCALE * 0.5 + (uv.y - EPSILON) * WAVE_SCALE * 0.5).sin() * 0.15;

    // Calculate derivatives
    let dzdx = (height_x_plus - height_x_minus) / (2.0 * EPSILON);
    let dzdy = (height_y_plus - height_y_minus) / (2.0 * EPSILON);

    // Calculate normal in world space
    let normal_ws = Vec3::new(-dzdx, -dzdy, 1.0).normalize();

    // Convert to tangent space (for normal maps, +X = right, +Y = up, +Z = out of surface)
    // For a simple water surface, we assume standard tangent space
    // N.x (red) = +X component, N.y (green) = -Y component, N.z (blue) = Z component
    // Then remap from [-1, 1] to [0, 1]
    Vec3::new(
        (normal_ws.x * 0.5) + 0.5,
        (-normal_ws.y * 0.5) + 0.5,
        normal_ws.z * 0.5 + 0.5,
    )
}

/// Occlusion texture (linear).
/// Bevy: R = occlusion.
///
/// Water surface has high occlusion (is transparent/clear).
/// Subtle variation based on wave shadows.
pub fn occlusion_texture(uv: Vec2) -> f32 {
    // Water is mostly transparent, so occlusion is high
    // Add subtle variation from waves casting shadows
    let wave_shadow = crate::noise::perlin_noise(uv, WAVE_SCALE * 0.5);
    let shadow_factor = (wave_shadow + 1.0) / 2.0; // [0, 1]

    // Base occlusion with slight shadow variation
    (0.95 - shadow_factor * 0.05).clamp(0.85, 1.0)
}

/// Specular texture (linear).
/// Bevy: A = specular.
pub fn specular_texture(_uv: Vec2) -> f32 {
    // Water has moderate specular reflection
    0.5
}

/// Specular tint texture (sRGB).
/// Bevy: RGB = specular tint color.
///
/// Water specular highlights are typically white to light blue
pub fn specular_tint_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.9, 0.95, 1.0)
}

/// Clearcoat texture (linear).
/// Bevy: R = clearcoat.
pub fn clearcoat_texture(_uv: Vec2) -> f32 {
    0.0 // Not used for standard water
}

/// Clearcoat roughness texture (linear).
/// Bevy: G = clearcoat roughness.
pub fn clearcoat_roughness_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Clearcoat normal texture (linear).
/// Bevy: RGB = clearcoat normal vector.
pub fn clearcoat_normal_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.5, 0.5, 1.0)
}

/// Anisotropy texture (linear).
/// Bevy: RG = anisotropy direction, B = anisotropy strength.
///
/// Water can have slight anisotropy due to wave direction
pub fn anisotropy_texture(uv: Vec2) -> Vec3 {
    // Calculate wave direction
    let wave_angle = uv.x * WAVE_SCALE * 0.5;
    let direction = Vec2::new(wave_angle.sin(), wave_angle.cos());

    // Anisotropy strength varies with wave intensity
    let wave_intensity = crate::noise::perlin_noise(uv, WAVE_SCALE);
    let strength = ((wave_intensity + 1.0) / 2.0 * 0.3).clamp(0.0, 0.3);

    Vec3::new(direction.x, direction.y, strength)
}

/// Depth map / Parallax texture (linear).
/// Bevy: R = parallax depth.
pub fn depth_map(uv: Vec2) -> f32 {
    // Subtle parallax based on wave height
    let wave_height = crate::noise::perlin_noise(uv, WAVE_SCALE);
    ((wave_height + 1.0) / 2.0 * 0.05).clamp(0.0, 0.1)
}

/// Diffuse transmission texture (linear).
/// Bevy: A = transmission.
///
/// How much light passes through the water diffusely
pub fn diffuse_transmission_texture(_uv: Vec2) -> f32 {
    WATER_DIFFUSE_TRANSMISSION
}

/// Specular transmission texture (linear).
/// Bevy: R = transmission.
///
/// How much light passes through the water specularly (refraction)
pub fn specular_transmission_texture(_uv: Vec2) -> f32 {
    WATER_SPECULAR_TRANSMISSION
}

/// Thickness texture (linear).
/// Bevy: G = thickness.
///
/// Water thickness affects light absorption
pub fn thickness_texture(_uv: Vec2) -> f32 {
    WATER_THICKNESS
}

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
// ========== Rocky Ground Texture Parameters ==========
// pub fn emissive_texture(uv: Vec2) -> Vec3 {
//     let noise = crate::noise::combined_noise(uv);
//     let intensity = (noise + 1.0) / 2.0;
// }
// ========== Rocky Ground Texture Parameters ==========
// ========== Rocky Ground Texture Parameters ==========

/// Dark brown/black - base rock color
const ROCKY_BASE_COLOR: Vec3 = Vec3::new(0.25, 0.18, 0.12);
/// Medium brown - primary rock color
const ROCKY_MID_COLOR: Vec3 = Vec3::new(0.45, 0.32, 0.20);
/// Light gray/tan - highlighted rock edges
const ROCKY_HIGHLIGHT_COLOR: Vec3 = Vec3::new(0.65, 0.55, 0.45);
/// Pale gray - dry/weathered rock
const ROCKY_DRY_COLOR: Vec3 = Vec3::new(0.75, 0.70, 0.65);

/// Scale for large rock formations
const ROCK_SCALE: f32 = 12.0;
/// Scale for small rock details
const ROCK_DETAIL_SCALE: f32 = 30.0;
/// Scale for fine grain
const ROCK_GRAIN_SCALE: f32 = 50.0;
/// Number of noise octaves for rock detail
const ROCK_OCTAVES: usize = 4;

/// Base roughness for rocky surface (high - very rough)
const ROCK_BASE_ROUGHNESS: f32 = 0.85;
/// Roughness variation range
const ROCK_ROUGHNESS_VARIATION: f32 = 0.10;
/// Metallic value for rocks (very low - non-metallic)
const ROCK_METALLIC: f32 = 0.02;

/// Occlusion base (rocks have deep shadows)
const ROCK_BASE_OCCLUSION: f32 = 0.6;

// ========== Rocky Ground Texture Functions ==========

/// Base color texture for rocky ground (sRGB).
pub fn rocky_base_color_texture(uv: Vec2) -> Vec4 {
    let formation_noise = crate::noise::perlin_noise(uv, ROCK_SCALE * 0.8);
    let formation_factor = (formation_noise + 1.0) / 2.0;
    let detail_noise = crate::noise::perlin_noise(uv, ROCK_DETAIL_SCALE);
    let detail_factor = (detail_noise + 1.0) / 2.0;
    let grain_noise = crate::noise::perlin_noise(uv, ROCK_GRAIN_SCALE);
    let grain_factor = (grain_noise + 1.0) / 2.0;

    let base_color = ROCKY_BASE_COLOR.lerp(ROCKY_MID_COLOR, formation_factor);
    let detail_color = ROCKY_MID_COLOR.lerp(ROCKY_HIGHLIGHT_COLOR, detail_factor);
    let mixed_color = base_color.lerp(detail_color, 0.4);
    let grain_variation = Vec3::splat(0.03) * (grain_factor - 0.5);

    let dry_noise = crate::noise::simplex_noise(uv * ROCK_DETAIL_SCALE, 2.0);
    let dry_factor = ((dry_noise + 1.0) / 2.0).powi(2);
    let final_color = mixed_color.lerp(ROCKY_DRY_COLOR, dry_factor * 0.2);

    Vec4::new(
        (final_color.x + grain_variation.x).clamp(0.0, 1.0),
        (final_color.y + grain_variation.y).clamp(0.0, 1.0),
        (final_color.z + grain_variation.z).clamp(0.0, 1.0),
        1.0,
    )
}

/// Emissive texture for rocky ground (sRGB).
pub fn rocky_emissive_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.01, 0.005, 0.002)
}

/// Metallic-roughness texture for rocky ground (linear).
pub fn rocky_metallic_roughness_texture(uv: Vec2) -> Vec2 {
    let noise = crate::noise::fbm(
        uv,
        ROCK_SCALE,
        ROCK_OCTAVES,
        2.0,
        0.5,
        crate::noise::perlin_noise,
    );
    let noise_factor = (noise + 1.0) / 2.0;
    let roughness = (ROCK_BASE_ROUGHNESS + (noise_factor - 0.5) * ROCK_ROUGHNESS_VARIATION * 2.0)
        .clamp(0.75, 0.98);
    Vec2::new(roughness, ROCK_METALLIC)
}

/// Normal map texture for rocky ground (linear).
pub fn rocky_normal_map_texture(uv: Vec2) -> Vec3 {
    const EPSILON: f32 = 1.0 / 1024.0;

    let height = |offset: Vec2| -> f32 {
        crate::noise::perlin_noise(uv + offset, ROCK_SCALE * 0.6) * 0.5
            + crate::noise::perlin_noise(uv + offset, ROCK_SCALE * 1.2) * 0.3
            + crate::noise::perlin_noise(uv + offset, ROCK_DETAIL_SCALE) * 0.15
            + crate::noise::perlin_noise(uv + offset, ROCK_GRAIN_SCALE) * 0.05
    };

    let height_x_plus = height(Vec2::new(EPSILON, 0.0));
    let height_x_minus = height(Vec2::new(-EPSILON, 0.0));
    let height_y_plus = height(Vec2::new(0.0, EPSILON));
    let height_y_minus = height(Vec2::new(0.0, -EPSILON));

    let dzdx = (height_x_plus - height_x_minus) / (2.0 * EPSILON);
    let dzdy = (height_y_plus - height_y_minus) / (2.0 * EPSILON);
    let normal_ws = Vec3::new(-dzdx, -dzdy, 1.0).normalize();

    Vec3::new(
        (normal_ws.x * 0.5) + 0.5,
        (-normal_ws.y * 0.5) + 0.5,
        normal_ws.z * 0.5 + 0.5,
    )
}

/// Occlusion texture for rocky ground (linear).
pub fn rocky_occlusion_texture(uv: Vec2) -> f32 {
    let formation_shadow = crate::noise::perlin_noise(uv, ROCK_SCALE * 0.5);
    let shadow_factor = (formation_shadow + 1.0) / 2.0;
    let detail_shadow = crate::noise::perlin_noise(uv, ROCK_DETAIL_SCALE);
    let detail_factor = (detail_shadow + 1.0) / 2.0;
    (ROCK_BASE_OCCLUSION - shadow_factor * 0.25 - detail_factor * 0.15).clamp(0.4, 0.85)
}

/// Specular texture for rocky ground (linear).
pub fn rocky_specular_texture(_uv: Vec2) -> f32 {
    0.05
}

/// Specular tint texture for rocky ground (sRGB).
pub fn rocky_specular_tint_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.8, 0.75, 0.7)
}

/// Clearcoat texture for rocky ground (linear).
pub fn rocky_clearcoat_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Clearcoat roughness texture for rocky ground (linear).
pub fn rocky_clearcoat_roughness_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Clearcoat normal texture for rocky ground (linear).
pub fn rocky_clearcoat_normal_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.5, 0.5, 1.0)
}

/// Anisotropy texture for rocky ground (linear).
pub fn rocky_anisotropy_texture(uv: Vec2) -> Vec3 {
    let variation = crate::noise::perlin_noise(uv, ROCK_SCALE * 0.3);
    let strength = ((variation + 1.0) / 2.0 * 0.05).clamp(0.0, 0.05);
    Vec3::new(0.0, 0.0, strength)
}

/// Depth map / Parallax texture for rocky ground (linear).
pub fn rocky_depth_map(uv: Vec2) -> f32 {
    let height = crate::noise::perlin_noise(uv, ROCK_SCALE * 0.8);
    ((height + 1.0) / 2.0 * 0.15).clamp(0.0, 0.15)
}

/// Diffuse transmission texture for rocky ground (linear).
pub fn rocky_diffuse_transmission_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Specular transmission texture for rocky ground (linear).
pub fn rocky_specular_transmission_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Thickness texture for rocky ground (linear).
pub fn rocky_thickness_texture(_uv: Vec2) -> f32 {
    0.0
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
// }==========

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
