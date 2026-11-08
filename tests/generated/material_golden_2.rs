use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;

/// The `golden_2` material exported by bevy_pbr_generator.
///
/// Loads its maps from `assets/golden_2/`. The mesh it goes on needs tangents
/// (`Mesh::generate_tangents`), and every spawn adds a new `StandardMaterial`.
pub fn material_golden_2() -> impl Scene {
    bsn! {
        template(|ctx| {
            let linear = |settings: &mut ImageLoaderSettings| settings.is_srgb = false;
            let assets = ctx.resource::<AssetServer>();
            let base_color: Handle<Image> = assets.load("golden_2/base_color.ktx2");
            let normal: Handle<Image> = assets.load_builder().with_settings(linear).load("golden_2/normal.ktx2");
            let orm: Handle<Image> = assets.load_builder().with_settings(linear).load("golden_2/orm.ktx2");
            let emissive: Handle<Image> = assets.load("golden_2/emissive.ktx2");
            let transmission: Handle<Image> = assets.load_builder().with_settings(linear).load("golden_2/transmission.ktx2");
            let specular: Handle<Image> = assets.load("golden_2/specular.ktx2");
            let clearcoat: Handle<Image> = assets.load_builder().with_settings(linear).load("golden_2/clearcoat.ktx2");
            let clearcoat_normal: Handle<Image> = assets.load_builder().with_settings(linear).load("golden_2/clearcoat_normal.ktx2");
            let anisotropy: Handle<Image> = assets.load_builder().with_settings(linear).load("golden_2/anisotropy.ktx2");
            let depth: Handle<Image> = assets.load_builder().with_settings(linear).load("golden_2/depth.ktx2");
            let material = StandardMaterial {
                base_color_texture: Some(base_color.clone()),
                normal_map_texture: Some(normal.clone()),
                occlusion_texture: Some(orm.clone()),
                metallic_roughness_texture: Some(orm.clone()),
                emissive_texture: Some(emissive.clone()),
                specular_transmission_texture: Some(transmission.clone()),
                thickness_texture: Some(transmission.clone()),
                diffuse_transmission_texture: Some(transmission.clone()),
                specular_tint_texture: Some(specular.clone()),
                specular_texture: Some(specular.clone()),
                clearcoat_texture: Some(clearcoat.clone()),
                clearcoat_roughness_texture: Some(clearcoat.clone()),
                clearcoat_normal_texture: Some(clearcoat_normal.clone()),
                anisotropy_texture: Some(anisotropy.clone()),
                depth_map: Some(depth.clone()),
                emissive: LinearRgba::WHITE,
                perceptual_roughness: 1.0,
                metallic: 1.0,
                reflectance: 1.0,
                clearcoat: 1.0,
                clearcoat_perceptual_roughness: 1.0,
                anisotropy_strength: 1.0,
                specular_transmission: 1.0,
                diffuse_transmission: 1.0,
                thickness: 1.0,
                ..default()
            };
            let handle = ctx.resource_mut::<Assets<StandardMaterial>>().add(material);
            Ok(MeshMaterial3d(handle))
        })
    }
}
