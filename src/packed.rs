//! Packed texture generator functions.
//! These combine individual PBR properties into optimally packed textures
//! according to BEVY_STANDARD_MATERIAL_TEXTURE_PACKING.md.

use crate::textures;
use glam::Vec2;

/// Base color + Opacity (sRGB)
/// RGB = base_color, A = opacity
pub fn base_color(uv: Vec2) -> glam::Vec4 {
    textures::base_color_texture(uv)
}

/// Normal map (Linear)
/// RGB = normal vector
pub fn normal(uv: Vec2) -> glam::Vec3 {
    textures::normal_map_texture(uv)
}

/// ORM: Occlusion, Roughness, Metallic (Linear)
/// R = occlusion, G = roughness, B = metallic
pub fn orm(uv: Vec2) -> glam::Vec3 {
    let occlusion = textures::occlusion_texture(uv);
    let mr = textures::metallic_roughness_texture(uv);
    glam::Vec3::new(occlusion, mr.x, mr.y) // R=occlusion, G=roughness, B=metallic
}

/// Emissive (sRGB)
/// RGB = emissive color
pub fn emissive(uv: Vec2) -> glam::Vec3 {
    textures::emissive_texture(uv)
}

/// Transmission (Linear)
/// R = specular_transmission, G = thickness, A = diffuse_transmission
pub fn transmission(uv: Vec2) -> glam::Vec4 {
    let specular_trans = textures::specular_transmission_texture(uv);
    let thickness = textures::thickness_texture(uv);
    let diffuse_trans = textures::diffuse_transmission_texture(uv);
    glam::Vec4::new(specular_trans, thickness, 0.0, diffuse_trans)
}

/// Specular + Specular Tint (Linear for specular, sRGB for tint)
/// RGB = specular_tint, A = specular
pub fn specular(uv: Vec2) -> glam::Vec4 {
    let tint = textures::specular_tint_texture(uv);
    let specular = textures::specular_texture(uv);
    glam::Vec4::new(tint.x, tint.y, tint.z, specular)
}

/// Clearcoat (Linear)
/// R = clearcoat, G = clearcoat_roughness
pub fn clearcoat(uv: Vec2) -> glam::Vec2 {
    let clearcoat = textures::clearcoat_texture(uv);
    let roughness = textures::clearcoat_roughness_texture(uv);
    glam::Vec2::new(clearcoat, roughness)
}

/// Clearcoat Normal (Linear)
/// RGB = clearcoat normal vector
pub fn clearcoat_normal(uv: Vec2) -> glam::Vec3 {
    textures::clearcoat_normal_texture(uv)
}

/// Anisotropy (Linear)
/// RG = direction, B = strength
pub fn anisotropy(uv: Vec2) -> glam::Vec3 {
    textures::anisotropy_texture(uv)
}

/// Depth / Parallax (Linear)
/// R = parallax depth
pub fn depth(uv: Vec2) -> f32 {
    textures::depth_map(uv)
}
