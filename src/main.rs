//! Entry point: the editor, opened on a material project, and the commands that
//! set one up.

use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::theme::UiTheme;
use bevy::feathers::FeathersPlugins;
use bevy::prelude::*;
use bevy::render::error_handler::{ErrorType, RenderError, RenderErrorHandler, RenderErrorPolicy};

use bevy_pbr_generator::app::EditorPlugin;
use bevy_pbr_generator::gpu::GpuPlugin;
use bevy_pbr_generator::material::project::MaterialProject;
use bevy_pbr_generator::scaffold::{self, VENDOR_DIR};
use bevy_pbr_generator::shaders::PACKAGE;
use bevy_pbr_generator::ui::EditorUiPlugin;

const USAGE: &str = "usage:
  bevy_pbr_generator [<dir>]             open the editor on the material project in <dir>
  bevy_pbr_generator init [<dir>]        make <dir> a material project, or refresh its
                                         copy of the bevy_pbr_generator package
  bevy_pbr_generator new <name> [<dir>]  write <name>.wesl from the material template

<dir> defaults to the current directory.";

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let args: Vec<&str> = match args.iter().map(|a| a.to_str()).collect() {
        Some(args) => args,
        None => return fail("arguments must be UTF-8"),
    };
    match args.as_slice() {
        ["-h" | "--help"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        ["init"] => init(Path::new(".")),
        ["init", dir] => init(Path::new(dir)),
        ["new", name] => new_material(Path::new("."), name),
        ["new", name, dir] => new_material(Path::new(dir), name),
        [] => edit(Path::new(".")),
        [dir] if !matches!(*dir, "init" | "new") && !dir.starts_with('-') => edit(Path::new(dir)),
        _ => fail(USAGE),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

fn init(dir: &Path) -> ExitCode {
    let report = match scaffold::init(dir) {
        Ok(report) => report,
        Err(e) => return fail(&e.to_string()),
    };
    if let Some(toml) = &report.created_toml {
        println!("wrote {}", toml.display());
    }
    println!(
        "copied the {PACKAGE} package to {}",
        report.vendored.display()
    );
    if report.dependency_mismatch {
        eprintln!(
            "warning: `{}` does not declare `{PACKAGE} = {{ path = \"{VENDOR_DIR}\" }}`, \
             so the language server does not read the copy the editor runs",
            dir.join("wesl.toml").display()
        );
    }
    ExitCode::SUCCESS
}

fn new_material(dir: &Path, name: &str) -> ExitCode {
    match scaffold::new_material(dir, name) {
        Ok(path) => {
            println!("wrote {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => fail(&e.to_string()),
    }
}

fn edit(dir: &Path) -> ExitCode {
    let project = match MaterialProject::open(dir) {
        Ok(project) => project,
        Err(e) => return fail(&format!("{e}\n\n{USAGE}")),
    };
    for name in project.foreign_dependencies() {
        eprintln!(
            "warning: `{}` declares the dependency `{name}`, which the editor does not provide; \
             a material importing it will not compile",
            project.toml_dir.join("wesl.toml").display()
        );
    }
    if scaffold::package_is_stale(&project) {
        eprintln!(
            "warning: the {PACKAGE} package the language server reads differs from the one \
             the editor runs; `bevy_pbr_generator init` refreshes it"
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
    ExitCode::SUCCESS
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
