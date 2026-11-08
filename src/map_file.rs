//! The file formats a map is stored in, and the conversion between a map's 8-bit
//! RGBA texels and a KTX2 file.

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, Image, ImageSampler, ImageType};
use bevy::render::render_resource::TextureFormat;
use ctt::encoders::bc7enc::Bc7encSettings;
use ctt::encoders::Encoder;
use ctt::{
    AlphaMode, ColorSpace, Container, ConvertSettings, Format, PipelineOutput, Quality, Surface,
    TargetFormat, TextureKind,
};

use crate::gpu::maps::MapKind;
use crate::material::params::{linear_to_srgb, srgb_to_linear};

/// How exported and combined maps are stored. A PNG holds a single level; a KTX2
/// file holds a block-compressed mip chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapFormat {
    #[default]
    Ktx2,
    Png,
}

impl MapFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Ktx2 => "ktx2",
            Self::Png => "png",
        }
    }
}

const SMALLEST_ATLAS_CELL_LEVEL: u32 = 16;

/// Levels from a square map of `side` texels down to 1x1. `side` must be a power
/// of two.
pub fn full_chain_levels(side: u32) -> u32 {
    side.ilog2() + 1
}

/// Levels of an atlas whose cells are `cell` texels, ending before a cell would
/// fall under 16 texels. A cell of at most 16 texels gets one level. `cell` must
/// be a power of two, so that halving never mixes texels of two cells.
pub fn atlas_chain_levels(cell: u32) -> u32 {
    cell.ilog2()
        .saturating_sub(SMALLEST_ATLAS_CELL_LEVEL.ilog2())
        + 1
}

/// The bytes of a KTX2 file holding `rgba` and `levels - 1` smaller levels, each
/// the 2x2 average of the one above.
///
/// `rgba` is 8-bit RGBA in the encoding [`MapKind::rgb_is_srgb`] says. Each side
/// must be a multiple of 4 and stay whole through `levels - 1` halvings, or reach
/// 1 on the way. Only the channels the map's block format stores survive.
pub fn encode_ktx2(
    kind: MapKind,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    levels: u32,
) -> Result<Vec<u8>, String> {
    let image = ctt::Image {
        surfaces: vec![mip_chain(kind, width, height, rgba, levels)],
        kind: TextureKind::Texture2D,
    };
    let settings = ConvertSettings {
        format: Some(TargetFormat::Compressed {
            encoder: encoder_for(kind),
            format: block_format(kind),
        }),
        container: Container::ktx2_zstd(0),
        quality: Quality::Basic,
        allow_discarding_alpha: true,
        mipmap: false,
        ..Default::default()
    };
    match ctt::convert(image, settings).map_err(|error| error.to_string())? {
        PipelineOutput::Encoded(bytes) => Ok(bytes),
        _ => Err("the encoder returned no KTX2 file".to_string()),
    }
}

/// Width and height of a KTX2 file, from its header alone.
pub fn ktx2_size(bytes: &[u8]) -> Result<(u32, u32), String> {
    let reader = ktx2::Reader::new(bytes).map_err(|error| error.to_string())?;
    let header = reader.header();
    Ok((header.pixel_width, header.pixel_height))
}

/// The largest level of a KTX2 file as 8-bit RGBA, with its width and height.
///
/// Channels the map's block format does not store read as 0, and alpha as 255.
/// Errors when the file is not what [`encode_ktx2`] writes for `kind`.
pub fn decode_ktx2(kind: MapKind, bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    let image = load_ktx2(kind, bytes)?;
    let (width, height) = (image.width(), image.height());
    let found = image.texture_descriptor.format;
    if !matches!(
        (block_format(kind), found),
        (
            Format::BC7_UNORM_BLOCK,
            TextureFormat::Bc7RgbaUnorm | TextureFormat::Bc7RgbaUnormSrgb
        ) | (Format::BC5_UNORM_BLOCK, TextureFormat::Bc5RgUnorm)
            | (Format::BC4_UNORM_BLOCK, TextureFormat::Bc4RUnorm)
    ) {
        return Err(format!("holds {found:?}, not the format of this map"));
    }
    let data = image.data.as_deref().ok_or("the file holds no texels")?;
    Ok((width, height, decode_blocks(found, data, width, height)?))
}

/// A KTX2 file as Bevy loads it, whatever the GPU supports. `data` holds the levels
/// from largest to smallest.
pub fn load_ktx2(kind: MapKind, bytes: &[u8]) -> Result<Image, String> {
    Image::from_buffer(
        bytes,
        ImageType::Extension("ktx2"),
        CompressedImageFormats::all(),
        kind.rgb_is_srgb(),
        ImageSampler::Default,
        RenderAssetUsages::default(),
    )
    .map_err(|error| error.to_string())
}

fn block_format(kind: MapKind) -> Format {
    match kind {
        MapKind::Normal | MapKind::ClearcoatNormal | MapKind::Clearcoat => Format::BC5_UNORM_BLOCK,
        MapKind::Depth => Format::BC4_UNORM_BLOCK,
        MapKind::BaseColor
        | MapKind::Orm
        | MapKind::Emissive
        | MapKind::Transmission
        | MapKind::Specular
        | MapKind::Anisotropy => Format::BC7_UNORM_BLOCK,
    }
}

fn encoder_for(kind: MapKind) -> Encoder {
    if block_format(kind) == Format::BC7_UNORM_BLOCK {
        Encoder::Bc7enc(Bc7encSettings {
            perceptual: kind.rgb_is_srgb(),
            ..Default::default()
        })
    } else {
        Encoder::Auto
    }
}

fn mip_chain(kind: MapKind, width: u32, height: u32, rgba: Vec<u8>, levels: u32) -> Vec<Surface> {
    let srgb = kind.rgb_is_srgb();
    let surface = |data: Vec<u8>, width: u32, height: u32| Surface {
        data,
        width,
        height,
        depth: 1,
        stride: width * 4,
        slice_stride: 0,
        format: Format::R8G8B8A8_UNORM,
        color_space: if srgb {
            ColorSpace::Srgb
        } else {
            ColorSpace::Linear
        },
        alpha: AlphaMode::Opaque,
    };
    let mut chain = Vec::with_capacity(levels as usize);
    let (mut w, mut h, mut level) = (width, height, rgba);
    for _ in 1..levels {
        let next = halved(&level, w as usize, h as usize, srgb);
        chain.push(surface(level, w, h));
        level = next;
        w /= 2;
        h /= 2;
    }
    chain.push(surface(level, w, h));
    chain
}

fn halved(level: &[u8], width: usize, height: usize, srgb: bool) -> Vec<u8> {
    let (half_width, half_height) = (width / 2, height / 2);
    let mut out = Vec::with_capacity(half_width * half_height * 4);
    for y in 0..half_height {
        for x in 0..half_width {
            for channel in 0..4 {
                let is_colour = srgb && channel < 3;
                let sum: f32 = [(0, 0), (1, 0), (0, 1), (1, 1)]
                    .into_iter()
                    .map(|(dx, dy)| {
                        let at = ((y * 2 + dy) * width + x * 2 + dx) * 4 + channel;
                        let value = f32::from(level[at]) / 255.0;
                        if is_colour {
                            srgb_to_linear(value)
                        } else {
                            value
                        }
                    })
                    .sum();
                let mean = sum / 4.0;
                let value = if is_colour {
                    linear_to_srgb(mean)
                } else {
                    mean
                };
                out.push((value.clamp(0.0, 1.0) * 255.0).round() as u8);
            }
        }
    }
    out
}

fn decode_blocks(
    format: TextureFormat,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    let (width, height) = (width as usize, height as usize);
    let block_bytes = if format == TextureFormat::Bc4RUnorm {
        8
    } else {
        16
    };
    let (columns, rows) = (width.div_ceil(4), height.div_ceil(4));
    if data.len() < columns * rows * block_bytes {
        return Err("the largest level is shorter than its size says".to_string());
    }
    let mut out = vec![0; width * height * 4];
    for (index, block) in data
        .chunks_exact(block_bytes)
        .take(columns * rows)
        .enumerate()
    {
        let (left, top) = ((index % columns) * 4, (index / columns) * 4);
        let texels = decode_block(format, block);
        for y in 0..4.min(height - top) {
            for x in 0..4.min(width - left) {
                let to = ((top + y) * width + left + x) * 4;
                out[to..to + 4].copy_from_slice(&texels[y * 4 + x]);
            }
        }
    }
    Ok(out)
}

fn decode_block(format: TextureFormat, block: &[u8]) -> [[u8; 4]; 16] {
    let mut texels = [[0, 0, 0, 255]; 16];
    match format {
        TextureFormat::Bc4RUnorm => {
            let mut red = [0; 16];
            bcdec_rs::bc4(block, &mut red, 4, false);
            for (texel, value) in texels.iter_mut().zip(red) {
                texel[0] = value;
            }
        }
        TextureFormat::Bc5RgUnorm => {
            let mut red_green = [0; 32];
            bcdec_rs::bc5(block, &mut red_green, 8, false);
            for (texel, value) in texels.iter_mut().zip(red_green.as_chunks::<2>().0) {
                texel[..2].copy_from_slice(value);
            }
        }
        _ => {
            let mut rgba = [0; 64];
            bcdec_rs::bc7(block, &mut rgba, 16);
            for (texel, value) in texels.iter_mut().zip(rgba.as_chunks::<4>().0) {
                texel.copy_from_slice(value);
            }
        }
    }
    texels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(side: usize, texel: [u8; 4]) -> Vec<u8> {
        texel.repeat(side * side)
    }

    #[test]
    fn chain_lengths_follow_the_side_and_the_cell() {
        assert_eq!(full_chain_levels(2048), 12);
        assert_eq!(atlas_chain_levels(2048), 8);
        assert_eq!(atlas_chain_levels(256), 5);
        assert_eq!(atlas_chain_levels(16), 1);
        assert_eq!(atlas_chain_levels(4), 1);
    }

    #[test]
    fn halving_averages_colour_in_linear_light_and_alpha_as_stored() {
        let level = [
            0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
        ];
        let srgb = halved(&level, 2, 2, true);
        assert_eq!(srgb, vec![188, 188, 188, 128]);
        let linear = halved(&level, 2, 2, false);
        assert_eq!(linear, vec![128, 128, 128, 128]);
    }

    #[test]
    fn every_map_survives_a_round_trip_with_its_chain() {
        for kind in MapKind::ALL {
            let bytes = encode_ktx2(kind, 16, 16, solid(16, [200, 100, 50, 255]), 5).unwrap();
            assert_eq!(ktx2_size(&bytes).unwrap(), (16, 16));
            assert_eq!(
                load_ktx2(kind, &bytes)
                    .unwrap()
                    .texture_descriptor
                    .mip_level_count,
                5
            );
            let (width, height, rgba) = decode_ktx2(kind, &bytes).unwrap();
            assert_eq!((width, height, rgba.len()), (16, 16, 16 * 16 * 4));
            assert!(rgba[0].abs_diff(200) <= 2, "{kind:?}: {:?}", &rgba[..4]);
        }
    }

    #[test]
    fn colour_stored_under_zero_alpha_is_kept() {
        let bytes =
            encode_ktx2(MapKind::Transmission, 8, 8, solid(8, [180, 90, 30, 0]), 1).unwrap();
        let (_, _, rgba) = decode_ktx2(MapKind::Transmission, &bytes).unwrap();
        assert!(rgba[0].abs_diff(180) <= 2, "{:?}", &rgba[..4]);
        assert!(rgba[3] <= 2, "{:?}", &rgba[..4]);
    }

    #[test]
    fn a_file_of_another_maps_format_is_refused() {
        let depth = encode_ktx2(MapKind::Depth, 8, 8, solid(8, [9, 9, 9, 255]), 1).unwrap();
        assert!(decode_ktx2(MapKind::BaseColor, &depth).is_err());
    }
}
