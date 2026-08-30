//! The editor around the generator: what the preview shows, how the camera and
//! sun are driven, and what causes a fresh set of maps to be generated.

pub mod camera;
pub mod preview;

use bevy::prelude::*;

use crate::gpu::maps::{allocate_maps, MAP_COUNT};
use crate::gpu::{GenerateRequest, Generation, PreviewMaps};
use crate::material::catalog::{scan_materials, MaterialCatalog};

/// Root of the asset directory on disk, as scanned for materials.
pub const ASSET_ROOT: &str = "assets";

/// Resolutions offered in the preview and export dropdowns.
pub const RESOLUTIONS: [u32; 5] = [256, 512, 1024, 2048, 4096];

/// The mesh the generated material is previewed on.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewShape {
    #[default]
    Cube,
    Sphere,
    Plane,
    Quad,
}

impl PreviewShape {
    pub const ALL: [PreviewShape; 4] = [Self::Cube, Self::Sphere, Self::Plane, Self::Quad];

    pub fn label(self) -> &'static str {
        match self {
            Self::Cube => "cube",
            Self::Sphere => "sphere",
            Self::Plane => "plane",
            Self::Quad => "quad",
        }
    }

    /// The mesh this shape is previewed on, carrying mikktspace tangents.
    ///
    /// Bevy reads the normal, clearcoat normal, anisotropy and depth maps only
    /// on a mesh with tangents, so a shape that failed to generate them is
    /// previewed with those four maps silently inert; the failure is warned
    /// about rather than returned, because there is still a mesh to draw.
    pub fn mesh(self) -> Mesh {
        let mut mesh: Mesh = match self {
            Self::Cube => Cuboid::new(1.0, 1.0, 1.0).into(),
            Self::Sphere => Sphere::new(0.7).mesh().uv(48, 24),
            Self::Plane => Plane3d::default().mesh().size(1.6, 1.6).into(),
            Self::Quad => Rectangle::new(1.4, 1.4).into(),
        };
        if let Err(error) = mesh.generate_tangents() {
            warn!("no tangents on the {} preview mesh: {error}", self.label());
        }
        mesh
    }
}

/// Where the sun sits, in degrees. Drives the directional light and the
/// atmosphere together so the sky and the image-based light agree.
#[derive(Resource, Debug, Clone, Copy)]
pub struct SunAngles {
    pub elevation: f32,
    pub azimuth: f32,
}

impl Default for SunAngles {
    fn default() -> Self {
        Self {
            elevation: 35.0,
            azimuth: 30.0,
        }
    }
}

impl SunAngles {
    /// The rotation a `DirectionalLight` needs to shine from these angles.
    pub fn rotation(self) -> Quat {
        Quat::from_euler(
            EulerRot::YXZ,
            self.azimuth.to_radians(),
            -self.elevation.to_radians(),
            0.0,
        )
    }
}

/// Everything the user sets that is not a material parameter.
#[derive(Resource, Debug, Clone)]
pub struct EditorSettings {
    pub preview_resolution: u32,
    pub export_resolution: u32,
    pub output_dir: String,
    pub animate: bool,
    pub turntable: bool,
    pub seed: f32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            preview_resolution: 512,
            export_resolution: 2048,
            output_dir: "./output".to_string(),
            animate: false,
            turntable: false,
            seed: 0.0,
        }
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreviewShape>()
            .init_resource::<SunAngles>()
            .init_resource::<EditorSettings>()
            .init_resource::<MaterialCatalog>()
            .add_systems(
                Startup,
                (load_catalog, preview::spawn_preview, camera::spawn_cameras),
            )
            .add_systems(
                Update,
                (
                    camera::orbit_camera,
                    camera::turntable,
                    preview::follow_shape,
                    preview::follow_sun,
                    preview::bind_maps_to_material,
                    reload_changed_materials,
                    advance_animation,
                    build_request,
                ),
            );
    }
}

fn load_catalog(
    mut commands: Commands,
    mut catalog: ResMut<MaterialCatalog>,
    mut images: ResMut<Assets<Image>>,
    settings: Res<EditorSettings>,
    asset_server: Res<AssetServer>,
) {
    catalog.materials = scan_materials(std::path::Path::new(ASSET_ROOT), &asset_server);
    if catalog.materials.is_empty() {
        warn!("no materials found under {ASSET_ROOT}/materials");
    }

    let resolution = settings.preview_resolution;
    commands.insert_resource(PreviewMaps {
        images: allocate_maps(&mut images, resolution),
        resolution,
    });
}

/// Re-parses a material whose shader asset changed, and bumps the generation so
/// the edit is visible without any further input.
fn reload_changed_materials(
    mut events: MessageReader<AssetEvent<bevy::shader::Shader>>,
    mut catalog: ResMut<MaterialCatalog>,
    mut generation: ResMut<Generation>,
) {
    let mut touched = false;
    for event in events.read() {
        let AssetEvent::Modified { id } = event else {
            continue;
        };
        for spec in &mut catalog.materials {
            if spec.shader.id() == *id {
                crate::material::catalog::reload(spec);
            }
        }
        touched = true;
    }
    if touched {
        generation.bump();
    }
}

fn advance_animation(settings: Res<EditorSettings>, mut generation: ResMut<Generation>) {
    if settings.animate {
        generation.bump();
    }
}

/// Assembles what the render world needs. Deliberately does not bump the
/// generation: only a change that alters the output does that.
fn build_request(
    catalog: Res<MaterialCatalog>,
    maps: Res<PreviewMaps>,
    settings: Res<EditorSettings>,
    generation: Res<Generation>,
    time: Res<Time>,
    mut request: ResMut<GenerateRequest>,
) {
    let Some(active) = catalog.active() else {
        request.shader = None;
        return;
    };

    request.generation = generation.0;
    request.shader = Some(active.shader.clone());
    request.params = active.packed();
    request.images = maps.images.clone();
    request.resolution = maps.resolution;
    request.time = time.elapsed_secs();
    request.seed = settings.seed;

    debug_assert_eq!(request.images.len(), MAP_COUNT);
}
