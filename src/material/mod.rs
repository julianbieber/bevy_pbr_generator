pub mod params;

use bevy::asset::io::file::FileAssetReader;
use bevy::asset::AssetPath;
use bevy::prelude::*;
use bevy::shader::Shader;

use params::{parse_params, ParamLayout};

pub const ASSET_DIR: &str = "assets";
pub const MATERIAL_DIR: &str = "materials";

#[derive(Clone)]
pub struct MaterialEntry {
    pub name: String,
    pub file: String,
    pub shader: Handle<Shader>,
    pub layout: ParamLayout,
    pub error: Option<String>,
}

#[derive(Resource, Default)]
pub struct MaterialCatalog {
    pub entries: Vec<MaterialEntry>,
}

impl MaterialCatalog {
    pub fn get(&self, index: usize) -> Option<&MaterialEntry> {
        self.entries.get(index)
    }
}

#[derive(Message)]
pub struct MaterialsChanged {
    pub reparsed: Vec<usize>,
}

pub struct MaterialCatalogPlugin;

impl Plugin for MaterialCatalogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MaterialCatalog>()
            .add_message::<MaterialsChanged>()
            .add_systems(PreStartup, scan_materials)
            .add_systems(PreUpdate, reparse_loaded_materials);
    }
}

fn scan_materials(mut catalog: ResMut<MaterialCatalog>, asset_server: Res<AssetServer>) {
    let root = FileAssetReader::get_base_path()
        .join(ASSET_DIR)
        .join(MATERIAL_DIR);
    let Ok(dir) = std::fs::read_dir(&root) else {
        error!("no material directory at {}", root.display());
        return;
    };

    let mut files: Vec<String> = dir
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "wgsl"))
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .filter(|name| !name.starts_with('_'))
        .collect();
    files.sort();

    for file in files {
        let asset_path = format!("{MATERIAL_DIR}/{file}");
        catalog.entries.push(MaterialEntry {
            name: file.trim_end_matches(".wgsl").to_string(),
            shader: asset_server.load(AssetPath::from(asset_path)),
            file,
            layout: ParamLayout::default(),
            error: None,
        });
    }

    if catalog.entries.is_empty() {
        error!("no materials found in {}", root.display());
    }
}

fn reparse_loaded_materials(
    mut events: MessageReader<AssetEvent<Shader>>,
    mut catalog: ResMut<MaterialCatalog>,
    mut changed: MessageWriter<MaterialsChanged>,
    shaders: Res<Assets<Shader>>,
) {
    let mut reparsed = Vec::new();

    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };
        let Some(index) = catalog
            .entries
            .iter()
            .position(|entry| entry.shader.id() == id)
        else {
            continue;
        };
        let Some(shader) = shaders.get(id) else {
            continue;
        };

        let source = shader.source.as_str();
        let entry = &mut catalog.entries[index];
        let (layout, error) = parse_or_report(source, &entry.file);
        entry.name = display_name(source, &entry.file);
        entry.layout = layout;
        entry.error = error;
        if !reparsed.contains(&index) {
            reparsed.push(index);
        }
    }

    if !reparsed.is_empty() {
        changed.write(MaterialsChanged { reparsed });
    }
}

fn parse_or_report(source: &str, file: &str) -> (ParamLayout, Option<String>) {
    match parse_params(source) {
        Ok(layout) => (layout, None),
        Err(error) => (ParamLayout::default(), Some(format!("{file}: {error}"))),
    }
}

fn display_name(source: &str, file: &str) -> String {
    for line in source.lines().take(16) {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("//") {
            if let Some(name) = rest.trim().strip_prefix("@material") {
                let name = name.trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    file.trim_end_matches(".wgsl").to_string()
}

pub fn carry_over_values(layout: &ParamLayout, previous: &[(String, [f32; 4])]) -> Vec<[f32; 4]> {
    layout
        .params
        .iter()
        .map(|spec| {
            previous
                .iter()
                .find(|(name, _)| *name == spec.name)
                .map(|(_, value)| *value)
                .unwrap_or(spec.default)
        })
        .collect()
}
