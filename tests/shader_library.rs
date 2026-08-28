//! Compose-time checks over the WGSL shader library and the shipped materials.

use bevy_pbr_generator::material::params::{parse_params, PARAMS_BUFFER_SIZE};
use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, NagaModuleDescriptor, ShaderLanguage, ShaderType,
};

const LIBRARIES: [&str; 2] = [
    "assets/shaders/lib/pbr_maps.wgsl",
    "assets/shaders/lib/noise.wgsl",
];

fn composer_with_libraries() -> Result<Composer, String> {
    let mut composer = Composer::default();
    for path in LIBRARIES {
        let source = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        composer
            .add_composable_module(ComposableModuleDescriptor {
                source: &source,
                file_path: path,
                language: ShaderLanguage::Wgsl,
                ..Default::default()
            })
            .map_err(|e| format!("{path}: {e:?}"))?;
    }
    Ok(composer)
}

fn compose_material(path: &str) -> Result<naga::Module, String> {
    let source = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    composer_with_libraries()?
        .make_naga_module(NagaModuleDescriptor {
            source: &source,
            file_path: path,
            shader_type: ShaderType::Wgsl,
            ..Default::default()
        })
        .map_err(|e| format!("{path}: {e:?}"))
}

// The libraries are only ever compiled by the render pipeline at runtime, where
// a syntax or type error surfaces as a shader compile failure with the app
// already open; this pins that both parse and type-check under the same
// naga_oil version bevy uses.
#[test]
fn libraries_compose() {
    if let Err(e) = composer_with_libraries() {
        panic!("{e}");
    }
}

// Pins the whole material contract: that a shipped material's imports resolve
// against the libraries, that its `surface()` type-checks against `Surface`, and
// that its `generate` entry point survives composition.
#[test]
fn shipped_materials_compose() {
    for path in ["assets/materials/water.wgsl", "assets/materials/rocky.wgsl"] {
        match compose_material(path) {
            Ok(module) => assert!(
                module.entry_points.iter().any(|e| e.name == "generate"),
                "{path}: composed module has no `generate` entry point"
            ),
            Err(e) => panic!("{e}"),
        }
    }
}

// The compute pass writes parameters at offsets `params.rs` computes by hand,
// but the shader reads them at offsets naga computes; a disagreement would
// corrupt every parameter with no error anywhere. This pins the two against
// each other for the shipped materials.
#[test]
fn parsed_offsets_agree_with_naga() {
    for path in ["assets/materials/water.wgsl", "assets/materials/rocky.wgsl"] {
        let source = std::fs::read_to_string(path).expect("material should be readable");
        let layout = parse_params(&source).expect("Params block should parse");
        let module = compose_material(path).expect("material should compose");

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
                panic!("{path}: no struct in the composed module matches {names:?}")
            });

        for (field, member) in layout.fields.iter().zip(members) {
            assert_eq!(
                field.offset as u32, member.offset,
                "{path}: `{}` is at {} but naga puts it at {}",
                field.name, field.offset, member.offset
            );
        }
        assert!(
            layout.size <= PARAMS_BUFFER_SIZE,
            "{path}: Params is {} bytes, over the binding size",
            layout.size
        );
    }
}
