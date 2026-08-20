use bevy::render::render_resource::TextureFormat;

pub const MAP_COUNT: usize = 10;
pub const MAP_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PbrMap {
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

impl PbrMap {
    pub const ALL: [PbrMap; MAP_COUNT] = [
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

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }

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

    pub fn label(self) -> &'static str {
        match self {
            Self::BaseColor => "Base colour",
            Self::Normal => "Normal",
            Self::Orm => "ORM",
            Self::Emissive => "Emissive",
            Self::Transmission => "Transmission",
            Self::Specular => "Specular",
            Self::Clearcoat => "Clearcoat",
            Self::ClearcoatNormal => "Clearcoat normal",
            Self::Anisotropy => "Anisotropy",
            Self::Depth => "Depth",
        }
    }

    pub fn channels(self) -> &'static str {
        match self {
            Self::BaseColor => "RGB colour, A opacity",
            Self::Normal => "RGB normal",
            Self::Orm => "R occlusion, G roughness, B metallic",
            Self::Emissive => "RGB emissive",
            Self::Transmission => "R specular, G thickness, A diffuse",
            Self::Specular => "RGB tint, A specular",
            Self::Clearcoat => "R clearcoat, G roughness",
            Self::ClearcoatNormal => "RGB normal",
            Self::Anisotropy => "RG direction, B strength",
            Self::Depth => "R parallax depth",
        }
    }

    pub fn encodes_srgb_rgb(self) -> bool {
        matches!(self, Self::BaseColor | Self::Emissive | Self::Specular)
    }
}
