//! The materials found in the opened project, which one is being edited, and
//! the live parameter values for each.

use std::path::{Path, PathBuf};

use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy::shader::Shader;

use crate::material::params::{parse_params, ParamsLayout};
use crate::material::project::MaterialProject;
use crate::shaders::{entry_asset_path, entry_module_source};

/// One material shader on disk, with the parameters parsed out of it.
///
/// `layout` is always the last one that parsed successfully, so a material whose
/// `error` is set still renders with the parameters it had before the failing
/// edit.
#[derive(Debug, Clone)]
pub struct MaterialSpec {
    /// Display name, from the `// @material` header or the file stem.
    pub name: String,
    /// Path of the material file relative to the project's package root.
    pub asset_path: String,
    pub disk_path: PathBuf,
    /// The module path an import names the material by, `package::…`.
    pub module_path: String,
    /// The material file itself. Held so the file stays loaded and its edits
    /// arrive as `Modified` events for this id.
    pub shader: Handle<Shader>,
    /// The generated entry module the pipeline is built from.
    pub entry: Handle<Shader>,
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
    /// `None` when `active` is out of range of `materials`, as with an empty catalog.
    pub fn active(&self) -> Option<&MaterialSpec> {
        self.materials.get(self.active)
    }

    /// `None` when `active` is out of range of `materials`, as with an empty catalog.
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

fn has_material_header(source: &str) -> bool {
    !declared_name(source, "").is_empty()
}

/// Whether a project file with this `path` and `source` is a material the
/// editor should list: a `.wesl` file whose leading comment block declares a
/// `@material` name. Files whose name begins with `_` are skipped, which is what
/// keeps `_template.wesl` out of the material list.
pub fn is_material_file(path: &Path, source: &str) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    path.extension().and_then(|e| e.to_str()) == Some("wesl")
        && !name.starts_with('_')
        && has_material_header(source)
}

/// Lists every material in `project`, in module path order, loading each file
/// as a Shader asset and adding its generated entry module to `shaders`.
///
/// A file that fails to parse is still listed, with its `error` set and an empty
/// layout, so that fixing it in place brings it back without a restart. Files
/// the project scan skipped, and a scan that fails outright, are logged.
pub fn scan_materials(
    project: &MaterialProject,
    asset_server: &AssetServer,
    shaders: &mut Assets<Shader>,
) -> Vec<MaterialSpec> {
    let files = match project.files() {
        Ok((files, warnings)) => {
            for warning in warnings {
                warn!("{warning}");
            }
            files
        }
        Err(e) => {
            warn!("{e}");
            return Vec::new();
        }
    };

    files
        .into_iter()
        .filter_map(|file| {
            let asset_path = file.asset_path();
            if !is_material_file(Path::new(&asset_path), &file.source) {
                return None;
            }
            let stem = file.components.last().cloned().unwrap_or_default();
            let (layout, error) = match parse_params(&file.source) {
                Ok(layout) => (layout, None),
                Err(e) => (ParamsLayout::default(), Some(e.to_string())),
            };
            let entry = Shader::from_wesl(
                entry_module_source(&file.components),
                entry_asset_path(&file.components),
            );

            Some(MaterialSpec {
                name: declared_name(&file.source, &stem),
                shader: asset_server.load(asset_path.clone()),
                entry: shaders.add(entry),
                module_path: file.module_path(),
                values: layout.defaults(),
                layout,
                disk_path: project.package_root.join(&asset_path),
                asset_path,
                error,
            })
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

    // A project mixes materials with its own helper modules; the header is
    // what tells them apart, and the `_` prefix is what keeps a template out.
    #[test]
    fn only_headed_files_without_underscore_are_materials() {
        let headed = "// @material Water\n";
        assert!(is_material_file(Path::new("water.wesl"), headed));
        assert!(is_material_file(Path::new("walls/stone.wesl"), headed));
        assert!(!is_material_file(Path::new("common.wesl"), "fn f() {}\n"));
        assert!(!is_material_file(Path::new("_template.wesl"), headed));
        assert!(!is_material_file(Path::new("notes.txt"), headed));
    }

    // The header is what names a material in the list; a listed material whose
    // header was edited away keeps its file stem until restart, not a blank row.
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
