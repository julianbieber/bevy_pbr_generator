use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bevy::{
    image::Image,
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        gpu_readback::{Readback, ReadbackComplete},
    },
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use half::f16;

use super::maps::{PbrMap, MAP_COUNT};
use super::new_map_image;
use crate::editor::Editor;
use crate::material::params::linear_to_srgb;
use crate::material::MaterialCatalog;

#[derive(Resource, Clone, ExtractResource, Default)]
pub struct ExportRequest {
    pub token: u64,
    pub images: Vec<Handle<Image>>,
    pub resolution: u32,
}

#[derive(Resource, Clone, ExtractResource, Default)]
pub struct ExportDispatched(pub Arc<Mutex<u64>>);

#[derive(Message)]
pub struct StartExport;

#[derive(Resource, Default)]
pub struct ExportProgress {
    pub busy: bool,
    pub message: String,
}

type Collected = Arc<Mutex<Vec<Option<Vec<u8>>>>>;

#[derive(Resource, Default)]
enum ExportState {
    #[default]
    Idle,
    AwaitingDispatch {
        token: u64,
    },
    Collecting {
        data: Collected,
    },
    Encoding {
        task: Task<Result<PathBuf, String>>,
    },
}

pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExportRequest>()
            .init_resource::<ExportProgress>()
            .init_resource::<ExportState>()
            .init_resource::<ExportDispatched>()
            .add_message::<StartExport>()
            .add_systems(
                Update,
                (
                    begin_export,
                    await_dispatch,
                    collect_readbacks,
                    finish_encoding,
                )
                    .chain(),
            );
    }
}

fn begin_export(
    mut requests: MessageReader<StartExport>,
    mut images: ResMut<Assets<Image>>,
    mut request: ResMut<ExportRequest>,
    mut progress: ResMut<ExportProgress>,
    mut state: ResMut<ExportState>,
    editor: Res<Editor>,
) {
    let wanted = requests.read().count();
    if wanted == 0 || !matches!(*state, ExportState::Idle) {
        return;
    }

    let resolution = editor.export_resolution;
    let handles: Vec<Handle<Image>> = (0..MAP_COUNT)
        .map(|_| images.add(new_map_image(resolution)))
        .collect();

    let token = request.token.wrapping_add(1).max(1);
    *request = ExportRequest {
        token,
        images: handles,
        resolution,
    };

    progress.busy = true;
    progress.message = format!("rendering {resolution}x{resolution}");
    *state = ExportState::AwaitingDispatch { token };
}

fn await_dispatch(
    mut commands: Commands,
    mut state: ResMut<ExportState>,
    mut progress: ResMut<ExportProgress>,
    request: Res<ExportRequest>,
    dispatched: Res<ExportDispatched>,
) {
    let ExportState::AwaitingDispatch { token } = *state else {
        return;
    };
    let seen = dispatched.0.lock().map(|value| *value).unwrap_or(0);
    if seen != token {
        return;
    }

    let data: Collected = Arc::new(Mutex::new(vec![None; MAP_COUNT]));
    for (index, handle) in request.images.iter().enumerate() {
        let sink = data.clone();
        commands.spawn(Readback::texture(handle.clone())).observe(
            move |event: On<ReadbackComplete>, mut commands: Commands| {
                if let Ok(mut slots) = sink.lock() {
                    if slots[index].is_none() {
                        slots[index] = Some(event.data.clone());
                    }
                }
                commands.entity(event.entity).despawn();
            },
        );
    }

    progress.message = "reading back".to_string();
    *state = ExportState::Collecting { data };
}

fn collect_readbacks(
    mut state: ResMut<ExportState>,
    mut progress: ResMut<ExportProgress>,
    mut request: ResMut<ExportRequest>,
    editor: Res<Editor>,
    catalog: Res<MaterialCatalog>,
) {
    let ExportState::Collecting { data } = &*state else {
        return;
    };

    let Ok(slots) = data.lock() else {
        return;
    };
    let ready = slots.iter().filter(|slot| slot.is_some()).count();
    progress.message = format!("reading back {ready}/{MAP_COUNT}");
    if ready < MAP_COUNT {
        return;
    }

    let planes: Vec<Vec<u8>> = slots.iter().map(|slot| slot.clone().unwrap()).collect();
    drop(slots);

    let resolution = request.resolution;
    let name = catalog
        .get(editor.active)
        .map(|entry| entry.file.trim_end_matches(".wgsl").to_string())
        .unwrap_or_else(|| "material".to_string());
    let directory = PathBuf::from(&editor.output_dir).join(name);

    let task =
        AsyncComputeTaskPool::get().spawn(async move { write_pngs(directory, resolution, planes) });

    *request = ExportRequest::default();
    progress.message = "encoding PNGs".to_string();
    *state = ExportState::Encoding { task };
}

fn finish_encoding(mut state: ResMut<ExportState>, mut progress: ResMut<ExportProgress>) {
    let ExportState::Encoding { task } = &mut *state else {
        return;
    };
    let Some(result) = block_on(future::poll_once(task)) else {
        return;
    };

    progress.busy = false;
    progress.message = match result {
        Ok(path) => format!("wrote {MAP_COUNT} maps to {}", path.display()),
        Err(error) => format!("export failed: {error}"),
    };
    *state = ExportState::Idle;
}

fn write_pngs(
    directory: PathBuf,
    resolution: u32,
    planes: Vec<Vec<u8>>,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    for (map, plane) in PbrMap::ALL.iter().zip(planes) {
        let pixels = decode_rgba16f(&plane, resolution, map.encodes_srgb_rgb())?;
        let image = image::RgbaImage::from_raw(resolution, resolution, pixels)
            .ok_or_else(|| format!("{}: pixel buffer size mismatch", map.file_stem()))?;
        let path = directory.join(format!("{}.png", map.file_stem()));
        image.save(&path).map_err(|error| error.to_string())?;
    }

    Ok(directory)
}

fn decode_rgba16f(raw: &[u8], resolution: u32, srgb_rgb: bool) -> Result<Vec<u8>, String> {
    let width = resolution as usize;
    let bytes_per_pixel = 8usize;
    let unpadded_row = width * bytes_per_pixel;
    let rows = resolution as usize;

    if raw.len() < unpadded_row * rows {
        return Err(format!(
            "readback returned {} bytes, expected at least {}",
            raw.len(),
            unpadded_row * rows
        ));
    }

    let padded_row = if rows > 1 {
        raw.len() / rows
    } else {
        raw.len()
    };
    let mut out = Vec::with_capacity(width * rows * 4);

    for row in 0..rows {
        let start = row * padded_row;
        let line = &raw[start..start + unpadded_row];
        for pixel in line.chunks_exact(bytes_per_pixel) {
            for channel in 0..4 {
                let bits = u16::from_ne_bytes([pixel[channel * 2], pixel[channel * 2 + 1]]);
                let mut value = f16::from_bits(bits).to_f32();
                if srgb_rgb && channel < 3 {
                    value = linear_to_srgb(value.clamp(0.0, 1.0));
                }
                out.push((value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            }
        }
    }

    Ok(out)
}
