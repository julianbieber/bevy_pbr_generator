use bevy::{
    prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

use crate::editor::{Editor, ViewMode};
use crate::gpu::maps::PbrMap;
use crate::gpu::MapTextures;

const MAP_VIEW_SHADER: &str = "shaders/map_view.wgsl";

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct MapViewMaterial {
    #[uniform(0)]
    pub channel_mask: Vec4,
    #[uniform(1)]
    pub options: Vec4,
    #[texture(2)]
    #[sampler(3)]
    pub map: Handle<Image>,
}

impl UiMaterial for MapViewMaterial {
    fn fragment_shader() -> ShaderRef {
        MAP_VIEW_SHADER.into()
    }
}

#[derive(Component, Clone, Copy)]
pub struct MapView;

#[derive(Component)]
pub struct FullMapView;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<MapViewMaterial>::default())
            .add_systems(Update, (sync_map_views, sync_full_view));
    }
}

pub fn make_map_material(
    materials: &mut Assets<MapViewMaterial>,
    textures: &MapTextures,
    map: PbrMap,
) -> Handle<MapViewMaterial> {
    materials.add(MapViewMaterial {
        channel_mask: Vec4::ONE,
        options: Vec4::ZERO,
        map: textures.images[map.index()].clone(),
    })
}

fn sync_map_views(
    editor: Res<Editor>,
    views: Query<&MaterialNode<MapViewMaterial>, With<MapView>>,
    mut materials: ResMut<Assets<MapViewMaterial>>,
) {
    if !editor.is_changed() {
        return;
    }
    let mask = editor.channel.mask();
    let options = Vec4::new(
        if editor.channel.is_single() { 1.0 } else { 0.0 },
        0.0,
        0.0,
        0.0,
    );

    for node in &views {
        if let Some(mut material) = materials.get_mut(&node.0) {
            material.channel_mask = mask;
            material.options = options;
        }
    }
}

fn sync_full_view(
    editor: Res<Editor>,
    textures: Res<MapTextures>,
    mut full: Query<(&mut Node, &MaterialNode<MapViewMaterial>), With<FullMapView>>,
    mut materials: ResMut<Assets<MapViewMaterial>>,
) {
    if !editor.is_changed() {
        return;
    }
    for (mut node, handle) in &mut full {
        match editor.view {
            ViewMode::Mesh => node.display = Display::None,
            ViewMode::Map(map) => {
                node.display = Display::Flex;
                if let Some(mut material) = materials.get_mut(&handle.0) {
                    material.map = textures.images[map.index()].clone();
                    material.channel_mask = editor.channel.mask();
                    material.options = Vec4::new(
                        if editor.channel.is_single() { 1.0 } else { 0.0 },
                        0.0,
                        0.0,
                        0.0,
                    );
                }
            }
        }
    }
}
