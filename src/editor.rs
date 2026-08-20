use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

use crate::gpu::maps::PbrMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PreviewMesh {
    #[default]
    Cube,
    Sphere,
    Plane,
    Quad,
}

impl PreviewMesh {
    pub const ALL: [PreviewMesh; 4] = [Self::Cube, Self::Sphere, Self::Plane, Self::Quad];

    pub fn label(self) -> &'static str {
        match self {
            Self::Cube => "Cube",
            Self::Sphere => "Sphere",
            Self::Plane => "Plane",
            Self::Quad => "Quad",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ViewMode {
    #[default]
    Mesh,
    Map(PbrMap),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ChannelView {
    #[default]
    Rgb,
    R,
    G,
    B,
    A,
}

impl ChannelView {
    pub const ALL: [ChannelView; 5] = [Self::Rgb, Self::R, Self::G, Self::B, Self::A];

    pub fn label(self) -> &'static str {
        match self {
            Self::Rgb => "RGB",
            Self::R => "R",
            Self::G => "G",
            Self::B => "B",
            Self::A => "A",
        }
    }

    pub fn mask(self) -> Vec4 {
        match self {
            Self::Rgb => Vec4::new(1.0, 1.0, 1.0, 1.0),
            Self::R => Vec4::new(1.0, 0.0, 0.0, 0.0),
            Self::G => Vec4::new(0.0, 1.0, 0.0, 0.0),
            Self::B => Vec4::new(0.0, 0.0, 1.0, 0.0),
            Self::A => Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn is_single(self) -> bool {
        self != Self::Rgb
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ShaderStatus {
    #[default]
    Loading,
    Ok,
    Failed(String),
}

#[derive(Resource, Clone, Default, ExtractResource)]
pub struct SharedStatus(pub Arc<Mutex<ShaderStatus>>);

impl SharedStatus {
    pub fn get(&self) -> ShaderStatus {
        self.0.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn set(&self, status: ShaderStatus) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = status;
        }
    }
}

pub const RESOLUTIONS: [u32; 5] = [256, 512, 1024, 2048, 4096];

#[derive(Resource)]
pub struct Editor {
    pub active: usize,
    pub values: Vec<[f32; 4]>,
    pub preview_resolution: u32,
    pub export_resolution: u32,
    pub output_dir: String,
    pub seed: f32,
    pub animate: bool,
    pub turntable: bool,
    pub mesh: PreviewMesh,
    pub view: ViewMode,
    pub channel: ChannelView,
    pub sun_elevation: f32,
    pub sun_azimuth: f32,
    pub generation: u64,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            active: 0,
            values: Vec::new(),
            preview_resolution: 512,
            export_resolution: 1024,
            output_dir: "output".to_string(),
            seed: 0.0,
            animate: false,
            turntable: false,
            mesh: PreviewMesh::default(),
            view: ViewMode::default(),
            channel: ChannelView::default(),
            sun_elevation: 0.6,
            sun_azimuth: 0.9,
            generation: 1,
        }
    }
}

impl Editor {
    pub fn bump(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn sun_direction(&self) -> Vec3 {
        let (elevation, azimuth) = (self.sun_elevation, self.sun_azimuth);
        Vec3::new(
            elevation.cos() * azimuth.sin(),
            elevation.sin(),
            elevation.cos() * azimuth.cos(),
        )
    }
}
