//! Entry point: opens the editor on the material project named on the command
//! line, or on the example project when none is.

use std::path::PathBuf;

use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::theme::UiTheme;
use bevy::feathers::FeathersPlugins;
use bevy::prelude::*;
use bevy::render::error_handler::{ErrorType, RenderError, RenderErrorHandler, RenderErrorPolicy};

use bevy_pbr_generator::app::EditorPlugin;
use bevy_pbr_generator::gpu::GpuPlugin;
use bevy_pbr_generator::material::project::MaterialProject;
use bevy_pbr_generator::ui::EditorUiPlugin;

fn main() {
    let dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("example"));
    let project = match MaterialProject::open(&dir) {
        Ok(project) => project,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    for name in project.foreign_dependencies() {
        eprintln!(
            "warning: `{}` declares the dependency `{name}`, which the editor does not provide; \
             a material importing it will not compile",
            project.toml_dir.join("wesl.toml").display()
        );
    }

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: project.package_root.to_string_lossy().into_owned(),
                    meta_check: AssetMetaCheck::Never,
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy_pbr_generator".to_string(),
                        resolution: (1600u32, 1000u32).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(FeathersPlugins)
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(RenderErrorHandler(tolerate_shader_errors))
        .add_plugins((GpuPlugin, EditorPlugin { project }, EditorUiPlugin))
        .run();
}

/// Keeps a bad user shader from ending the session.
///
/// The default policy quits the app on any render error, which would make a
/// typo in a hot-reloaded material fatal — the opposite of what hot reload is
/// for. Everything else still stops rendering.
fn tolerate_shader_errors(
    error: &RenderError,
    _main_world: &mut World,
    _render_world: &mut World,
) -> RenderErrorPolicy {
    match error.ty {
        ErrorType::Validation => RenderErrorPolicy::Ignore,
        _ => RenderErrorPolicy::StopRendering,
    }
}
