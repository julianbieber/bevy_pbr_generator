//! The editor shell: the panels, the controls that drive the generator, and the
//! viewport rect the 3D camera renders into.

pub mod map_view;
pub mod params_panel;

#[allow(deprecated)]
use bevy::feathers::controls::{button_bundle, ButtonBundleProps, ButtonVariant};
use bevy::feathers::rounded_corners::RoundedCorners;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_render::ui_material::MaterialNode;
use bevy::ui_widgets::{
    slider_self_update, Activate, Slider, SliderOrientation, SliderRange, SliderValue, TrackClick,
    ValueChange,
};

use crate::app::{EditorSettings, PreviewShape, SunAngles, RESOLUTIONS};
use crate::gpu::export::ExportJob;
use crate::gpu::maps::{MapKind, MAP_COUNT};
use crate::gpu::{Generation, PreviewMaps};
use crate::material::catalog::MaterialCatalog;
use crate::status::{CompileState, SharedStatus};
use map_view::{Channel, MapViewMaterial};

const PANEL: Color = Color::srgb(0.11, 0.11, 0.13);
const TEXT_DIM: Color = Color::srgb(0.62, 0.62, 0.66);
const TRACK: Color = Color::srgb(0.18, 0.18, 0.21);
const ACCENT: Color = Color::srgb(0.35, 0.55, 0.85);

/// The node the 3D camera's viewport is kept aligned with.
#[derive(Component, Debug)]
pub struct ViewportNode;

/// The single line at the bottom of the window.
#[derive(Component, Debug)]
pub struct StatusText;

/// Selects a material by its index in the catalog.
#[derive(Component, Debug)]
pub struct MaterialButton(pub usize);

#[derive(Component, Debug)]
pub struct ShapeButton(pub PreviewShape);

#[derive(Component, Debug)]
pub struct ChannelButton(pub Channel);

#[derive(Component, Debug)]
pub struct PreviewResButton(pub u32);

#[derive(Component, Debug)]
pub struct ExportResButton(pub u32);

#[derive(Component, Debug)]
pub struct ExportButton;

#[derive(Component, Debug)]
pub struct AnimateButton;

#[derive(Component, Debug)]
pub struct TurntableButton;

#[derive(Component, Debug)]
pub struct MapThumbnail(pub MapKind);

/// Which axis of the sun a slider drives.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SunSlider {
    Elevation,
    Azimuth,
}

/// The map shown full-bleed over the viewport, if any.
#[derive(Resource, Debug, Default)]
pub struct SelectedMap(pub Option<MapKind>);

/// The node holding the full-bleed map view.
#[derive(Component, Debug)]
pub struct FullMapView;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Channel>()
            .init_resource::<SelectedMap>()
            .init_resource::<params_panel::PanelState>()
            .add_plugins(bevy::ui_render::prelude::UiMaterialPlugin::<MapViewMaterial>::default())
            .add_systems(Startup, spawn_ui.after(crate::app::camera::spawn_cameras))
            .add_systems(
                Update,
                (
                    map_view::follow_channel,
                    update_slider_fill,
                    params_panel::rebuild,
                    update_status_text,
                    update_full_map_view,
                ),
            );
    }
}

fn label(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(12.0),
        TextColor(TEXT_DIM),
    )
}

fn heading(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(11.0),
        TextColor(Color::srgb(0.45, 0.45, 0.5)),
        Node {
            margin: UiRect::vertical(Val::Px(6.0)),
            ..default()
        },
    )
}

#[allow(deprecated)]
pub fn small_button(text: &str, marker: impl Bundle) -> impl Bundle {
    (
        Node {
            margin: UiRect::all(Val::Px(2.0)),
            flex_shrink: 0.0,
            ..default()
        },
        children![(
            button_bundle(
                ButtonBundleProps {
                    variant: ButtonVariant::Normal,
                    corners: RoundedCorners::All,
                },
                (),
                bevy::ecs::spawn::Spawn(label(text.to_string())),
            ),
            marker,
        )],
    )
}

/// The bar inside a slider whose width tracks the value.
#[derive(Component, Debug)]
pub struct SliderFill;

/// A labelled horizontal slider carrying `marker`.
///
/// Built from the headless `Slider` widget rather than a Feathers control: the
/// themed slider is a scene component, which cannot be spawned as part of a
/// bundle.
pub fn labelled_slider(
    text: &str,
    value: f32,
    min: f32,
    max: f32,
    marker: impl Bundle,
) -> impl Bundle {
    let fraction = if max > min {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    (
        Node {
            flex_direction: FlexDirection::Column,
            margin: UiRect::vertical(Val::Px(3.0)),
            ..default()
        },
        children![
            label(text.to_string()),
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(12.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(TRACK),
                Slider {
                    track_click: TrackClick::Drag,
                    orientation: SliderOrientation::Horizontal,
                },
                SliderValue(value),
                SliderRange::new(min, max),
                Hovered::default(),
                marker,
                children![(
                    Node {
                        width: Val::Percent(fraction * 100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(ACCENT),
                    SliderFill,
                )],
            ),
        ],
    )
}

/// Keeps each slider's fill bar in step with its value.
fn update_slider_fill(
    sliders: Query<(&SliderValue, &SliderRange, &Children), Changed<SliderValue>>,
    mut fills: Query<&mut Node, With<SliderFill>>,
) {
    for (value, range, children) in &sliders {
        let span = range.end() - range.start();
        let fraction = if span > 0.0 {
            ((value.0 - range.start()) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        for child in children.iter() {
            if let Ok(mut node) = fills.get_mut(child) {
                node.width = Val::Percent(fraction * 100.0);
            }
        }
    }
}

fn spawn_ui(
    mut commands: Commands,
    catalog: Res<MaterialCatalog>,
    sun: Res<SunAngles>,
    maps: Res<PreviewMaps>,
    channel: Res<Channel>,
    mut materials: ResMut<Assets<MapViewMaterial>>,
) {
    let root = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .id();

    let top_bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .id();
    commands.entity(root).add_child(top_bar);

    commands.entity(top_bar).with_children(|bar| {
        bar.spawn(label("mesh"));
        for option in PreviewShape::ALL {
            bar.spawn(small_button(option.label(), ShapeButton(option)));
        }
        bar.spawn((
            label("preview"),
            Node {
                margin: UiRect::left(Val::Px(12.0)),
                ..default()
            },
        ));
        for resolution in RESOLUTIONS {
            bar.spawn(small_button(
                &resolution.to_string(),
                PreviewResButton(resolution),
            ));
        }
        bar.spawn((
            label("export"),
            Node {
                margin: UiRect::left(Val::Px(12.0)),
                ..default()
            },
        ));
        for resolution in RESOLUTIONS {
            bar.spawn(small_button(
                &resolution.to_string(),
                ExportResButton(resolution),
            ));
        }
        bar.spawn(small_button("Export PNGs", ExportButton));
        bar.spawn(small_button("animate", AnimateButton));
        bar.spawn(small_button("turntable", TurntableButton));
    });

    let main = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .id();
    commands.entity(root).add_child(main);

    // Left panel: the material list and the data-driven parameter panel.
    let left = commands
        .spawn((
            Node {
                width: Val::Px(230.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .id();
    commands.entity(main).add_child(left);

    commands.entity(left).with_children(|panel| {
        panel.spawn(heading("MATERIALS"));
        for (index, spec) in catalog.materials.iter().enumerate() {
            panel.spawn(small_button(&spec.name, MaterialButton(index)));
        }
        panel.spawn(heading("PARAMETERS"));
        panel.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            params_panel::ParamsPanel,
        ));
    });

    // Centre: the rect the 3D camera renders into, with the full-bleed map view
    // stacked over it.
    let viewport = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                ..default()
            },
            ViewportNode,
        ))
        .id();
    commands.entity(main).add_child(viewport);

    let first_map = maps.images.first().cloned().unwrap_or_default();
    commands.entity(viewport).with_children(|centre| {
        centre.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::None,
                ..default()
            },
            MaterialNode(materials.add(MapViewMaterial::new(first_map, *channel))),
            FullMapView,
        ));
    });

    // Right panel: the map thumbnails, the channel selector and the sun.
    let right = commands
        .spawn((
            Node {
                width: Val::Px(210.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .id();
    commands.entity(main).add_child(right);

    commands.entity(right).with_children(|panel| {
        panel.spawn(heading("MAPS"));
        let grid = panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .id();
        panel.commands().entity(grid).with_children(|grid| {
            for (index, kind) in MapKind::ALL.iter().enumerate() {
                let handle = maps.images.get(index).cloned().unwrap_or_default();
                grid.spawn((
                    Node {
                        width: Val::Px(88.0),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    children![
                        (
                            Node {
                                width: Val::Px(88.0),
                                height: Val::Px(88.0),
                                ..default()
                            },
                            MaterialNode(materials.add(MapViewMaterial::new(handle, *channel))),
                            MapThumbnail(*kind),
                            Button,
                        ),
                        label(kind.label().to_string()),
                    ],
                ));
            }
        });

        panel.spawn(heading("CHANNEL"));
        let row = panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .id();
        panel.commands().entity(row).with_children(|row| {
            for option in Channel::ALL {
                row.spawn(small_button(option.label(), ChannelButton(option)));
            }
        });

        panel.spawn(heading("SUN"));
        panel.spawn(labelled_slider(
            "elevation",
            sun.elevation,
            -10.0,
            90.0,
            SunSlider::Elevation,
        ));
        panel.spawn(labelled_slider(
            "azimuth",
            sun.azimuth,
            0.0,
            360.0,
            SunSlider::Azimuth,
        ));
    });

    let status = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(PANEL),
            children![(label("starting up"), StatusText)],
        ))
        .id();
    commands.entity(root).add_child(status);

    commands.add_observer(slider_self_update);
    commands.add_observer(on_select);
    commands.add_observer(on_setting);
    commands.add_observer(on_value_change);
}

fn update_full_map_view(
    selected: Res<SelectedMap>,
    maps: Res<PreviewMaps>,
    mut materials: ResMut<Assets<MapViewMaterial>>,
    mut views: Query<(&mut Node, &MaterialNode<MapViewMaterial>), With<FullMapView>>,
) {
    if !selected.is_changed() {
        return;
    }
    for (mut node, material) in &mut views {
        match selected.0 {
            Some(kind) => {
                node.display = Display::Flex;
                let index = MapKind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
                if let (Some(handle), Some(mut material)) =
                    (maps.images.get(index), materials.get_mut(&material.0))
                {
                    material.map = handle.clone();
                }
            }
            None => node.display = Display::None,
        }
    }
}

fn update_status_text(
    status: Res<SharedStatus>,
    catalog: Res<MaterialCatalog>,
    maps: Res<PreviewMaps>,
    job: Res<ExportJob>,
    mut text: Query<(&mut Text, &mut TextColor), With<StatusText>>,
) {
    let Ok((mut text, mut colour)) = text.single_mut() else {
        return;
    };

    let (compile, micros) = status
        .with(|s| (s.compile.clone(), s.last_dispatch_micros))
        .unwrap_or((CompileState::Loading, 0));

    let name = catalog
        .active()
        .map(|s| s.asset_path.clone())
        .unwrap_or_else(|| "no material".to_string());

    let parse_error = catalog.active().and_then(|s| s.error.clone());

    let (message, tint) = match (parse_error, &compile) {
        (Some(error), _) => (error, Color::srgb(0.95, 0.45, 0.4)),
        (None, CompileState::Failed(error)) => (
            error.lines().next().unwrap_or("shader error").to_string(),
            Color::srgb(0.95, 0.45, 0.4),
        ),
        (None, CompileState::Loading) => ("compiling".to_string(), TEXT_DIM),
        (None, CompileState::Ok) => (
            format!(
                "compiled OK · {}x{} · {MAP_COUNT} maps · {:.1} ms",
                maps.resolution,
                maps.resolution,
                micros as f32 / 1000.0
            ),
            TEXT_DIM,
        ),
    };

    let suffix = if job.running() { " · exporting" } else { "" };
    text.0 = format!("{name} · {message}{suffix}");
    colour.0 = tint;
}

fn on_select(
    event: On<Activate>,
    materials: Query<&MaterialButton>,
    shapes: Query<&ShapeButton>,
    channels: Query<&ChannelButton>,
    thumbnails: Query<&MapThumbnail>,
    toggles: Query<&params_panel::ParamToggle>,
    mut catalog: ResMut<MaterialCatalog>,
    mut shape: ResMut<PreviewShape>,
    mut channel: ResMut<Channel>,
    mut selected: ResMut<SelectedMap>,
    mut generation: ResMut<Generation>,
) {
    let entity = event.event().entity;

    if let Ok(button) = materials.get(entity) {
        if button.0 < catalog.materials.len() {
            catalog.active = button.0;
            generation.bump();
        }
    }
    if let Ok(button) = shapes.get(entity) {
        *shape = button.0;
    }
    if let Ok(button) = channels.get(entity) {
        *channel = button.0;
    }
    if let Ok(toggle) = toggles.get(entity) {
        if let Some(active) = catalog.active_mut() {
            if let Some(stored) = active.values.get_mut(toggle.field) {
                stored[0] = if stored[0] >= 0.5 { 0.0 } else { 1.0 };
                generation.bump();
            }
        }
    }
    if let Ok(thumbnail) = thumbnails.get(entity) {
        selected.0 = if selected.0 == Some(thumbnail.0) {
            None
        } else {
            Some(thumbnail.0)
        };
    }
}

fn on_setting(
    event: On<Activate>,
    mut commands: Commands,
    preview_res: Query<&PreviewResButton>,
    export_res: Query<&ExportResButton>,
    exports: Query<&ExportButton>,
    animates: Query<&AnimateButton>,
    turntables: Query<&TurntableButton>,
    mut settings: ResMut<EditorSettings>,
    mut job: ResMut<ExportJob>,
    mut generation: ResMut<Generation>,
    mut images: ResMut<Assets<Image>>,
) {
    let entity = event.event().entity;

    if let Ok(button) = preview_res.get(entity) {
        settings.preview_resolution = button.0;
        commands.insert_resource(PreviewMaps {
            images: crate::gpu::maps::allocate_maps(&mut images, button.0),
            resolution: button.0,
        });
        generation.bump();
    }
    if let Ok(button) = export_res.get(entity) {
        settings.export_resolution = button.0;
    }
    if exports.get(entity).is_ok() {
        job.request();
    }
    if animates.get(entity).is_ok() {
        settings.animate = !settings.animate;
    }
    if turntables.get(entity).is_ok() {
        settings.turntable = !settings.turntable;
    }
}

fn on_value_change(
    event: On<ValueChange<f32>>,
    params: Query<&params_panel::ParamSlider>,
    suns: Query<&SunSlider>,
    mut catalog: ResMut<MaterialCatalog>,
    mut sun: ResMut<SunAngles>,
    mut generation: ResMut<Generation>,
) {
    let source = event.event().source;
    let value = event.event().value;

    if let Ok(slider) = params.get(source) {
        if let Some(active) = catalog.active_mut() {
            if let Some(stored) = active.values.get_mut(slider.field) {
                stored[slider.component] = slider.encode(value);
                generation.bump();
            }
        }
    }

    if let Ok(axis) = suns.get(source) {
        match axis {
            SunSlider::Elevation => sun.elevation = value,
            SunSlider::Azimuth => sun.azimuth = value,
        }
    }
}
