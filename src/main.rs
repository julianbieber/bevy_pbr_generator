use bevy::{
    app::AppExit,
    asset::AssetPlugin,
    prelude::*,
    render::error_handler::{ErrorType, RenderError, RenderErrorHandler, RenderErrorPolicy},
};

mod editor;
mod gpu;
mod material;
mod preview;
mod ui;

use editor::{Editor, SharedStatus};

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
                        resolution: (1600u32, 900u32).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.06)))
        .insert_resource(RenderErrorHandler(survive_shader_errors))
        .init_resource::<Editor>()
        .init_resource::<SharedStatus>()
        .add_plugins((
            material::MaterialCatalogPlugin,
            gpu::GenerationPlugin,
            gpu::export::ExportPlugin,
            preview::PreviewPlugin,
            ui::EditorUiPlugin,
        ))
        .run();
}

fn survive_shader_errors(
    error: &RenderError,
    main_world: &mut World,
    _render_world: &mut World,
) -> RenderErrorPolicy {
    match error.ty {
        ErrorType::Validation => RenderErrorPolicy::Ignore,
        _ => {
            main_world.write_message(AppExit::error());
            RenderErrorPolicy::StopRendering
        }
    }
}
