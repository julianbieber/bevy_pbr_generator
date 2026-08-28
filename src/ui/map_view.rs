//! The UI material that draws one generated map, masked to a single channel.

use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

/// Which channels of a map are shown.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Channel {
    #[default]
    Rgb,
    R,
    G,
    B,
    A,
}

impl Channel {
    pub const ALL: [Channel; 5] = [Self::Rgb, Self::R, Self::G, Self::B, Self::A];

    pub fn label(self) -> &'static str {
        match self {
            Self::Rgb => "RGB",
            Self::R => "R",
            Self::G => "G",
            Self::B => "B",
            Self::A => "A",
        }
    }

    fn index(self) -> f32 {
        match self {
            Self::Rgb => 0.0,
            Self::R => 1.0,
            Self::G => 2.0,
            Self::B => 3.0,
            Self::A => 4.0,
        }
    }
}

/// Draws a map into a UI node.
///
/// The map is sampled and written out linear, exactly as the 3D preview samples
/// it, so a thumbnail and the mesh agree about what the map contains.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MapViewMaterial {
    /// `x` selects the channel; the rest is padding to a 16-byte uniform.
    #[uniform(0)]
    pub settings: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub map: Handle<Image>,
}

impl MapViewMaterial {
    pub fn new(map: Handle<Image>, channel: Channel) -> Self {
        Self {
            settings: Vec4::new(channel.index(), 0.0, 0.0, 0.0),
            map,
        }
    }

    pub fn set_channel(&mut self, channel: Channel) {
        self.settings.x = channel.index();
    }
}

impl UiMaterial for MapViewMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_view.wgsl".into()
    }
}

/// Pushes the selected channel into every map material.
pub fn follow_channel(channel: Res<Channel>, mut materials: ResMut<Assets<MapViewMaterial>>) {
    if !channel.is_changed() {
        return;
    }
    let ids: Vec<_> = materials.ids().collect();
    for id in ids {
        if let Some(mut material) = materials.get_mut(id) {
            material.set_channel(*channel);
        }
    }
}
