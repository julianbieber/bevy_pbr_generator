//! The compute pipeline built for the active material, and the bind groups it
//! is dispatched with.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::{texture_storage_2d, uniform_buffer_sized};
use bevy::render::render_resource::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
    Buffer, BufferDescriptor, BufferUsages, CachedComputePipelineId, ComputePipelineDescriptor,
    PipelineCache, ShaderStages, ShaderType, StorageTextureAccess,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::shader::Shader;

use crate::gpu::maps::{MAP_COUNT, MAP_FORMAT};
use crate::gpu::{ExportRequest, GenerateRequest, GpuGlobals};
use crate::material::params::PARAMS_BUFFER_SIZE;

/// The two bind group layouts every material pipeline uses.
///
/// Group 0 is engine-owned and identical for every material, so it is built
/// once. Group 1 holds only the params uniform, declared at a fixed size rather
/// than the parsed struct's, because the pipeline is queued before any struct
/// has been parsed.
#[derive(Resource, Debug)]
pub struct ComputeLayouts {
    pub maps: BindGroupLayoutDescriptor,
    pub params: BindGroupLayoutDescriptor,
}

/// The buffers and bind groups for one map set.
#[derive(Debug)]
pub struct ComputeBinding {
    globals: Buffer,
    params: Buffer,
    pub maps: Option<BindGroup>,
    pub values: Option<BindGroup>,
}

impl ComputeBinding {
    fn new(render_device: &RenderDevice, label: &'static str) -> Self {
        Self {
            globals: render_device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size: GpuGlobals::min_size().get(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            params: render_device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size: PARAMS_BUFFER_SIZE as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            maps: None,
            values: None,
        }
    }

    /// Whether both groups are built and the pass can be recorded.
    pub fn ready(&self) -> bool {
        self.maps.is_some() && self.values.is_some()
    }
}

/// The preview map set and the export map set.
#[derive(Resource, Debug)]
pub struct ComputeBindings {
    pub preview: ComputeBinding,
    pub export: ComputeBinding,
}

/// The pipeline currently built for the active material.
#[derive(Resource, Debug, Default)]
pub struct MaterialPipeline {
    pub shader: Option<AssetId<Shader>>,
    pub id: Option<CachedComputePipelineId>,
}

pub fn init_compute_resources(mut commands: Commands, render_device: Res<RenderDevice>) {
    let mut builders = vec![uniform_buffer_sized(false, Some(GpuGlobals::min_size()))];
    builders.extend(
        (0..MAP_COUNT).map(|_| texture_storage_2d(MAP_FORMAT, StorageTextureAccess::WriteOnly)),
    );
    let map_entries: Vec<BindGroupLayoutEntry> = builders
        .iter()
        .enumerate()
        .map(|(binding, builder)| builder.build(binding as u32, ShaderStages::COMPUTE))
        .collect();

    commands.insert_resource(ComputeLayouts {
        maps: BindGroupLayoutDescriptor::new("pbr_gen_maps_layout", &map_entries),
        params: BindGroupLayoutDescriptor::new(
            "pbr_gen_params_layout",
            &[uniform_buffer_sized(false, None).build(0, ShaderStages::COMPUTE)],
        ),
    });

    commands.insert_resource(ComputeBindings {
        preview: ComputeBinding::new(&render_device, "pbr_gen_preview"),
        export: ComputeBinding::new(&render_device, "pbr_gen_export"),
    });
}

pub fn prepare_pipeline(
    request: Res<GenerateRequest>,
    layouts: Res<ComputeLayouts>,
    pipeline_cache: Res<PipelineCache>,
    mut pipeline: ResMut<MaterialPipeline>,
) {
    let Some(shader) = request.shader.clone() else {
        pipeline.shader = None;
        pipeline.id = None;
        return;
    };

    if pipeline.shader == Some(shader.id()) {
        return;
    }

    pipeline.shader = Some(shader.id());
    pipeline.id = Some(
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("pbr_gen_material".into()),
            layout: vec![layouts.maps.clone(), layouts.params.clone()],
            shader,
            entry_point: Some("generate".into()),
            zero_initialize_workgroup_memory: false,
            ..default()
        }),
    );
}

pub fn prepare_bind_groups(
    request: Res<GenerateRequest>,
    export: Res<ExportRequest>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    layouts: Res<ComputeLayouts>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut bindings: ResMut<ComputeBindings>,
) {
    let bindings = bindings.as_mut();
    build_binding(
        &mut bindings.preview,
        &request,
        &gpu_images,
        &layouts,
        &pipeline_cache,
        &render_device,
        &render_queue,
    );
    build_binding(
        &mut bindings.export,
        &export.0,
        &gpu_images,
        &layouts,
        &pipeline_cache,
        &render_device,
        &render_queue,
    );
}

fn build_binding(
    binding: &mut ComputeBinding,
    request: &GenerateRequest,
    gpu_images: &RenderAssets<GpuImage>,
    layouts: &ComputeLayouts,
    pipeline_cache: &PipelineCache,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) {
    if request.images.len() != MAP_COUNT || request.resolution == 0 {
        binding.maps = None;
        binding.values = None;
        return;
    }

    let views: Option<Vec<_>> = request
        .images
        .iter()
        .map(|handle| gpu_images.get(handle).map(|image| &image.texture_view))
        .collect();
    let Some(views) = views else {
        binding.maps = None;
        binding.values = None;
        return;
    };

    let globals = GpuGlobals {
        resolution: UVec2::splat(request.resolution),
        time: request.time,
        seed: request.seed,
    };
    let mut bytes = bevy::render::render_resource::encase::UniformBuffer::new(Vec::new());
    bytes.write(&globals).expect("globals are fixed size");
    render_queue.write_buffer(&binding.globals, 0, bytes.as_ref());

    let mut params = request.params.clone();
    params.resize(PARAMS_BUFFER_SIZE, 0);
    render_queue.write_buffer(&binding.params, 0, &params);

    let mut entries: Vec<BindGroupEntry> = Vec::with_capacity(MAP_COUNT + 1);
    entries.push(BindGroupEntry {
        binding: 0,
        resource: binding.globals.as_entire_binding(),
    });
    for (index, view) in views.iter().enumerate() {
        entries.push(BindGroupEntry {
            binding: index as u32 + 1,
            resource: BindingResource::TextureView(view),
        });
    }

    binding.maps = Some(render_device.create_bind_group(
        "pbr_gen_maps_bind_group",
        &pipeline_cache.get_bind_group_layout(&layouts.maps),
        &entries,
    ));
    binding.values = Some(render_device.create_bind_group(
        "pbr_gen_params_bind_group",
        &pipeline_cache.get_bind_group_layout(&layouts.params),
        &[BindGroupEntry {
            binding: 0,
            resource: binding.params.as_entire_binding(),
        }],
    ));
}
