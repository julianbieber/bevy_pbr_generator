//! The WESL side of the material contract: the package `bevy_pbr_generator`
//! that materials import, and the entry point that calls into a material.

use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::gpu::WORKGROUP_SIZE;

/// The package name a material imports the library under, in the editor and in
/// the material project's `wesl.toml` alike.
pub const PACKAGE: &str = "bevy_pbr_generator";

/// The embedded asset path of each package module, in the form bevy maps to the
/// module path `bevy_pbr_generator::<file stem>`.
pub const MODULE_ASSETS: [&str; 2] = [
    "embedded://bevy_pbr_generator/pbr_maps.wesl",
    "embedded://bevy_pbr_generator/noise.wesl",
];

/// The first component of every entry module's asset path.
///
/// It is not a WESL identifier, so no file of a material project can map to a
/// module under it, and an entry can never displace a project module.
pub const ENTRY_ROOT: &str = "pbr-generator-entry";

/// Registers the package modules as embedded assets, ready to load from
/// [`MODULE_ASSETS`].
///
/// Must run in `Plugin::build` after `AssetPlugin`, which owns the embedded
/// registry.
pub fn embed(app: &mut App) {
    embedded_asset!(app, "src/shaders", "pbr_maps.wesl");
    embedded_asset!(app, "src/shaders", "noise.wesl");
}

/// The asset path an entry module is registered under, for the material whose
/// module path below `package::` is `components`.
pub fn entry_asset_path(components: &[String]) -> String {
    format!("{ENTRY_ROOT}/{}.wesl", components.join("/"))
}

/// The source of the entry module for the material at `package::<components>`.
///
/// It declares the `generate` compute entry point the pipeline is built with,
/// dispatched in [`WORKGROUP_SIZE`]-square workgroups, and passes the material's
/// `surface(uv)` at each texel centre to `write_surface`.
pub fn entry_module_source(components: &[String]) -> String {
    format!(
        "import package::{module}::surface;\n\
         import {PACKAGE}::pbr_maps::{{globals, write_surface}};\n\
         \n\
         @compute @workgroup_size({WORKGROUP_SIZE}, {WORKGROUP_SIZE}, 1)\n\
         fn generate(@builtin(global_invocation_id) id: vec3<u32>) {{\n\
         \x20   if id.x >= globals.resolution.x || id.y >= globals.resolution.y {{\n\
         \x20       return;\n\
         \x20   }}\n\
         \x20   let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(globals.resolution);\n\
         \x20   write_surface(id.xy, surface(uv));\n\
         }}\n",
        module = components.join("::"),
    )
}
