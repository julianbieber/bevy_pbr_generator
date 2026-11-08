//! Export's writer followed by the real `bevy_pbr_combine` binary, with the results
//! loaded the way a game loads them.

use std::path::{Path, PathBuf};
use std::process::Command;

use bevy_pbr_generator::gpu::export::write_map;
use bevy_pbr_generator::gpu::maps::MapKind;
use bevy_pbr_generator::map_file::{decode_ktx2, load_ktx2, MapFormat};

const SIDE: u32 = 256;

fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("exported-mips-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn readback(colour: [f32; 4]) -> Vec<u8> {
    let texel: Vec<u8> = colour
        .iter()
        .flat_map(|value| half::f16::from_f32(*value).to_le_bytes())
        .collect();
    texel.repeat((SIDE * SIDE) as usize)
}

fn export(root: &Path, name: &str, colour: [f32; 4], format: MapFormat) -> PathBuf {
    let folder = root.join(name);
    for kind in MapKind::ALL {
        let file = folder.join(format!("{}.{}", kind.file_stem(), format.extension()));
        write_map(&file, SIDE, kind, format, &readback(colour)).unwrap();
    }
    folder
}

fn combine(folders: &[PathBuf], output: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_bevy_pbr_combine"))
        .args(folders)
        .arg("-o")
        .arg(output)
        .status()
        .unwrap();
    assert!(status.success());
}

fn mip_levels(kind: MapKind, file: &Path) -> u32 {
    load_ktx2(kind, &std::fs::read(file).unwrap())
        .unwrap()
        .texture_descriptor
        .mip_level_count
}

#[test]
fn export_and_combine_load_with_mip_chains() {
    let root = scratch("chains");
    let water = export(&root, "water", [1.0, 0.0, 0.0, 1.0], MapFormat::Ktx2);
    let rock = export(&root, "rock", [0.0, 1.0, 0.0, 1.0], MapFormat::Ktx2);
    let out = root.join("out");
    combine(&[water.clone(), rock], &out);

    for kind in MapKind::ALL {
        let name = format!("{}.ktx2", kind.file_stem());
        assert_eq!(mip_levels(kind, &water.join(&name)), 9, "{name}");
        assert_eq!(mip_levels(kind, &out.join(&name)), 5, "{name}");
    }
    let image = load_ktx2(
        MapKind::BaseColor,
        &std::fs::read(out.join("base_color.ktx2")).unwrap(),
    )
    .unwrap();
    assert_eq!((image.width(), image.height()), (SIDE * 2, SIDE));
}

#[test]
fn the_last_atlas_level_keeps_its_columns_apart() {
    let root = scratch("columns");
    let water = export(&root, "water", [1.0, 0.0, 0.0, 1.0], MapFormat::Ktx2);
    let rock = export(&root, "rock", [0.0, 1.0, 0.0, 1.0], MapFormat::Ktx2);
    let out = root.join("out");
    combine(&[water, rock], &out);

    let bytes = std::fs::read(out.join("orm.ktx2")).unwrap();
    let (width, height, _) = decode_ktx2(MapKind::Orm, &bytes).unwrap();
    assert_eq!((width, height), (512, 256));
    let reader = ktx2::Reader::new(&bytes[..]).unwrap();
    assert_eq!(reader.header().level_count, 5);

    let image = load_ktx2(MapKind::Orm, &bytes).unwrap();
    let data = image.data.unwrap();
    let last_level_blocks = (32 / 4) * (16 / 4);
    let last = &data[data.len() - last_level_blocks * 16..];
    let decoded = decode_small_level(last, 32, 16);
    for y in 0..16 {
        for x in 0..32 {
            let texel = &decoded[(y * 32 + x) * 4..][..4];
            let (bright, dark) = if x < 16 { (0, 1) } else { (1, 0) };
            assert!(texel[bright] >= 250, "({x}, {y}): {texel:?}");
            assert!(texel[dark] <= 5, "({x}, {y}): {texel:?}");
        }
    }
}

fn decode_small_level(blocks: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = vec![0; width * height * 4];
    for (index, block) in blocks.as_chunks::<16>().0.iter().enumerate() {
        let (left, top) = ((index % (width / 4)) * 4, (index / (width / 4)) * 4);
        let mut texels = [0; 64];
        bcdec_rs::bc7(block, &mut texels, 16);
        for y in 0..4 {
            let to = ((top + y) * width + left) * 4;
            out[to..to + 16].copy_from_slice(&texels[y * 16..y * 16 + 16]);
        }
    }
    out
}

#[test]
fn png_exports_combine_to_png() {
    let root = scratch("png");
    let water = export(&root, "water", [1.0, 0.0, 0.0, 1.0], MapFormat::Png);
    let rock = export(&root, "rock", [0.0, 1.0, 0.0, 1.0], MapFormat::Png);
    let out = root.join("out");
    combine(&[water, rock], &out);
    let base = image::open(out.join("base_color.png"))
        .unwrap()
        .into_rgba8();
    assert_eq!(base.dimensions(), (SIDE * 2, SIDE));
    assert!(!out.join("base_color.ktx2").exists());
}
