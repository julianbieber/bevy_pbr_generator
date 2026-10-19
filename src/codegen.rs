//! The Rust a bevy project pastes in to build a `StandardMaterial` from a
//! material's exported maps, the way the editor's preview builds it.

use std::fmt::Write;

use crate::gpu::export::slug;
use crate::gpu::maps::MapKind;

/// Each `StandardMaterial` texture slot the preview binds, and the map that
/// feeds it.
pub const MATERIAL_SLOTS: [(&str, MapKind); 15] = [
    ("base_color_texture", MapKind::BaseColor),
    ("normal_map_texture", MapKind::Normal),
    ("occlusion_texture", MapKind::Orm),
    ("metallic_roughness_texture", MapKind::Orm),
    ("emissive_texture", MapKind::Emissive),
    ("specular_transmission_texture", MapKind::Transmission),
    ("thickness_texture", MapKind::Transmission),
    ("diffuse_transmission_texture", MapKind::Transmission),
    ("specular_tint_texture", MapKind::Specular),
    ("specular_texture", MapKind::Specular),
    ("clearcoat_texture", MapKind::Clearcoat),
    ("clearcoat_roughness_texture", MapKind::Clearcoat),
    ("clearcoat_normal_texture", MapKind::ClearcoatNormal),
    ("anisotropy_texture", MapKind::Anisotropy),
    ("depth_map", MapKind::Depth),
];

/// Each `StandardMaterial` factor the preview sets away from its default, as
/// the Rust expression the generated code assigns it.
pub const NEUTRAL_FACTORS: [(&str, &str); 10] = [
    ("emissive", "LinearRgba::WHITE"),
    ("perceptual_roughness", "1.0"),
    ("metallic", "1.0"),
    ("reflectance", "1.0"),
    ("clearcoat", "1.0"),
    ("clearcoat_perceptual_roughness", "1.0"),
    ("anisotropy_strength", "1.0"),
    ("specular_transmission", "1.0"),
    ("diffuse_transmission", "1.0"),
    ("thickness", "1.0"),
];

/// The name of the function [`material_code`] generates for `material`: a
/// valid Rust identifier for any material name.
pub fn function_name(material: &str) -> String {
    format!("material_{}", slug(material))
}

/// The source of one Rust module body for a bevy project: the `use` lines and a
/// function returning a `bsn!` scene that gives its entity a `MeshMaterial3d`.
///
/// The text depends on `material` alone. It loads the maps from
/// `<slug>/<stem>.png`, so the exported material folder has to sit directly
/// under the project's `assets/`, and it binds every map, so the project needs
/// the bevy features `pbr_transmission_textures`, `pbr_specular_textures`,
/// `pbr_anisotropy_texture` and `pbr_multi_layer_material_textures`.
pub fn material_code(material: &str) -> String {
    let folder = slug(material);
    let mut code = String::new();

    let _ = writeln!(code, "use bevy::image::ImageLoaderSettings;");
    let _ = writeln!(code, "use bevy::prelude::*;");
    let _ = writeln!(code);
    let _ = writeln!(
        code,
        "/// The `{folder}` material exported by bevy_pbr_generator."
    );
    let _ = writeln!(code, "///");
    let _ = writeln!(
        code,
        "/// Loads its maps from `assets/{folder}/`. The mesh it goes on needs tangents"
    );
    let _ = writeln!(
        code,
        "/// (`Mesh::generate_tangents`), and every spawn adds a new `StandardMaterial`."
    );
    let _ = writeln!(
        code,
        "pub fn {}() -> impl Scene {{",
        function_name(material)
    );
    let _ = writeln!(code, "    bsn! {{");
    let _ = writeln!(code, "        template(|ctx| {{");
    let _ = writeln!(
        code,
        "            let linear = |settings: &mut ImageLoaderSettings| settings.is_srgb = false;"
    );
    let _ = writeln!(
        code,
        "            let assets = ctx.resource::<AssetServer>();"
    );
    for kind in MapKind::ALL {
        let stem = kind.file_stem();
        let path = format!("{folder}/{stem}.png");
        if kind.rgb_is_srgb() {
            let _ = writeln!(
                code,
                "            let {stem}: Handle<Image> = assets.load(\"{path}\");"
            );
        } else {
            let _ = writeln!(
                code,
                "            let {stem}: Handle<Image> = assets.load_builder().with_settings(linear).load(\"{path}\");"
            );
        }
    }
    let _ = writeln!(code, "            let material = StandardMaterial {{");
    for (field, kind) in MATERIAL_SLOTS {
        let _ = writeln!(
            code,
            "                {field}: Some({}.clone()),",
            kind.file_stem()
        );
    }
    for (field, value) in NEUTRAL_FACTORS {
        let _ = writeln!(code, "                {field}: {value},");
    }
    let _ = writeln!(code, "                ..default()");
    let _ = writeln!(code, "            }};");
    let _ = writeln!(
        code,
        "            let handle = ctx.resource_mut::<Assets<StandardMaterial>>().add(material);"
    );
    let _ = writeln!(code, "            Ok(MeshMaterial3d(handle))");
    let _ = writeln!(code, "        }})");
    let _ = writeln!(code, "    }}");
    let _ = writeln!(code, "}}");
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    // A material name may start with a digit, be empty, or spell a keyword; the
    // pasted function has to compile for every one of them.
    #[test]
    fn function_names_are_identifiers() {
        for name in ["", "2x", "!", "Self", "fn", "Rocky Ground"] {
            let ident = function_name(name);
            assert!(ident.starts_with("material_"), "{ident}");
            assert!(
                ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
                "{ident}"
            );
        }
    }

    // Export encodes only the colour maps as sRGB, so the pasted code has to load
    // exactly the others as linear or they render wrong.
    #[test]
    fn only_colour_maps_load_as_srgb() {
        let code = material_code("Rocky Ground");
        for kind in MapKind::ALL {
            let line = code
                .lines()
                .find(|line| line.contains(&format!("let {}:", kind.file_stem())))
                .unwrap();
            assert_eq!(
                !kind.rgb_is_srgb(),
                line.contains("with_settings(linear)"),
                "{line}"
            );
        }
    }

    // Every map export writes must be bound somewhere, or the pasted material
    // silently drops one.
    #[test]
    fn every_map_feeds_a_slot() {
        for kind in MapKind::ALL {
            assert!(MATERIAL_SLOTS.iter().any(|(_, k)| *k == kind), "{kind:?}");
        }
    }
}
