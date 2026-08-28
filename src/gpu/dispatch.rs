//! The generation-gated compute dispatch, and the pipeline status it reports.

use std::time::Instant;

use bevy::prelude::*;
use bevy::render::render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::shader::ShaderCacheError;

use crate::gpu::pipeline::{ComputeBinding, ComputeBindings, MaterialPipeline};
use crate::gpu::{ExportRequest, GenerateRequest, WORKGROUP_SIZE};
use crate::status::{CompileState, SharedStatus};

/// The generations whose maps are currently on the GPU.
#[derive(Resource, Debug, Default)]
pub struct LastDispatched {
    pub preview: Option<u64>,
    pub export: Option<u64>,
}

/// Dispatches the active material once per generation, for the preview and for
/// an in-flight export, and never while its pipeline is failing to compile.
///
/// A failed compile leaves the previously generated maps bound and untouched, so
/// a broken edit costs the preview nothing but its freshness.
pub fn dispatch_maps(
    request: Res<GenerateRequest>,
    export: Res<ExportRequest>,
    pipeline: Res<MaterialPipeline>,
    pipeline_cache: Res<PipelineCache>,
    bindings: Res<ComputeBindings>,
    status: Res<SharedStatus>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut last: ResMut<LastDispatched>,
) {
    let Some(id) = pipeline.id else {
        status.with(|s| s.compile = CompileState::Loading);
        return;
    };

    let state = match pipeline_cache.get_compute_pipeline_state(id) {
        CachedPipelineState::Ok(_) => CompileState::Ok,
        CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_))
        | CachedPipelineState::Err(ShaderCacheError::ShaderImportNotYetAvailable) => {
            CompileState::Loading
        }
        CachedPipelineState::Err(error) => CompileState::Failed(error.to_string()),
        _ => CompileState::Loading,
    };
    let compiled = state == CompileState::Ok;
    status.with(|s| s.compile = state);

    if !compiled {
        return;
    }
    let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(id) else {
        return;
    };

    let started = Instant::now();
    let mut recorded = false;
    let mut encoder = render_device.create_command_encoder(&default());

    if last.preview != Some(request.generation) && bindings.preview.ready() {
        record(
            &mut encoder,
            compute_pipeline,
            &bindings.preview,
            request.resolution,
        );
        last.preview = Some(request.generation);
        recorded = true;
    }

    if export.0.shader.is_some()
        && last.export != Some(export.0.generation)
        && bindings.export.ready()
    {
        record(
            &mut encoder,
            compute_pipeline,
            &bindings.export,
            export.0.resolution,
        );
        last.export = Some(export.0.generation);
        recorded = true;
    }

    if !recorded {
        return;
    }

    render_queue.submit([encoder.finish()]);
    status.with(|s| s.last_dispatch_micros = started.elapsed().as_micros() as u64);
}

fn record(
    encoder: &mut bevy::render::render_resource::CommandEncoder,
    pipeline: &bevy::render::render_resource::ComputePipeline,
    binding: &ComputeBinding,
    resolution: u32,
) {
    let (Some(maps), Some(values)) = (binding.maps.as_ref(), binding.values.as_ref()) else {
        return;
    };
    let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
        label: Some("pbr_gen_generate"),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, maps, &[]);
    pass.set_bind_group(1, values, &[]);
    let groups_per_side = resolution.div_ceil(WORKGROUP_SIZE);
    pass.dispatch_workgroups(groups_per_side, groups_per_side, 1);
}
