//! The ten PBR maps a material writes, and the images they are generated into.

use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

/// How many maps a material writes. Fixed: it is also the number of storage
/// texture bindings in group 0, which every material shares.
pub const MAP_COUNT: usize = 10;

/// The format every map is allocated in.
///
/// `rgba16float` is a core WebGPU write-only storage format and is filterable,
/// so one allocation serves as both the compute target and the texture
/// `StandardMaterial` samples.
pub const MAP_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

/// One of the ten maps, in binding order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum MapKind {
    BaseColor,
    Normal,
    Orm,
    Emissive,
    Transmission,
    Specular,
    Clearcoat,
    ClearcoatNormal,
    Anisotropy,
    Depth,
}

impl MapKind {
    /// Every map, in the order they are bound in group 0 and listed in the UI.
    pub const ALL: [MapKind; MAP_COUNT] = [
        Self::BaseColor,
        Self::Normal,
        Self::Orm,
        Self::Emissive,
        Self::Transmission,
        Self::Specular,
        Self::Clearcoat,
        Self::ClearcoatNormal,
        Self::Anisotropy,
        Self::Depth,
    ];

    /// Name shown on the map thumbnail.
    pub fn label(self) -> &'static str {
        match self {
            Self::BaseColor => "base colour",
            Self::Normal => "normal",
            Self::Orm => "ORM",
            Self::Emissive => "emissive",
            Self::Transmission => "transmission",
            Self::Specular => "specular",
            Self::Clearcoat => "clearcoat",
            Self::ClearcoatNormal => "clearcoat normal",
            Self::Anisotropy => "anisotropy",
            Self::Depth => "depth",
        }
    }

    /// Stem of the PNG this map exports to.
    pub fn file_stem(self) -> &'static str {
        match self {
            Self::BaseColor => "base_color",
            Self::Normal => "normal",
            Self::Orm => "orm",
            Self::Emissive => "emissive",
            Self::Transmission => "transmission",
            Self::Specular => "specular",
            Self::Clearcoat => "clearcoat",
            Self::ClearcoatNormal => "clearcoat_normal",
            Self::Anisotropy => "anisotropy",
            Self::Depth => "depth",
        }
    }

    /// Whether the RGB channels carry a colour that must have the sRGB transfer
    /// function applied when written to a PNG. Alpha is always linear, and maps
    /// that carry no colour at all are linear throughout.
    pub fn rgb_is_srgb(self) -> bool {
        matches!(self, Self::BaseColor | Self::Emissive | Self::Specular)
    }
}

/// Allocates a fresh set of ten square maps at `resolution`.
///
/// The images live only in the render world and carry the usages the compute
/// pass, the preview material and the export readback each need.
pub fn allocate_maps(images: &mut Assets<Image>, resolution: u32) -> Vec<Handle<Image>> {
    MapKind::ALL
        .iter()
        .map(|kind| {
            let mut image = Image::new_fill(
                Extent3d {
                    width: resolution,
                    height: resolution,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &[0u8; 8],
                MAP_FORMAT,
                RenderAssetUsages::RENDER_WORLD,
            );
            image.texture_descriptor.label = Some(kind.file_stem());
            image.texture_descriptor.usage = TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC;
            images.add(image)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Group 0 binds the maps by position, so the order of `ALL` is part of the
    // shader contract in `pbr_maps.wgsl` and must not drift.
    #[test]
    fn map_order_matches_the_binding_order() {
        assert_eq!(MapKind::ALL.len(), MAP_COUNT);
        assert_eq!(MapKind::ALL[0], MapKind::BaseColor);
        assert_eq!(MapKind::ALL[MAP_COUNT - 1], MapKind::Depth);
    }

    // Exactly the three maps whose RGB is a colour get the sRGB transfer
    // function at export; applying it to a data map would corrupt it.
    #[test]
    fn only_colour_maps_are_srgb() {
        let srgb: Vec<&str> = MapKind::ALL
            .iter()
            .filter(|k| k.rgb_is_srgb())
            .map(|k| k.file_stem())
            .collect();
        assert_eq!(srgb, ["base_color", "emissive", "specular"]);
    }
}
