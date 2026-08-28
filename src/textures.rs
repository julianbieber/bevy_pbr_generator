//! The look of each material: what value every PBR property takes at a point on
//! the surface.
//!
//! Every function here samples a single UV coordinate, given in `[0.0, 1.0]` on
//! both axes, and returns that property's natural type. They are pure and hold
//! no state between samples, so they may be called in any order.

use glam::{Vec2, Vec3, Vec4};

const WATER_DEEP_COLOR: Vec3 = Vec3::new(0.02, 0.15, 0.40);
const WATER_SHALLOW_COLOR: Vec3 = Vec3::new(0.05, 0.35, 0.55);
const WATER_CREST_COLOR: Vec3 = Vec3::new(0.7, 0.85, 0.95);

const WAVE_SCALE: f32 = 8.0;
const RIPPLE_SCALE: f32 = 25.0;
const WAVE_FREQUENCY: f32 = 2.0;
const WAVE_OCTAVES: usize = 3;

const WATER_BASE_ROUGHNESS: f32 = 0.15;
const WATER_CREST_ROUGHNESS: f32 = 0.40;

const WATER_DIFFUSE_TRANSMISSION: f32 = 0.90;
const WATER_SPECULAR_TRANSMISSION: f32 = 0.50;
const WATER_THICKNESS: f32 = 0.10;

const WATER_EMISSIVE_INTENSITY: f32 = 0.05;

/// Water surface colour in RGB and opacity in A, all in `[0.0, 1.0]`. sRGB.
/// Opacity is always 1.0 — transparency is carried by the transmission
/// textures, not by alpha.
pub fn base_color_texture(uv: Vec2) -> Vec4 {
    let depth_noise = crate::noise::perlin_noise(uv, WAVE_SCALE * 0.5);
    let depth_factor = (depth_noise + 1.0) / 2.0;

    let ripple_noise = crate::noise::perlin_noise(uv, RIPPLE_SCALE);
    let ripple_factor = (ripple_noise + 1.0) / 2.0;

    let crest_noise = crate::noise::simplex_noise(uv * WAVE_SCALE, WAVE_FREQUENCY);
    let crest_factor = ((crest_noise + 1.0) / 2.0).powi(2);

    let base_color = WATER_DEEP_COLOR.lerp(WATER_SHALLOW_COLOR, depth_factor);

    let crest_color = WATER_CREST_COLOR.lerp(base_color, 0.7);
    let final_color = base_color.lerp(crest_color, crest_factor * 0.3);

    let ripple_variation = Vec3::splat(0.02) * (ripple_factor - 0.5);

    Vec4::new(
        (final_color.x + ripple_variation.x).clamp(0.0, 1.0),
        (final_color.y + ripple_variation.y).clamp(0.0, 1.0),
        (final_color.z + ripple_variation.z).clamp(0.0, 1.0),
        1.0,
    )
}

/// Light the water emits on its own, as a blue-cyan RGB colour scaled by
/// `WATER_EMISSIVE_INTENSITY`; never brighter than about 0.075 per channel, so
/// it reads as a glow rather than a light source. sRGB.
pub fn emissive_texture(uv: Vec2) -> Vec3 {
    let crest_noise = crate::noise::simplex_noise(uv * WAVE_SCALE, WAVE_FREQUENCY);
    let crest_factor = ((crest_noise + 1.0) / 2.0).powi(2);

    let deep_glow = Vec3::new(0.1, 0.3, 0.6) * WATER_EMISSIVE_INTENSITY;

    let crest_glow = Vec3::new(0.7, 0.85, 0.95) * WATER_EMISSIVE_INTENSITY * 1.5;

    deep_glow.lerp(crest_glow, crest_factor)
}

/// Roughness in `x` and metallic in `y`, both in `[0.0, 1.0]`. Linear.
/// Water is a dielectric, so metallic is always 0.0.
pub fn metallic_roughness_texture(uv: Vec2) -> Vec2 {
    let wave_noise = crate::noise::fbm(
        uv,
        WAVE_SCALE,
        WAVE_OCTAVES,
        2.0,
        0.5,
        crate::noise::perlin_noise,
    );
    let wave_factor = (wave_noise + 1.0) / 2.0;

    let crest_noise = crate::noise::simplex_noise(uv * WAVE_SCALE * 1.5, WAVE_FREQUENCY * 2.0);
    let crest_factor = ((crest_noise + 1.0) / 2.0).powi(3);

    let roughness = WATER_BASE_ROUGHNESS
        + (WATER_CREST_ROUGHNESS - WATER_BASE_ROUGHNESS) * crest_factor
        + (WATER_CREST_ROUGHNESS - WATER_BASE_ROUGHNESS) * wave_factor * 0.3;

    Vec2::new(roughness.clamp(0.0, 1.0), 0.0)
}

/// Tangent-space surface normal of the water, in the usual normal-map encoding:
/// `+X` right, `+Y` up, `+Z` out of the surface, each component remapped from
/// `[-1.0, 1.0]` to `[0.0, 1.0]`, with green flipped for the OpenGL convention
/// Bevy expects. Linear.
///
/// The height field is differenced at a fixed offset of 1/1024 UV, so the
/// result is only faithful when the image is rendered near 1024 pixels square.
pub fn normal_map_texture(uv: Vec2) -> Vec3 {
    const EPSILON: f32 = 1.0 / 1024.0;

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

    let dzdx = (height_x_plus - height_x_minus) / (2.0 * EPSILON);
    let dzdy = (height_y_plus - height_y_minus) / (2.0 * EPSILON);

    let normal_ws = Vec3::new(-dzdx, -dzdy, 1.0).normalize();

    Vec3::new(
        (normal_ws.x * 0.5) + 0.5,
        (-normal_ws.y * 0.5) + 0.5,
        normal_ws.z * 0.5 + 0.5,
    )
}

/// Ambient light reaching the water surface, in `[0.85, 1.0]` — an open surface
/// occludes very little. Linear.
pub fn occlusion_texture(uv: Vec2) -> f32 {
    let wave_shadow = crate::noise::perlin_noise(uv, WAVE_SCALE * 0.5);
    let shadow_factor = (wave_shadow + 1.0) / 2.0;

    (0.95 - shadow_factor * 0.05).clamp(0.85, 1.0)
}

/// Strength of the specular reflection. Uniform at 0.5 across the surface.
/// Linear.
pub fn specular_texture(_uv: Vec2) -> f32 {
    0.5
}

/// Colour of the specular highlight — near-white with a blue cast, uniform
/// across the surface. sRGB.
pub fn specular_tint_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.9, 0.95, 1.0)
}

/// Strength of the clearcoat layer. Always 0.0: water carries no such layer,
/// so the accompanying clearcoat textures have no effect. Linear.
pub fn clearcoat_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Roughness of the clearcoat layer. Always 0.0, and unused while
/// [`clearcoat_texture`] is 0.0. Linear.
pub fn clearcoat_roughness_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Tangent-space normal of the clearcoat layer, remapped to `[0.0, 1.0]`.
/// Always flat, and unused while [`clearcoat_texture`] is 0.0. Linear.
pub fn clearcoat_normal_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.5, 0.5, 1.0)
}

/// Direction in which highlights are stretched, in `x`/`y` as a unit vector in
/// `[-1.0, 1.0]`, and the strength of that stretching in `z`, capped at 0.3.
/// The direction follows the wave fronts. Linear.
pub fn anisotropy_texture(uv: Vec2) -> Vec3 {
    let wave_angle = uv.x * WAVE_SCALE * 0.5;
    let direction = Vec2::new(wave_angle.sin(), wave_angle.cos());

    let wave_intensity = crate::noise::perlin_noise(uv, WAVE_SCALE);
    let strength = ((wave_intensity + 1.0) / 2.0 * 0.3).clamp(0.0, 0.3);

    Vec3::new(direction.x, direction.y, strength)
}

/// Apparent height of the surface for parallax mapping, in `[0.0, 0.05]` —
/// deliberately shallow, since water displaces little. Linear.
pub fn depth_map(uv: Vec2) -> f32 {
    let wave_height = crate::noise::perlin_noise(uv, WAVE_SCALE);
    ((wave_height + 1.0) / 2.0 * 0.05).clamp(0.0, 0.1)
}

/// Fraction of light scattered through the surface diffusely. Uniform at 0.9:
/// water passes nearly all of it. Linear.
pub fn diffuse_transmission_texture(_uv: Vec2) -> f32 {
    WATER_DIFFUSE_TRANSMISSION
}

/// Fraction of light refracted through the surface. Uniform at 0.5. Linear.
pub fn specular_transmission_texture(_uv: Vec2) -> f32 {
    WATER_SPECULAR_TRANSMISSION
}

/// Distance light travels inside the volume, driving how much of it is
/// absorbed. Uniform at 0.1, a thin body. Linear.
pub fn thickness_texture(_uv: Vec2) -> f32 {
    WATER_THICKNESS
}

const ROCKY_BASE_COLOR: Vec3 = Vec3::new(0.25, 0.18, 0.12);
const ROCKY_MID_COLOR: Vec3 = Vec3::new(0.45, 0.32, 0.20);
const ROCKY_HIGHLIGHT_COLOR: Vec3 = Vec3::new(0.65, 0.55, 0.45);
const ROCKY_DRY_COLOR: Vec3 = Vec3::new(0.75, 0.70, 0.65);

const ROCK_SCALE: f32 = 12.0;
const ROCK_DETAIL_SCALE: f32 = 30.0;
const ROCK_GRAIN_SCALE: f32 = 50.0;
const ROCK_OCTAVES: usize = 4;

const ROCK_BASE_ROUGHNESS: f32 = 0.85;
const ROCK_ROUGHNESS_VARIATION: f32 = 0.10;
const ROCK_METALLIC: f32 = 0.02;

const ROCK_BASE_OCCLUSION: f32 = 0.6;

/// Rock surface colour in RGB and opacity in A, all in `[0.0, 1.0]`. sRGB.
/// Opacity is always 1.0 — rock is fully opaque.
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

/// Light the rock emits on its own. Near-black and uniform: rock glows only
/// enough to lift it off pure black. sRGB.
pub fn rocky_emissive_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.01, 0.005, 0.002)
}

/// Roughness in `x`, clamped to `[0.75, 0.98]`, and metallic in `y`. Linear.
/// Rock is a dielectric, so metallic stays at 0.02 everywhere.
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

/// Tangent-space surface normal of the rock, in the usual normal-map encoding:
/// `+X` right, `+Y` up, `+Z` out of the surface, each component remapped from
/// `[-1.0, 1.0]` to `[0.0, 1.0]`, with green flipped for the OpenGL convention
/// Bevy expects. Linear.
///
/// The height field is differenced at a fixed offset of 1/1024 UV, so the
/// result is only faithful when the image is rendered near 1024 pixels square.
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

/// Ambient light reaching the rock surface, in `[0.4, 0.85]` — crevices between
/// formations shadow deeply. Linear.
pub fn rocky_occlusion_texture(uv: Vec2) -> f32 {
    let formation_shadow = crate::noise::perlin_noise(uv, ROCK_SCALE * 0.5);
    let shadow_factor = (formation_shadow + 1.0) / 2.0;
    let detail_shadow = crate::noise::perlin_noise(uv, ROCK_DETAIL_SCALE);
    let detail_factor = (detail_shadow + 1.0) / 2.0;
    (ROCK_BASE_OCCLUSION - shadow_factor * 0.25 - detail_factor * 0.15).clamp(0.4, 0.85)
}

/// Strength of the specular reflection. Uniform at 0.05: rock is close to
/// fully diffuse. Linear.
pub fn rocky_specular_texture(_uv: Vec2) -> f32 {
    0.05
}

/// Colour of the specular highlight — a warm grey, uniform across the surface.
/// sRGB.
pub fn rocky_specular_tint_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.8, 0.75, 0.7)
}

/// Strength of the clearcoat layer. Always 0.0: rock carries no such layer, so
/// the accompanying clearcoat textures have no effect. Linear.
pub fn rocky_clearcoat_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Roughness of the clearcoat layer. Always 0.0, and unused while
/// [`rocky_clearcoat_texture`] is 0.0. Linear.
pub fn rocky_clearcoat_roughness_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Tangent-space normal of the clearcoat layer, remapped to `[0.0, 1.0]`.
/// Always flat, and unused while [`rocky_clearcoat_texture`] is 0.0. Linear.
pub fn rocky_clearcoat_normal_texture(_uv: Vec2) -> Vec3 {
    Vec3::new(0.5, 0.5, 1.0)
}

/// Direction in which highlights are stretched, in `x`/`y`, and the strength of
/// that stretching in `z`, capped at 0.05. The direction is always zero: rock
/// has no grain to align to, leaving only a faint isotropic strength. Linear.
pub fn rocky_anisotropy_texture(uv: Vec2) -> Vec3 {
    let variation = crate::noise::perlin_noise(uv, ROCK_SCALE * 0.3);
    let strength = ((variation + 1.0) / 2.0 * 0.05).clamp(0.0, 0.05);
    Vec3::new(0.0, 0.0, strength)
}

/// Apparent height of the surface for parallax mapping, in `[0.0, 0.15]` —
/// three times the relief of water. Linear.
pub fn rocky_depth_map(uv: Vec2) -> f32 {
    let height = crate::noise::perlin_noise(uv, ROCK_SCALE * 0.8);
    ((height + 1.0) / 2.0 * 0.15).clamp(0.0, 0.15)
}

/// Fraction of light scattered through the surface diffusely. Always 0.0: rock
/// is opaque. Linear.
pub fn rocky_diffuse_transmission_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Fraction of light refracted through the surface. Always 0.0: rock is opaque.
/// Linear.
pub fn rocky_specular_transmission_texture(_uv: Vec2) -> f32 {
    0.0
}

/// Distance light travels inside the volume. Always 0.0, and unused while both
/// transmission textures are 0.0. Linear.
pub fn rocky_thickness_texture(_uv: Vec2) -> f32 {
    0.0
}
