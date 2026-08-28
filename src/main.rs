//! Entry point: opens the editor.

use bevy::asset::AssetPlugin;
use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::theme::UiTheme;
use bevy::feathers::FeathersPlugins;
use bevy::prelude::*;
use bevy::render::error_handler::{ErrorType, RenderError, RenderErrorHandler, RenderErrorPolicy};

use bevy_pbr_generator::app::EditorPlugin;
use bevy_pbr_generator::gpu::GpuPlugin;
use bevy_pbr_generator::ui::EditorUiPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
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
        .add_plugins((GpuPlugin, EditorPlugin, EditorUiPlugin))
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
