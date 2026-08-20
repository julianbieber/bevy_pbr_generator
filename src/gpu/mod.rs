pub mod export;
pub mod maps;

use std::borrow::Cow;
use std::num::NonZeroU64;

use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::schedule::camera_driver,
    image::Image,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{texture_storage_2d, uniform_buffer_sized},
            BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
            BindingResource, BufferInitDescriptor, BufferUsages, CachedComputePipelineId,
            CachedPipelineState, ComputePassDescriptor, ComputePipelineDescriptor, Extent3d,
            PipelineCache, ShaderStages, StorageTextureAccess, TextureDimension, TextureUsages,
        },
        renderer::{RenderContext, RenderDevice, RenderGraph},
        texture::GpuImage,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
    shader::{Shader, ShaderCacheError},
};

use crate::editor::{Editor, ShaderStatus, SharedStatus};
use crate::material::MaterialCatalog;
use maps::{MAP_COUNT, MAP_FORMAT};

pub const SHADER_LIBRARY: [&str; 2] = ["shaders/lib/noise.wgsl", "shaders/lib/pbr_maps.wgsl"];
pub const WORKGROUP_SIZE: u32 = 8;
pub const ENTRY_POINT: &str = "generate";
const GLOBALS_SIZE: u64 = 16;
pub const PARAMS_BUFFER_SIZE: usize = 1024;

#[derive(Resource, Clone, ExtractResource)]
pub struct MapTextures {
    pub images: Vec<Handle<Image>>,
    pub resolution: u32,
}

#[derive(Resource, Clone, ExtractResource, Default)]
pub struct GenerationRequest {
    pub generation: u64,
    pub resolution: u32,
    pub time: f32,
    pub seed: f32,
}

#[derive(Resource)]
pub struct ShaderLibrary(#[allow(dead_code)] Vec<Handle<Shader>>);

#[derive(Resource, Default)]
pub struct LastGenerated {
    pub generation: u64,
    pub at: f64,
}

#[derive(Resource, Clone, ExtractResource, Default)]
pub struct ActiveMaterial {
    pub shader: Option<Handle<Shader>>,
    pub params: Vec<u8>,
}

pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenerationRequest>()
            .init_resource::<ActiveMaterial>()
            .init_resource::<LastGenerated>()
            .add_plugins((
                ExtractResourcePlugin::<MapTextures>::default(),
                ExtractResourcePlugin::<GenerationRequest>::default(),
                ExtractResourcePlugin::<ActiveMaterial>::default(),
                ExtractResourcePlugin::<export::ExportRequest>::default(),
                ExtractResourcePlugin::<export::ExportDispatched>::default(),
                ExtractResourcePlugin::<SharedStatus>::default(),
            ))
            .add_systems(PreStartup, load_shader_library)
            .add_systems(Startup, create_map_textures)
            .add_systems(
                Update,
                (
                    regenerate_on_shader_change,
                    resize_map_textures,
                    sync_active_material,
                    publish_request,
                    record_generation_time,
                )
                    .chain(),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<MaterialPipeline>()
            .init_resource::<DispatchHistory>()
            .init_resource::<PendingDispatch>()
            .add_systems(RenderStartup, init_maps_layout)
            .add_systems(
                Render,
                (prepare_pipeline, publish_status)
                    .chain()
                    .in_set(RenderSystems::Prepare),
            )
            .add_systems(
                Render,
                prepare_dispatch.in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(RenderGraph, run_dispatch.before(camera_driver));
    }
}

fn load_shader_library(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ShaderLibrary(
        SHADER_LIBRARY
            .iter()
            .map(|path| asset_server.load(*path))
            .collect(),
    ));
}

pub fn new_map_image(resolution: u32) -> Image {
    let mut image = Image::new_uninit(
        Extent3d {
            width: resolution,
            height: resolution,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        MAP_FORMAT,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
        | TextureUsages::STORAGE_BINDING
        | TextureUsages::COPY_SRC
        | TextureUsages::COPY_DST;
    image
}

pub fn create_map_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let resolution = Editor::default().preview_resolution;
    let handles = (0..MAP_COUNT)
        .map(|_| images.add(new_map_image(resolution)))
        .collect();
    commands.insert_resource(MapTextures {
        images: handles,
        resolution,
    });
}

fn resize_map_textures(
    editor: Res<Editor>,
    mut textures: ResMut<MapTextures>,
    mut images: ResMut<Assets<Image>>,
) {
    if textures.resolution == editor.preview_resolution {
        return;
    }
    let resolution = editor.preview_resolution;
    for handle in &textures.images {
        if let Some(mut image) = images.get_mut(handle) {
            image.texture_descriptor.size = Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: 1,
            };
        }
    }
    textures.resolution = resolution;
}

fn sync_active_material(
    editor: Res<Editor>,
    catalog: Res<MaterialCatalog>,
    mut active: ResMut<ActiveMaterial>,
) {
    let Some(entry) = catalog.get(editor.active) else {
        return;
    };
    let params = pad_params(&entry.layout.pack(&editor.values));
    if active.shader.as_ref() != Some(&entry.shader) || active.params != params {
        active.shader = Some(entry.shader.clone());
        active.params = params;
    }
}

fn regenerate_on_shader_change(
    mut events: MessageReader<AssetEvent<Shader>>,
    mut editor: ResMut<Editor>,
) {
    let changed = events
        .read()
        .any(|event| matches!(event, AssetEvent::Modified { .. }));
    if changed {
        editor.bump();
    }
}

fn record_generation_time(editor: Res<Editor>, time: Res<Time>, mut last: ResMut<LastGenerated>) {
    if last.generation != editor.generation {
        last.generation = editor.generation;
        last.at = time.elapsed_secs_f64();
    }
}

fn publish_request(
    editor: Res<Editor>,
    textures: Res<MapTextures>,
    time: Res<Time>,
    mut request: ResMut<GenerationRequest>,
) {
    let next = GenerationRequest {
        generation: editor.generation,
        resolution: textures.resolution,
        time: time.elapsed_secs(),
        seed: editor.seed,
    };
    if request.generation != next.generation
        || request.resolution != next.resolution
        || request.seed != next.seed
    {
        *request = next;
    } else if editor.animate {
        request.time = next.time;
    }
}

#[derive(Resource)]
struct MapsLayout(BindGroupLayoutDescriptor);

fn init_maps_layout(mut commands: Commands) {
    let mut entries: Vec<BindGroupLayoutEntry> = Vec::with_capacity(MAP_COUNT + 1);
    entries.push(
        uniform_buffer_sized(false, NonZeroU64::new(GLOBALS_SIZE)).build(0, ShaderStages::COMPUTE),
    );
    for index in 0..MAP_COUNT {
        entries.push(
            texture_storage_2d(MAP_FORMAT, StorageTextureAccess::WriteOnly)
                .build(index as u32 + 1, ShaderStages::COMPUTE),
        );
    }

    commands.insert_resource(MapsLayout(BindGroupLayoutDescriptor::new(
        "pbr_gen_maps",
        &entries,
    )));
}

#[derive(Resource, Default)]
struct MaterialPipeline {
    shader: Option<AssetId<Shader>>,
    params_layout: Option<BindGroupLayoutDescriptor>,
    id: Option<CachedComputePipelineId>,
}

fn prepare_pipeline(
    active: Res<ActiveMaterial>,
    maps_layout: Res<MapsLayout>,
    pipeline_cache: Res<PipelineCache>,
    mut pipeline: ResMut<MaterialPipeline>,
) {
    let Some(shader) = active.shader.as_ref() else {
        return;
    };
    if pipeline.shader == Some(shader.id()) && pipeline.params_layout.is_some() {
        return;
    }

    let params_layout = BindGroupLayoutDescriptor::new(
        "pbr_gen_params",
        &[uniform_buffer_sized(false, None).build(0, ShaderStages::COMPUTE)],
    );

    pipeline.id = Some(
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("pbr_gen".into()),
            layout: vec![maps_layout.0.clone(), params_layout.clone()],
            shader: shader.clone(),
            entry_point: Some(Cow::from(ENTRY_POINT)),
            ..default()
        }),
    );
    pipeline.shader = Some(shader.id());
    pipeline.params_layout = Some(params_layout);
}

fn publish_status(
    pipeline: Res<MaterialPipeline>,
    pipeline_cache: Res<PipelineCache>,
    status: Res<SharedStatus>,
) {
    let Some(id) = pipeline.id else {
        return;
    };
    let next = match pipeline_cache.get_compute_pipeline_state(id) {
        CachedPipelineState::Ok(_) => ShaderStatus::Ok,
        CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) => ShaderStatus::Loading,
        CachedPipelineState::Err(error) => ShaderStatus::Failed(error.to_string()),
        _ => ShaderStatus::Loading,
    };
    if status.get() != next {
        status.set(next);
    }
}

struct DispatchJob {
    maps: BindGroup,
    params: BindGroup,
    resolution: u32,
}

#[derive(Resource, Default)]
struct PendingDispatch {
    jobs: Vec<DispatchJob>,
}

#[derive(Resource, Default)]
struct DispatchHistory {
    generation: u64,
    export_token: u64,
}

fn globals_bytes(resolution: u32, time: f32, seed: f32) -> [u8; GLOBALS_SIZE as usize] {
    let mut out = [0u8; GLOBALS_SIZE as usize];
    out[0..4].copy_from_slice(&resolution.to_ne_bytes());
    out[4..8].copy_from_slice(&resolution.to_ne_bytes());
    out[8..12].copy_from_slice(&time.to_ne_bytes());
    out[12..16].copy_from_slice(&seed.to_ne_bytes());
    out
}

fn pad_params(bytes: &[u8]) -> Vec<u8> {
    let mut padded = bytes.to_vec();
    padded.resize(PARAMS_BUFFER_SIZE, 0);
    padded
}

#[allow(clippy::too_many_arguments)]
fn prepare_dispatch(
    pipeline: Res<MaterialPipeline>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    maps_layout: Res<MapsLayout>,
    textures: Res<MapTextures>,
    request: Res<GenerationRequest>,
    export: Res<export::ExportRequest>,
    dispatched: Res<export::ExportDispatched>,
    active: Res<ActiveMaterial>,
    mut history: ResMut<DispatchHistory>,
    mut pending: ResMut<PendingDispatch>,
) {
    pending.jobs.clear();

    let ready = pipeline.id.is_some_and(|id| {
        matches!(
            pipeline_cache.get_compute_pipeline_state(id),
            CachedPipelineState::Ok(_)
        )
    });
    let Some(params_layout) = pipeline.params_layout.as_ref() else {
        return;
    };
    if !ready {
        return;
    }

    let build = |handles: &[Handle<Image>], resolution: u32| -> Option<DispatchJob> {
        if handles.len() != MAP_COUNT {
            return None;
        }
        let images: Vec<&GpuImage> = handles
            .iter()
            .map(|handle| gpu_images.get(handle))
            .collect::<Option<Vec<_>>>()?;
        if images.iter().any(|image| {
            image.texture_descriptor.size.width != resolution
                || image.texture_descriptor.size.height != resolution
        }) {
            return None;
        }

        let globals = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("pbr_gen_globals"),
            contents: &globals_bytes(resolution, request.time, request.seed),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let params = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("pbr_gen_params"),
            contents: &active.params,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let mut entries: Vec<BindGroupEntry> = Vec::with_capacity(MAP_COUNT + 1);
        entries.push(BindGroupEntry {
            binding: 0,
            resource: globals.as_entire_binding(),
        });
        for (index, image) in images.iter().enumerate() {
            entries.push(BindGroupEntry {
                binding: index as u32 + 1,
                resource: BindingResource::TextureView(&image.texture_view),
            });
        }

        let maps = render_device.create_bind_group(
            Some("pbr_gen_maps"),
            &pipeline_cache.get_bind_group_layout(&maps_layout.0),
            &entries,
        );
        let params = render_device.create_bind_group(
            Some("pbr_gen_params"),
            &pipeline_cache.get_bind_group_layout(params_layout),
            &[BindGroupEntry {
                binding: 0,
                resource: params.as_entire_binding(),
            }],
        );

        Some(DispatchJob {
            maps,
            params,
            resolution,
        })
    };

    if history.generation != request.generation {
        if let Some(job) = build(&textures.images, textures.resolution) {
            history.generation = request.generation;
            pending.jobs.push(job);
        }
    }

    if export.token != 0 && history.export_token != export.token {
        if let Some(job) = build(&export.images, export.resolution) {
            history.export_token = export.token;
            pending.jobs.push(job);
            if let Ok(mut slot) = dispatched.0.lock() {
                *slot = export.token;
            }
        }
    }
}

fn run_dispatch(
    mut render_context: RenderContext,
    pipeline: Res<MaterialPipeline>,
    pipeline_cache: Res<PipelineCache>,
    pending: Res<PendingDispatch>,
) {
    if pending.jobs.is_empty() {
        return;
    }
    let Some(compute) = pipeline
        .id
        .and_then(|id| pipeline_cache.get_compute_pipeline(id))
    else {
        return;
    };

    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("pbr_gen"),
            ..default()
        });
    pass.set_pipeline(compute);

    for job in &pending.jobs {
        let groups = job.resolution.div_ceil(WORKGROUP_SIZE);
        pass.set_bind_group(0, &job.maps, &[]);
        pass.set_bind_group(1, &job.params, &[]);
        pass.dispatch_workgroups(groups, groups, 1);
    }
}
