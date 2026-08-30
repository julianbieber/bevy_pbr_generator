//! The previewed mesh, the light on it, and the binding of the generated maps
//! to the material it is drawn with.

use bevy::light::light_consts::lux;
use bevy::light::AtmosphereEnvironmentMapLight;
use bevy::prelude::*;

use crate::app::camera::PreviewShapeMarker;
use crate::app::{PreviewShape, SunAngles};
use crate::gpu::maps::{MapKind, MAP_COUNT};
use crate::gpu::PreviewMaps;

/// Marks the directional light the sun sliders drive.
#[derive(Component, Debug)]
pub struct Sun;

pub fn spawn_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    shape: Res<PreviewShape>,
    sun: Res<SunAngles>,
) {
    let material = materials.add(preview_material());

    commands.spawn((
        Mesh3d(meshes.add(shape.mesh())),
        MeshMaterial3d(material),
        Transform::default(),
        PreviewShapeMarker(*shape),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(sun.rotation()),
        Sun,
    ));
}

/// The material the maps are bound into.
///
/// Every factor a generated map is multiplied by is set to its neutral value
/// rather than left at its `StandardMaterial` default, so that the map alone
/// decides the result. A factor left at zero — clearcoat, anisotropy and the
/// transmissions all default to it, and emissive to black — would multiply its
/// map away whatever the material wrote; `clearcoat` and `anisotropy_strength`
/// go further and key whether their shader path is compiled in at all.
fn preview_material() -> StandardMaterial {
    StandardMaterial {
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
    }
}

pub fn follow_shape(
    shape: Res<PreviewShape>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut Mesh3d, &mut PreviewShapeMarker)>,
) {
    if !shape.is_changed() {
        return;
    }
    for (mut mesh, mut marker) in &mut query {
        if marker.0 != *shape {
            marker.0 = *shape;
            mesh.0 = meshes.add(shape.mesh());
        }
    }
}

pub fn follow_sun(
    sun: Res<SunAngles>,
    mut lights: Query<&mut Transform, With<Sun>>,
    mut probes: Query<&mut AtmosphereEnvironmentMapLight>,
) {
    if !sun.is_changed() {
        return;
    }
    for mut transform in &mut lights {
        transform.rotation = sun.rotation();
    }
    // Touching the probe marks it changed, which is what makes the atmosphere
    // re-derive the image-based light from the new sun position.
    for mut probe in &mut probes {
        probe.set_changed();
    }
}

/// Points the preview material at the generated maps.
///
/// One image serves several `StandardMaterial` slots where Bevy reads different
/// channels of the same packed map — occlusion and metallic-roughness both come
/// from the ORM map, and all three transmission slots from the transmission map.
pub fn bind_maps_to_material(
    maps: Res<PreviewMaps>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&MeshMaterial3d<StandardMaterial>, With<PreviewShapeMarker>>,
) {
    if !maps.is_changed() || maps.images.len() != MAP_COUNT {
        return;
    }

    let at = |kind: MapKind| -> Option<Handle<Image>> {
        MapKind::ALL
            .iter()
            .position(|k| *k == kind)
            .and_then(|index| maps.images.get(index).cloned())
    };

    for handle in &query {
        let Some(mut material) = materials.get_mut(&handle.0) else {
            continue;
        };
        material.base_color_texture = at(MapKind::BaseColor);
        material.normal_map_texture = at(MapKind::Normal);
        material.occlusion_texture = at(MapKind::Orm);
        material.metallic_roughness_texture = at(MapKind::Orm);
        material.emissive_texture = at(MapKind::Emissive);
        material.specular_transmission_texture = at(MapKind::Transmission);
        material.thickness_texture = at(MapKind::Transmission);
        material.diffuse_transmission_texture = at(MapKind::Transmission);
        material.specular_tint_texture = at(MapKind::Specular);
        material.specular_texture = at(MapKind::Specular);
        material.clearcoat_texture = at(MapKind::Clearcoat);
        material.clearcoat_roughness_texture = at(MapKind::Clearcoat);
        material.clearcoat_normal_texture = at(MapKind::ClearcoatNormal);
        material.anisotropy_texture = at(MapKind::Anisotropy);
        material.depth_map = at(MapKind::Depth);
    }
}
