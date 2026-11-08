//! The code the editor copies, compiled and spawned against the bevy this repo
//! builds with.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy_pbr_generator::app::preview::{bind_maps, preview_material};
use bevy_pbr_generator::codegen::material_code;
use bevy_pbr_generator::map_file::MapFormat;

mod generated {
    include!("generated/material_golden_2.rs");
}

#[test]
fn generated_code_matches_the_checked_in_copy() {
    assert_eq!(
        material_code("Golden 2", MapFormat::Ktx2),
        include_str!("generated/material_golden_2.rs")
    );
}

#[test]
fn spawned_material_matches_the_preview() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin))
        .init_asset::<Image>()
        .init_asset::<StandardMaterial>();

    let entity = app
        .world_mut()
        .spawn_scene(generated::material_golden_2())
        .unwrap()
        .id();
    let world = app.world();
    let handle = &world
        .get::<MeshMaterial3d<StandardMaterial>>(entity)
        .unwrap()
        .0;
    let spawned = world
        .resource::<Assets<StandardMaterial>>()
        .get(handle)
        .unwrap();

    let server = world.resource::<AssetServer>();
    let mut expected = preview_material();
    bind_maps(&mut expected, |kind| {
        Some(server.load(format!("golden_2/{}.ktx2", kind.file_stem())))
    });

    for (index, (name, field)) in expected.iter_fields().enumerate() {
        let actual = spawned.field_at(index).unwrap();
        match field.try_downcast_ref::<Option<Handle<Image>>>() {
            Some(expected_texture) => {
                let actual_texture = actual.try_downcast_ref::<Option<Handle<Image>>>().unwrap();
                assert_eq!(
                    expected_texture.as_ref().and_then(|h| h.path().cloned()),
                    actual_texture.as_ref().and_then(|h| h.path().cloned()),
                    "{name}"
                );
            }
            None => {
                if let Some(equal) = field.reflect_partial_eq(actual) {
                    assert!(equal, "{name}");
                }
            }
        }
    }
}

#[test]
fn png_mode_loads_the_png_files() {
    let code = material_code("Golden 2", MapFormat::Png);
    assert!(code.contains("golden_2/base_color.png"));
    assert!(!code.contains(".ktx2"));
}
