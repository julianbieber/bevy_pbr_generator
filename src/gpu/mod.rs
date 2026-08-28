//! Generation of the PBR maps on the GPU: the resources both worlds share, the
//! compute pipeline built per material, and the dispatch that fills the maps.

pub mod dispatch;
pub mod export;
pub mod maps;
pub mod pipeline;

use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::ShaderType;
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::Shader;

use crate::status::SharedStatus;

/// Side of one compute workgroup, matching `@workgroup_size(8, 8, 1)` in every
/// material's entry point.
pub const WORKGROUP_SIZE: u32 = 8;

/// Strong handles to the shader libraries every material imports.
///
/// Held for the life of the app because `#import` resolves against loaded
/// assets: without a live handle the import fails with "Shader import not yet
/// available" and no material ever compiles.
#[derive(Resource, Debug)]
pub struct ShaderLibrary {
    pub handles: Vec<Handle<Shader>>,
}

/// Counter bumped whenever the generated maps would come out different.
///
/// Generation is on demand rather than per frame, so every input that affects
/// the output — material selection, a parameter edit, resolution, seed, a shader
/// reload — must bump this or the change will not appear.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Generation(pub u64);

impl Generation {
    pub fn bump(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// The map set the preview mesh samples.
#[derive(Resource, Debug, Clone)]
pub struct PreviewMaps {
    pub images: Vec<Handle<Image>>,
    pub resolution: u32,
}

/// Everything the render world needs to generate one set of maps.
///
/// A request with no `shader`, or whose `images` are not all resident, is
/// skipped rather than dispatched.
#[derive(Resource, Debug, Clone, Default, ExtractResource)]
pub struct GenerateRequest {
    pub generation: u64,
    pub shader: Option<Handle<Shader>>,
    pub images: Vec<Handle<Image>>,
    pub params: Vec<u8>,
    pub resolution: u32,
    pub time: f32,
    pub seed: f32,
}

/// A pending export dispatch, at the export resolution and into its own map
/// set. Empty whenever no export is in flight.
#[derive(Resource, Debug, Clone, Default, ExtractResource)]
pub struct ExportRequest(pub GenerateRequest);

/// Group 0's uniform, identical for every material.
#[derive(ShaderType, Debug, Clone, Copy, Default)]
pub struct GpuGlobals {
    pub resolution: UVec2,
    pub time: f32,
    pub seed: f32,
}

/// Wires up map generation in both worlds.
pub struct GpuPlugin;

impl Plugin for GpuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Generation>()
            .init_resource::<GenerateRequest>()
            .init_resource::<ExportRequest>()
            .init_resource::<export::ExportJob>()
            .init_resource::<SharedStatus>()
            .add_plugins((
                ExtractResourcePlugin::<GenerateRequest>::default(),
                ExtractResourcePlugin::<ExportRequest>::default(),
            ))
            .add_systems(PreStartup, load_shader_library)
            .add_systems(Update, export::drive_export);
    }

    fn finish(&self, app: &mut App) {
        let status = app.world().resource::<SharedStatus>().clone();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(status)
            .init_resource::<pipeline::MaterialPipeline>()
            .init_resource::<dispatch::LastDispatched>()
            .add_systems(RenderStartup, pipeline::init_compute_resources)
            .add_systems(
                Render,
                (
                    pipeline::prepare_pipeline.in_set(RenderSystems::Prepare),
                    pipeline::prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
                    dispatch::dispatch_maps.in_set(RenderSystems::Render),
                ),
            );
    }
}

fn load_shader_library(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ShaderLibrary {
        handles: vec![
            asset_server.load("shaders/lib/noise.wgsl"),
            asset_server.load("shaders/lib/pbr_maps.wgsl"),
        ],
    });
}
