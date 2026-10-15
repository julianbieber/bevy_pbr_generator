//! Compose-time checks over the `bevy_pbr_generator` WESL package and the
//! example project's materials, run through bevy's own shader cache.

use std::path::Path;

use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::asset::uuid::Uuid;
use bevy::asset::AssetId;
use bevy::prelude::*;
use bevy::shader::{Shader, ShaderCache, ShaderCacheError, ShaderCacheSource, ValidateShader};
use bevy_pbr_generator::material::catalog::declared_name;
use bevy_pbr_generator::material::params::{parse_params, PARAMS_BUFFER_SIZE};
use bevy_pbr_generator::material::project::{MaterialProject, ProjectFile};
use bevy_pbr_generator::shaders::{embed, entry_asset_path, entry_module_source, MODULE_ASSETS};

type Cache = ShaderCache<String, ()>;

fn to_wgsl(
    _: &(),
    source: ShaderCacheSource,
    _: &ValidateShader,
) -> Result<String, ShaderCacheError> {
    match source {
        ShaderCacheSource::Wgsl(wgsl) => Ok(wgsl),
        _ => Err(ShaderCacheError::ProcessShaderError("not WGSL".into())),
    }
}

fn id(n: usize) -> AssetId<Shader> {
    AssetId::Uuid {
        uuid: Uuid::from_u128(n as u128 + 1),
    }
}

struct Composed {
    cache: Cache,
    project: Vec<ProjectFile>,
}

impl Composed {
    fn example() -> Self {
        let mut cache = Cache::new((), to_wgsl);
        for (n, asset) in MODULE_ASSETS.iter().enumerate() {
            let file = asset.trim_start_matches("embedded://bevy_pbr_generator/");
            let source = std::fs::read_to_string(Path::new("src/shaders").join(file))
                .expect("package module should be readable");
            cache.set_shader(id(n), Shader::from_wesl(source, *asset));
        }
        let project = MaterialProject::open(Path::new("example")).expect("example project");
        let (files, warnings) = project.files().expect("example project should scan");
        assert!(warnings.is_empty(), "{warnings:?}");
        for (n, file) in files.iter().enumerate() {
            cache.set_shader(
                id(100 + n),
                Shader::from_wesl(file.source.clone(), file.asset_path()),
            );
        }
        Self {
            cache,
            project: files,
        }
    }

    fn materials(&self) -> impl Iterator<Item = (usize, &ProjectFile)> {
        self.project
            .iter()
            .enumerate()
            .filter(|(_, f)| !declared_name(&f.source, "").is_empty())
    }

    fn entry(&mut self, n: usize) -> Result<naga::Module, String> {
        let file = &self.project[n];
        let entry = Shader::from_wesl(
            entry_module_source(&file.components),
            entry_asset_path(&file.components),
        );
        self.cache.set_shader(id(1000 + n), entry);
        self.module(1000 + n)
    }

    fn module(&mut self, n: usize) -> Result<naga::Module, String> {
        let wgsl = self.cache.get(n, id(n), &[]).map_err(|e| e.to_string())?;
        naga::front::wgsl::parse_str(&wgsl).map_err(|e| e.emit_to_string(&wgsl))
    }
}

// Materials name the library `bevy_pbr_generator::<module>`, which bevy derives
// from the embedded asset path; the default `embedded_asset!` form would put a
// `shaders` segment in it and no import would resolve.
#[test]
fn package_modules_embed_at_the_paths_bevy_names_as_the_package() {
    let mut app = App::new();
    app.init_resource::<EmbeddedAssetRegistry>();
    embed(&mut app);
    let registry = app.world().resource::<EmbeddedAssetRegistry>();
    for asset in MODULE_ASSETS {
        let path = asset.trim_start_matches("embedded://");
        assert!(
            registry.remove_asset(Path::new(path)).is_some(),
            "nothing embedded at {path}"
        );
    }
}

// Pins the whole material contract as bevy composes it: the entry module the
// editor writes resolves the material by its module path, the material resolves
// the package, and `generate` survives composition. The template is included,
// since it is the file a new material starts from.
#[test]
fn every_example_material_composes_through_its_entry_module() {
    let mut composed = Composed::example();
    let materials: Vec<usize> = composed.materials().map(|(n, _)| n).collect();
    assert!(
        materials.len() >= 3,
        "expected water, rocky and the template"
    );
    for n in materials {
        let name = composed.project[n].module_path();
        let module = composed.entry(n).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            module.entry_points.iter().any(|e| e.name == "generate"),
            "{name}: composed entry module has no `generate` entry point"
        );
    }
}

// Only the generated entry module may declare an entry point; one left in a
// material would be stripped silently under the entry and never noticed.
#[test]
fn no_example_material_declares_an_entry_point() {
    let mut composed = Composed::example();
    let materials: Vec<usize> = composed.materials().map(|(n, _)| n).collect();
    for n in materials {
        let name = composed.project[n].module_path();
        let module = composed
            .module(100 + n)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            module.entry_points.is_empty(),
            "{name}: declares an entry point"
        );
    }
}

// The compute pass writes parameters at offsets `params.rs` computes by hand,
// but the shader reads them at offsets naga computes; a disagreement would
// corrupt every parameter with no error anywhere.
#[test]
fn parsed_offsets_agree_with_naga() {
    let mut composed = Composed::example();
    let materials: Vec<usize> = composed.materials().map(|(n, _)| n).collect();
    for n in materials {
        let name = composed.project[n].module_path();
        let layout = parse_params(&composed.project[n].source).expect("Params block should parse");
        let module = composed.entry(n).unwrap_or_else(|e| panic!("{name}: {e}"));

        let names: Vec<&str> = layout.fields.iter().map(|f| f.name.as_str()).collect();
        let members = module
            .types
            .iter()
            .find_map(|(_, ty)| match &ty.inner {
                naga::TypeInner::Struct { members, .. } => {
                    let found: Vec<&str> =
                        members.iter().filter_map(|m| m.name.as_deref()).collect();
                    (found == names).then_some(members)
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!("{name}: no struct in the composed module matches {names:?}")
            });

        for (field, member) in layout.fields.iter().zip(members) {
            assert_eq!(
                field.offset as u32, member.offset,
                "{name}: `{}` is at {} but naga puts it at {}",
                field.name, field.offset, member.offset
            );
        }
        assert!(
            layout.size <= PARAMS_BUFFER_SIZE,
            "{name}: Params is {} bytes, over the binding size",
            layout.size
        );
    }
}
