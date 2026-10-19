//! The code the editor copies, compiled and spawned against the bevy this repo
//! builds with.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy_pbr_generator::app::preview::{bind_maps, preview_material};
use bevy_pbr_generator::codegen::material_code;

mod generated {
    include!("generated/material_golden_2.rs");
}

// The checked-in copy is what cargo compiles, so it has to be exactly what the
// editor copies; a change to the output then shows up as a reviewable diff.
#[test]
fn generated_code_matches_the_checked_in_copy() {
    assert_eq!(
        material_code("Golden 2"),
        include_str!("generated/material_golden_2.rs")
    );
}

// The pasted material has to equal the preview's, slot for slot and factor for
// factor, so a game shows what the editor showed.
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
        Some(server.load(format!("golden_2/{}.png", kind.file_stem())))
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
