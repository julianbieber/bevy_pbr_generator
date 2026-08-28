//! Reading a generated map set back off the GPU and writing it out as PNGs.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::tasks::AsyncComputeTaskPool;

use crate::gpu::maps::{allocate_maps, MapKind, MAP_COUNT};
use crate::gpu::{ExportRequest, GenerateRequest};

/// How far along an export is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportStage {
    #[default]
    Idle,
    /// The UI has asked for an export; nothing has been allocated yet.
    Requested,
    /// The export map set is dispatching. Readback waits for the dispatch to
    /// have been submitted, which is why this stage spans more than one frame.
    Dispatching(u32),
    Reading,
}

/// The export currently in flight.
#[derive(Resource, Debug, Default)]
pub struct ExportJob {
    pub stage: ExportStage,
    pub images: Vec<Handle<Image>>,
    pub resolution: u32,
    pub output_dir: String,
    pub material: String,
    pub remaining: usize,
}

impl ExportJob {
    /// Asks for an export of the active material. Ignored while one is already
    /// running, since the export map set is single-use.
    pub fn request(&mut self) {
        if self.stage == ExportStage::Idle {
            self.stage = ExportStage::Requested;
        }
    }

    pub fn running(&self) -> bool {
        self.stage != ExportStage::Idle
    }
}

/// Identifies which map a readback entity is carrying.
#[derive(Component, Debug)]
pub struct ExportMap(pub MapKind);

/// Frames to wait between requesting the export dispatch and reading the maps
/// back, so the readback copies a generated texture rather than a blank one.
const DISPATCH_FRAMES: u32 = 2;

pub fn drive_export(
    mut commands: Commands,
    mut job: ResMut<ExportJob>,
    mut images: ResMut<Assets<Image>>,
    mut export: ResMut<ExportRequest>,
    request: Res<GenerateRequest>,
    settings: Res<crate::app::EditorSettings>,
    catalog: Res<crate::material::catalog::MaterialCatalog>,
) {
    match job.stage {
        ExportStage::Idle => {}
        ExportStage::Requested => {
            let Some(active) = catalog.active() else {
                job.stage = ExportStage::Idle;
                return;
            };
            let resolution = settings.export_resolution;
            let handles = allocate_maps(&mut images, resolution);

            export.0 = GenerateRequest {
                generation: request.generation.wrapping_add(1),
                shader: Some(active.shader.clone()),
                images: handles.clone(),
                params: active.packed(),
                resolution,
                time: request.time,
                seed: request.seed,
            };

            job.images = handles;
            job.resolution = resolution;
            job.output_dir = settings.output_dir.clone();
            job.material = active.name.clone();
            job.remaining = MAP_COUNT;
            job.stage = ExportStage::Dispatching(DISPATCH_FRAMES);
        }
        ExportStage::Dispatching(remaining) => {
            if remaining > 0 {
                job.stage = ExportStage::Dispatching(remaining - 1);
                return;
            }
            for (index, kind) in MapKind::ALL.iter().enumerate() {
                let Some(handle) = job.images.get(index).cloned() else {
                    continue;
                };
                commands
                    .spawn((Readback::texture(handle), ExportMap(*kind)))
                    .observe(receive_map);
            }
            job.stage = ExportStage::Reading;
        }
        ExportStage::Reading => {}
    }
}

fn receive_map(
    event: On<ReadbackComplete>,
    mut commands: Commands,
    maps: Query<&ExportMap>,
    mut job: ResMut<ExportJob>,
) {
    let entity = event.event().entity;
    let Ok(map) = maps.get(entity) else {
        return;
    };
    let kind = map.0;

    let resolution = job.resolution;
    let directory = PathBuf::from(&job.output_dir).join(slug(&job.material));
    let path = directory.join(format!("{}.png", kind.file_stem()));
    let data = event.event().data.clone();

    AsyncComputeTaskPool::get()
        .spawn(async move {
            if let Err(error) = write_map(&path, resolution, kind, &data) {
                error!("export of {} failed: {error}", path.display());
            } else {
                info!("wrote {}", path.display());
            }
        })
        .detach();

    commands.entity(entity).despawn();

    job.remaining = job.remaining.saturating_sub(1);
    if job.remaining == 0 {
        job.stage = ExportStage::Idle;
        job.images.clear();
    }
}

/// Turns a material's display name into a directory name.
pub fn slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Converts one `rgba16float` readback to 8-bit RGBA and writes it as a PNG.
///
/// The sRGB transfer function is applied here and only here, to the RGB of the
/// three colour maps; alpha and every data map stay linear. That is what makes
/// the exported PNG agree with the preview, which samples the same linear
/// texture.
fn write_map(
    path: &std::path::Path,
    resolution: u32,
    kind: MapKind,
    data: &[u8],
) -> Result<(), String> {
    let pixels = (resolution as usize) * (resolution as usize);
    if data.len() < pixels * 8 {
        return Err(format!(
            "readback is {} bytes, short of the {} a {resolution}x{resolution} map needs",
            data.len(),
            pixels * 8
        ));
    }

    let encode_srgb = kind.rgb_is_srgb();
    let mut out = Vec::with_capacity(pixels * 4);
    for pixel in 0..pixels {
        for channel in 0..4 {
            let at = pixel * 8 + channel * 2;
            let value = half::f16::from_le_bytes([data[at], data[at + 1]]).to_f32();
            let value = if encode_srgb && channel < 3 {
                crate::material::params::linear_to_srgb(value)
            } else {
                value
            };
            out.push((value.clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    image::RgbaImage::from_raw(resolution, resolution, out)
        .ok_or_else(|| "readback did not fill the image".to_string())?
        .save(path)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The material name becomes a directory, so anything a user might type in a
    // `@material` header has to survive the trip to the filesystem.
    #[test]
    fn slugs_are_filesystem_safe() {
        assert_eq!(slug("Water"), "water");
        assert_eq!(slug("Rocky Ground"), "rocky_ground");
        assert_eq!(slug("../etc"), "___etc");
    }
}
