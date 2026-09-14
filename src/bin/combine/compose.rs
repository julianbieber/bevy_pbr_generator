//! Reading material export folders and writing the combined maps built from
//! them.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use bevy_pbr_generator::gpu::maps::{MapKind, MAP_COUNT};
use image::codecs::png::PngDecoder;
use image::imageops::{self, FilterType};
use image::{ColorType, ImageDecoder, ImageFormat, RgbaImage};

use crate::layout::Layout;
use crate::CombineError;

/// One input folder, checked but not decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    pub name: String,
    /// Canonical path, so two spellings of one folder compare equal.
    pub folder: PathBuf,
    /// Side length of each map, in `MapKind::ALL` order.
    pub sizes: [u32; MAP_COUNT],
}

/// Checks every folder from its PNG headers alone, in the order given.
///
/// A material is named after its folder's final path component, resolved
/// through the filesystem so `.` and trailing slashes name the real folder.
/// Nothing is decoded, so this is cheap however large the maps are.
///
/// Errors with `FolderNotFound`, `UnusableName` when the final component is
/// missing or not UTF-8, `DuplicateName` when two folders share a name, and
/// for each map `MissingMap`, `Unreadable`, `NotRgba8` or `NotSquare`.
pub fn inspect(folders: &[PathBuf]) -> Result<Vec<Material>, CombineError> {
    let mut materials: Vec<Material> = Vec::with_capacity(folders.len());
    for folder in folders {
        let folder = folder
            .canonicalize()
            .map_err(|_| CombineError::FolderNotFound(folder.clone()))?;
        let name = folder
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| CombineError::UnusableName(folder.clone()))?
            .to_string();
        if materials.iter().any(|material| material.name == name) {
            return Err(CombineError::DuplicateName(name));
        }
        let mut sizes = [0; MAP_COUNT];
        for (size, kind) in sizes.iter_mut().zip(MapKind::ALL) {
            *size = map_size(&folder, kind)?;
        }
        materials.push(Material {
            name,
            folder,
            sizes,
        });
    }
    Ok(materials)
}

fn map_size(folder: &Path, kind: MapKind) -> Result<u32, CombineError> {
    let file = map_path(folder, kind);
    let reader = File::open(&file).map_err(|_| CombineError::MissingMap {
        folder: folder.to_path_buf(),
        file: file.clone(),
    })?;
    let decoder =
        PngDecoder::new(BufReader::new(reader)).map_err(|error| CombineError::Unreadable {
            file: file.clone(),
            reason: error.to_string(),
        })?;
    let (width, height) = decoder.dimensions();
    let color = decoder.color_type();
    if color != ColorType::Rgba8 {
        return Err(CombineError::NotRgba8 { file, color });
    }
    if width != height {
        return Err(CombineError::NotSquare {
            file,
            width,
            height,
        });
    }
    Ok(width)
}

fn map_path(folder: &Path, kind: MapKind) -> PathBuf {
    folder.join(format!("{}.png", kind.file_stem()))
}

/// The cell size every map is brought to: the largest map of any material, or
/// 0 for no materials.
pub fn cell_size(materials: &[Material]) -> u32 {
    materials
        .iter()
        .flat_map(|material| material.sizes)
        .max()
        .unwrap_or(0)
}

/// Errors with `OutputIsInput` when `output` is one of the material folders,
/// which writing would overwrite. An output directory that does not exist yet
/// cannot be one.
pub fn check_output(output: &Path, materials: &[Material]) -> Result<(), CombineError> {
    let Ok(output) = output.canonicalize() else {
        return Ok(());
    };
    match materials.iter().find(|material| material.folder == output) {
        Some(material) => Err(CombineError::OutputIsInput(material.folder.clone())),
        None => Ok(()),
    }
}

/// Builds the combined image for one map kind: each material's map, resized up
/// to the layout's cell when smaller, copied unchanged into its cell. Cells
/// without a material stay transparent black.
///
/// Decodes one material at a time, so memory holds the canvas and a single
/// map. `materials` must be the list the layout was placed for. Errors with
/// `Unreadable` when a map fails to decode.
pub fn compose(
    kind: MapKind,
    materials: &[Material],
    layout: &Layout,
) -> Result<RgbaImage, CombineError> {
    let mut canvas = RgbaImage::new(layout.width(), layout.height());
    for (index, material) in materials.iter().enumerate() {
        let file = map_path(&material.folder, kind);
        let map = image::open(&file)
            .map_err(|error| CombineError::Unreadable {
                file: file.clone(),
                reason: error.to_string(),
            })?
            .into_rgba8();
        let map = fit(map, layout.cell);
        let (x, y) = layout.origin(index);
        place(&mut canvas, &map, x, y);
    }
    Ok(canvas)
}

fn fit(map: RgbaImage, cell: u32) -> RgbaImage {
    if map.width() == cell && map.height() == cell {
        map
    } else {
        imageops::resize(&map, cell, cell, FilterType::Lanczos3)
    }
}

fn place(canvas: &mut RgbaImage, map: &RgbaImage, x: u32, y: u32) {
    let canvas_width = canvas.width() as usize;
    let map_width = map.width() as usize;
    let source = map.as_raw();
    let target: &mut [u8] = canvas;
    for row in 0..map.height() as usize {
        let from = row * map_width * 4;
        let to = ((y as usize + row) * canvas_width + x as usize) * 4;
        target[to..to + map_width * 4].copy_from_slice(&source[from..from + map_width * 4]);
    }
}

/// Writes one combined PNG per map kind and the manifest into `output`,
/// creating the directory if needed.
///
/// Every file goes to a `.tmp` sibling first and is renamed into place only
/// once all of them were written. A decode or write failure before the renames
/// leaves the files of an earlier combine untouched and no `.tmp` files behind,
/// though `output` itself may have been created. A rename that fails part-way
/// can leave a mix of old and new files. Errors with `Unreadable` from
/// composing, or `WriteFailed` naming the file.
pub fn write_all(
    output: &Path,
    materials: &[Material],
    layout: &Layout,
    manifest_name: &str,
    manifest: &str,
) -> Result<(), CombineError> {
    std::fs::create_dir_all(output).map_err(|error| CombineError::WriteFailed {
        file: output.to_path_buf(),
        reason: error.to_string(),
    })?;
    let mut staged: Vec<(PathBuf, PathBuf)> = Vec::with_capacity(MAP_COUNT + 1);
    let result = stage(
        output,
        materials,
        layout,
        manifest_name,
        manifest,
        &mut staged,
    )
    .and_then(|()| commit(&staged));
    if result.is_err() {
        for (temporary, _) in &staged {
            let _ = std::fs::remove_file(temporary);
        }
    }
    result
}

fn stage(
    output: &Path,
    materials: &[Material],
    layout: &Layout,
    manifest_name: &str,
    manifest: &str,
    staged: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), CombineError> {
    for kind in MapKind::ALL {
        let canvas = compose(kind, materials, layout)?;
        let file = output.join(format!("{}.png", kind.file_stem()));
        let temporary = temporary_path(&file);
        staged.push((temporary.clone(), file.clone()));
        canvas
            .save_with_format(&temporary, ImageFormat::Png)
            .map_err(|error| CombineError::WriteFailed {
                file,
                reason: error.to_string(),
            })?;
    }
    let file = output.join(manifest_name);
    let temporary = temporary_path(&file);
    staged.push((temporary.clone(), file.clone()));
    std::fs::write(&temporary, manifest).map_err(|error| CombineError::WriteFailed {
        file,
        reason: error.to_string(),
    })
}

fn commit(staged: &[(PathBuf, PathBuf)]) -> Result<(), CombineError> {
    for (temporary, file) in staged {
        std::fs::rename(temporary, file).map_err(|error| CombineError::WriteFailed {
            file: file.clone(),
            reason: error.to_string(),
        })?;
    }
    Ok(())
}

fn temporary_path(file: &Path) -> PathBuf {
    let mut name = file.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

/// Tests for reading and writing, and the export-folder fixtures the binary's
/// other tests build on.
#[cfg(test)]
pub mod tests {
    use super::*;
    use image::Rgba;

    /// A fresh, empty directory under the system temp dir, unique to `label`
    /// and this process.
    pub fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("combine-test-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Writes a complete export folder: every map a solid `colour` at `size`.
    pub fn export_folder(parent: &Path, name: &str, size: u32, colour: [u8; 4]) -> PathBuf {
        let folder = parent.join(name);
        std::fs::create_dir_all(&folder).unwrap();
        for kind in MapKind::ALL {
            RgbaImage::from_pixel(size, size, Rgba(colour))
                .save(map_path(&folder, kind))
                .unwrap();
        }
        folder
    }

    // A map copied into a cell must land at the cell's origin and nowhere else.
    #[test]
    fn place_copies_into_the_cell_only() {
        let mut canvas = RgbaImage::new(4, 2);
        let map = RgbaImage::from_pixel(2, 2, Rgba([9, 8, 7, 6]));
        place(&mut canvas, &map, 2, 0);
        assert_eq!(canvas.get_pixel(1, 1), &Rgba([0, 0, 0, 0]));
        assert_eq!(canvas.get_pixel(2, 0), &Rgba([9, 8, 7, 6]));
        assert_eq!(canvas.get_pixel(3, 1), &Rgba([9, 8, 7, 6]));
    }

    // Maps already at the cell size are copied verbatim; only smaller ones are resampled.
    #[test]
    fn fit_resizes_only_smaller_maps() {
        let map = RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 4]));
        assert_eq!(fit(map.clone(), 2), map);
        let grown = fit(map, 8);
        assert_eq!(grown.dimensions(), (8, 8));
        assert_eq!(grown.get_pixel(4, 4), &Rgba([1, 2, 3, 4]));
    }

    // The folder name is the material name, and a missing map names its folder and file.
    #[test]
    fn inspect_names_materials_and_reports_missing_maps() {
        let root = scratch("inspect");
        let water = export_folder(&root, "water", 4, [0, 0, 255, 255]);
        let materials = inspect(&[water.join(".")]).unwrap();
        assert_eq!(materials[0].name, "water");
        assert_eq!(cell_size(&materials), 4);

        std::fs::remove_file(water.join("orm.png")).unwrap();
        match inspect(std::slice::from_ref(&water)) {
            Err(CombineError::MissingMap { file, .. }) => assert!(file.ends_with("orm.png")),
            other => panic!("expected MissingMap, got {other:?}"),
        }
    }

    // Two folders with the same final name would be indistinguishable in the manifest.
    #[test]
    fn duplicate_folder_names_are_refused() {
        let root = scratch("duplicate");
        let a = export_folder(&root.join("a"), "sand", 2, [1, 1, 1, 255]);
        let b = export_folder(&root.join("b"), "sand", 2, [2, 2, 2, 255]);
        assert!(matches!(
            inspect(&[a, b]),
            Err(CombineError::DuplicateName(name)) if name == "sand"
        ));
    }

    // Export only ever writes square RGBA8 maps; anything else is refused before decoding.
    #[test]
    fn non_square_and_non_rgba8_maps_are_refused() {
        let root = scratch("formats");
        let folder = export_folder(&root, "rock", 4, [5, 5, 5, 255]);
        RgbaImage::new(4, 2).save(folder.join("depth.png")).unwrap();
        assert!(matches!(
            inspect(std::slice::from_ref(&folder)),
            Err(CombineError::NotSquare {
                width: 4,
                height: 2,
                ..
            })
        ));
        image::RgbImage::new(4, 4)
            .save(folder.join("depth.png"))
            .unwrap();
        assert!(matches!(
            inspect(&[folder]),
            Err(CombineError::NotRgba8 { .. })
        ));
    }

    // Writing into an input folder would overwrite that material's own maps.
    #[test]
    fn output_inside_an_input_folder_is_refused() {
        let root = scratch("output-is-input");
        let grass = export_folder(&root, "grass", 2, [0, 255, 0, 255]);
        let materials = inspect(std::slice::from_ref(&grass)).unwrap();
        assert!(matches!(
            check_output(&grass, &materials),
            Err(CombineError::OutputIsInput(_))
        ));
        assert!(check_output(&root.join("combined"), &materials).is_ok());
    }

    // A failure part-way leaves no staged files and no partial set in the output.
    #[test]
    fn failed_decode_writes_nothing() {
        let root = scratch("atomic");
        let water = export_folder(&root, "water", 2, [0, 0, 255, 255]);
        let materials = inspect(std::slice::from_ref(&water)).unwrap();
        std::fs::write(water.join("depth.png"), b"not a png").unwrap();
        let output = root.join("combined");
        let layout = Layout::place(1, 2, 8192).unwrap();
        assert!(write_all(&output, &materials, &layout, "atlas.json", "{}").is_err());
        assert_eq!(std::fs::read_dir(&output).unwrap().count(), 0);
    }
}
