//! The materials found on disk, which one is being edited, and the live
//! parameter values for each.

use std::path::{Path, PathBuf};

use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy::shader::Shader;

use crate::material::params::{parse_params, ParamsLayout};

/// Directory, relative to the asset root, scanned for material shaders.
pub const MATERIALS_DIR: &str = "materials";

/// One material shader on disk, with the parameters parsed out of it.
///
/// `layout` is always the last one that parsed successfully, so a material whose
/// `error` is set still renders with the parameters it had before the failing
/// edit.
#[derive(Debug, Clone)]
pub struct MaterialSpec {
    /// Display name, from the `// @material` header or the file stem.
    pub name: String,
    pub asset_path: String,
    pub disk_path: PathBuf,
    pub shader: Handle<Shader>,
    pub layout: ParamsLayout,
    pub values: Vec<[f32; 4]>,
    /// Why the most recent parse of this file failed, if it did.
    pub error: Option<String>,
}

impl MaterialSpec {
    /// The parameter buffer for the current values, ready to write to the GPU.
    pub fn packed(&self) -> Vec<u8> {
        self.layout.pack(&self.values)
    }

    /// Replaces the layout with a freshly parsed one, carrying over the value of
    /// every field whose name and type survived the edit so that a shader tweak
    /// does not reset the parameters the user has dialled in.
    pub fn adopt(&mut self, layout: ParamsLayout) {
        let mut values = layout.defaults();
        for (index, field) in layout.fields.iter().enumerate() {
            let previous = self
                .layout
                .fields
                .iter()
                .position(|f| f.name == field.name && f.ty == field.ty);
            if let Some(previous) = previous {
                if let Some(value) = self.values.get(previous) {
                    values[index] = *value;
                }
            }
        }
        self.layout = layout;
        self.values = values;
    }
}

/// Every material found on disk, and which one the editor is showing.
#[derive(Resource, Debug, Default)]
pub struct MaterialCatalog {
    pub materials: Vec<MaterialSpec>,
    pub active: usize,
}

impl MaterialCatalog {
    pub fn active(&self) -> Option<&MaterialSpec> {
        self.materials.get(self.active)
    }

    pub fn active_mut(&mut self) -> Option<&mut MaterialSpec> {
        self.materials.get_mut(self.active)
    }
}

/// The display name a material shader declares, falling back to `fallback`.
pub fn declared_name(source: &str, fallback: &str) -> String {
    source
        .lines()
        .take_while(|line| {
            let line = line.trim();
            line.is_empty() || line.starts_with("//")
        })
        .find_map(|line| {
            line.trim()
                .strip_prefix("//")
                .and_then(|c| c.trim().strip_prefix("@material "))
                .map(|name| name.trim().to_string())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

/// Whether a directory entry is a material the editor should list.
///
/// Files whose name begins with `_` are skipped, which is what keeps
/// `_template.wgsl` out of the material list.
pub fn is_material_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    path.extension().and_then(|e| e.to_str()) == Some("wgsl") && !name.starts_with('_')
}

/// Reads every material under `<asset_root>/materials`, in name order.
///
/// A file that fails to parse is still listed, with its `error` set and an empty
/// layout, so that fixing it in place brings it back without a restart.
pub fn scan_materials(asset_root: &Path, asset_server: &AssetServer) -> Vec<MaterialSpec> {
    let dir = asset_root.join(MATERIALS_DIR);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        warn!("no material directory at {}", dir.display());
        return Vec::new();
    };

    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| is_material_file(p))
        .collect();
    paths.sort();

    paths
        .into_iter()
        .map(|disk_path| {
            let stem = disk_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("material")
                .to_string();
            let asset_path = format!("{MATERIALS_DIR}/{stem}.wgsl");
            let source = std::fs::read_to_string(&disk_path).unwrap_or_default();

            let (layout, error) = match parse_params(&source) {
                Ok(layout) => (layout, None),
                Err(e) => (ParamsLayout::default(), Some(e.to_string())),
            };

            MaterialSpec {
                name: declared_name(&source, &stem),
                shader: asset_server.load(asset_path.clone()),
                values: layout.defaults(),
                layout,
                asset_path,
                disk_path,
                error,
            }
        })
        .collect()
}

/// Re-parses `spec` from disk, keeping the previous layout if the new text does
/// not parse. Returns whether anything about the material changed.
pub fn reload(spec: &mut MaterialSpec) -> bool {
    let Ok(source) = std::fs::read_to_string(&spec.disk_path) else {
        return false;
    };

    let stem = spec
        .disk_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("material")
        .to_string();
    spec.name = declared_name(&source, &stem);

    match parse_params(&source) {
        Ok(layout) => {
            let changed = layout != spec.layout || spec.error.is_some();
            if layout != spec.layout {
                spec.adopt(layout);
            }
            spec.error = None;
            changed
        }
        Err(e) => {
            let message = e.to_string();
            let changed = spec.error.as_deref() != Some(message.as_str());
            spec.error = Some(message);
            changed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The `_` prefix is the only thing keeping a template file out of the
    // material list, so it has to hold for exactly the files it should.
    #[test]
    fn underscore_prefixed_files_are_skipped() {
        assert!(is_material_file(Path::new("assets/materials/water.wgsl")));
        assert!(!is_material_file(Path::new(
            "assets/materials/_template.wgsl"
        )));
        assert!(!is_material_file(Path::new("assets/materials/notes.txt")));
    }

    // The header is what names a material in the list; without one the file
    // stem has to stand in, or the list shows a blank row.
    #[test]
    fn the_material_header_names_the_material() {
        assert_eq!(declared_name("// @material Water\n", "water"), "Water");
        assert_eq!(declared_name("// no header here\n", "water"), "water");
        assert_eq!(declared_name("", "water"), "water");
    }

    // A `@material` line below the leading comment block is part of the shader
    // body, not the header, and must not be picked up.
    #[test]
    fn only_the_leading_comment_block_is_searched() {
        let source = "// @material Real\nfn f() {}\n// @material Fake\n";
        assert_eq!(declared_name(source, "stem"), "Real");
    }
}
