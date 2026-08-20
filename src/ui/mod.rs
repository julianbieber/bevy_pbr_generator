pub mod inspector;

use bevy::{
    feathers::{
        controls::{
            ButtonVariant, ColorChannel, FeathersButton, FeathersCheckbox, FeathersColorSlider,
            FeathersSlider, SliderBaseColor,
        },
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, ThemeTextColor, ThemedText, UiTheme},
        tokens, FeathersPlugins,
    },
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    scene::prelude::{bsn, CommandsSceneExt},
    text::FontSize,
    ui::{Checked, InteractionDisabled},
    ui_widgets::{Activate, SliderPrecision, SliderStep, SliderValue, ValueChange},
};

use crate::editor::{
    ChannelView, Editor, PreviewMesh, ShaderStatus, SharedStatus, ViewMode, RESOLUTIONS,
};
use crate::gpu::export::{ExportProgress, StartExport};
use crate::gpu::maps::PbrMap;
use crate::gpu::{LastGenerated, MapTextures};
use crate::material::params::{linear_to_srgb, srgb_to_linear, Widget};
use crate::material::{carry_over_values, MaterialCatalog, MaterialsChanged};
use crate::preview::{BAR_BOTTOM, BAR_TOP, PANEL_LEFT, PANEL_RIGHT};
use inspector::{make_map_material, FullMapView, InspectorPlugin, MapView, MapViewMaterial};

#[derive(Component)]
struct ParamsPanel;

#[derive(Component)]
struct MaterialsList;

#[derive(Component)]
struct LeftPanel;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct ExportStatusText;

#[derive(Component, Clone, Copy)]
struct SelectMaterial(usize);

#[derive(Component, Clone, Copy)]
struct SelectMesh(PreviewMesh);

#[derive(Component, Clone, Copy)]
struct SelectChannel(ChannelView);

#[derive(Component, Clone, Copy)]
struct SelectMapView(PbrMap);

#[derive(Component, Clone, Copy)]
struct ParamSlider {
    index: usize,
    component: usize,
}

#[derive(Component, Clone, Copy)]
struct ParamToggle {
    index: usize,
}

#[derive(Component, Clone, Copy)]
struct SunSlider {
    elevation: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum ToolbarAction {
    CyclePreview,
    CycleExport,
    Export,
    Randomise,
    Reset,
    ToggleAnimate,
    ToggleTurntable,
    ShowMesh,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum DynamicLabel {
    PreviewResolution,
    ExportResolution,
}

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FeathersPlugins, InspectorPlugin))
            .insert_resource(UiTheme(create_dark_theme()))
            .add_systems(Startup, setup_ui.after(crate::gpu::create_map_textures))
            .add_systems(
                Update,
                (
                    rebuild_materials_list,
                    rebuild_params_panel,
                    update_dynamic_labels,
                    update_selection_marks,
                    update_status_text,
                    scroll_left_panel,
                ),
            )
            .add_observer(on_button)
            .add_observer(on_thumbnail_click)
            .add_observer(on_slider)
            .add_observer(on_checkbox);
    }
}

fn panel_node(width: f32) -> Node {
    Node {
        width: px(width),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(8)),
        row_gap: px(6),
        overflow: Overflow::scroll_y(),
        min_height: px(0),
        flex_shrink: 1.0,
        ..default()
    }
}

fn heading(text: &str) -> impl Bundle {
    (
        Text::new(text.to_string()),
        ThemedText,
        ThemeTextColor(tokens::TEXT_DIM),
        TextFont {
            font_size: FontSize::Px(11.0),
            ..default()
        },
        Node {
            margin: UiRect::top(px(6)),
            ..default()
        },
    )
}

fn row() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: px(6),
        ..default()
    }
}

fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<MapViewMaterial>>,
    textures: Res<MapTextures>,
) {
    let ui_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 0,
                ..default()
            },
        ))
        .id();

    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            UiTargetCamera(ui_camera),
            Pickable::IGNORE,
        ))
        .id();

    let toolbar = commands
        .spawn((
            Node {
                width: percent(100),
                height: px(BAR_TOP),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(6),
                padding: UiRect::horizontal(px(8)),
                ..default()
            },
            ThemeBackgroundColor(tokens::WINDOW_BG),
            ChildOf(root),
        ))
        .id();

    build_toolbar(&mut commands, toolbar);

    let middle = commands
        .spawn((
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_basis: px(0),
                min_height: px(0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(root),
        ))
        .id();

    let left = commands
        .spawn((
            panel_node(PANEL_LEFT),
            ThemeBackgroundColor(tokens::WINDOW_BG),
            LeftPanel,
            ChildOf(middle),
        ))
        .id();

    commands.spawn((heading("MATERIALS"), ChildOf(left)));
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            ..default()
        },
        MaterialsList,
        ChildOf(left),
    ));
    commands.spawn((heading("PARAMETERS"), ChildOf(left)));
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        ParamsPanel,
        ChildOf(left),
    ));

    let centre = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(middle),
        ))
        .id();

    commands.spawn((
        Node {
            max_width: percent(100),
            max_height: percent(100),
            width: percent(100),
            aspect_ratio: Some(1.0),
            display: Display::None,
            ..default()
        },
        MaterialNode(make_map_material(
            &mut materials,
            &textures,
            PbrMap::BaseColor,
        )),
        FullMapView,
        ChildOf(centre),
    ));

    let right = commands
        .spawn((
            panel_node(PANEL_RIGHT),
            ThemeBackgroundColor(tokens::WINDOW_BG),
            ChildOf(middle),
        ))
        .id();

    build_right_panel(&mut commands, right, &mut materials, &textures);

    let status = commands
        .spawn((
            Node {
                width: percent(100),
                height: px(BAR_BOTTOM),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12),
                padding: UiRect::horizontal(px(8)),
                ..default()
            },
            ThemeBackgroundColor(tokens::WINDOW_BG),
            ChildOf(root),
        ))
        .id();

    commands.spawn((
        Text::new(""),
        ThemedText,
        TextFont {
            font_size: FontSize::Px(11.0),
            ..default()
        },
        StatusText,
        ChildOf(status),
    ));
    commands.spawn((
        Text::new(""),
        ThemedText,
        ThemeTextColor(tokens::TEXT_DIM),
        TextFont {
            font_size: FontSize::Px(11.0),
            ..default()
        },
        ExportStatusText,
        ChildOf(status),
    ));
}

fn spawn_button(commands: &mut Commands, parent: Entity, caption: String, marker: impl Bundle) {
    commands
        .spawn_scene(bsn! {
            @FeathersButton {
                @caption: bsn! { Text({caption}) ThemedText },
            }
        })
        .insert((ChildOf(parent), marker));
}

fn build_toolbar(commands: &mut Commands, toolbar: Entity) {
    commands.spawn((
        Text::new("PBR Generator"),
        ThemedText,
        TextFont {
            font_size: FontSize::Px(13.0),
            ..default()
        },
        Node {
            margin: UiRect::right(px(10)),
            ..default()
        },
        ChildOf(toolbar),
    ));

    for mesh in PreviewMesh::ALL {
        spawn_button(
            commands,
            toolbar,
            mesh.label().to_string(),
            SelectMesh(mesh),
        );
    }

    spawn_button(commands, toolbar, "3D".to_string(), ToolbarAction::ShowMesh);

    commands.spawn((
        Node {
            flex_grow: 1.0,
            ..default()
        },
        ChildOf(toolbar),
    ));

    commands
        .spawn_scene(bsn! {
            @FeathersCheckbox {
                @caption: bsn! { Text("Animate") ThemedText },
            }
        })
        .insert((ChildOf(toolbar), ToolbarAction::ToggleAnimate));

    commands
        .spawn_scene(bsn! {
            @FeathersCheckbox {
                @caption: bsn! { Text("Turntable") ThemedText },
            }
        })
        .insert((ChildOf(toolbar), ToolbarAction::ToggleTurntable));

    spawn_button(
        commands,
        toolbar,
        "Preview 512".to_string(),
        (ToolbarAction::CyclePreview, DynamicLabel::PreviewResolution),
    );
    spawn_button(
        commands,
        toolbar,
        "Export 1024".to_string(),
        (ToolbarAction::CycleExport, DynamicLabel::ExportResolution),
    );

    commands
        .spawn_scene(bsn! {
            @FeathersButton {
                @caption: bsn! { Text("Export PNGs") ThemedText },
                @variant: ButtonVariant::Primary,
            }
        })
        .insert((ChildOf(toolbar), ToolbarAction::Export));
}

fn build_right_panel(
    commands: &mut Commands,
    panel: Entity,
    materials: &mut Assets<MapViewMaterial>,
    textures: &MapTextures,
) {
    commands.spawn((heading("MAPS"), ChildOf(panel)));

    let grid = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(4),
                row_gap: px(4),
                ..default()
            },
            ChildOf(panel),
        ))
        .id();

    for map in PbrMap::ALL {
        let cell = commands
            .spawn((
                Node {
                    width: px(100),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                ChildOf(grid),
            ))
            .id();

        commands.spawn((
            Node {
                width: px(100),
                height: px(100),
                ..default()
            },
            MaterialNode(make_map_material(materials, textures, map)),
            MapView,
            SelectMapView(map),
            ChildOf(cell),
        ));

        commands.spawn((
            Text::new(map.label().to_string()),
            ThemedText,
            ThemeTextColor(tokens::TEXT_DIM),
            TextFont {
                font_size: FontSize::Px(9.0),
                ..default()
            },
            ChildOf(cell),
        ));
    }

    commands.spawn((heading("CHANNEL"), ChildOf(panel)));
    let channels = commands.spawn((row(), ChildOf(panel))).id();
    for channel in ChannelView::ALL {
        spawn_button(
            commands,
            channels,
            channel.label().to_string(),
            SelectChannel(channel),
        );
    }

    commands.spawn((heading("SUN"), ChildOf(panel)));
    let defaults = Editor::default();
    for (label, elevation, value, max) in [
        ("Elevation", true, defaults.sun_elevation, 1.5_f32),
        (
            "Azimuth",
            false,
            defaults.sun_azimuth,
            std::f32::consts::TAU,
        ),
    ] {
        let line = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                ChildOf(panel),
            ))
            .id();
        commands.spawn((
            Text::new(label.to_string()),
            ThemedText,
            ThemeTextColor(tokens::TEXT_DIM),
            TextFont {
                font_size: FontSize::Px(10.0),
                ..default()
            },
            ChildOf(line),
        ));
        commands
            .spawn_scene(bsn! {
                @FeathersSlider {
                    @value: {value},
                    @min: 0.0,
                    @max: {max},
                }
                SliderPrecision(2)
            })
            .insert((ChildOf(line), SunSlider { elevation }));
    }

    commands.spawn((heading("SEED"), ChildOf(panel)));
    let seed_row = commands.spawn((row(), ChildOf(panel))).id();
    spawn_button(
        commands,
        seed_row,
        "Randomise".to_string(),
        ToolbarAction::Randomise,
    );
    spawn_button(
        commands,
        seed_row,
        "Reset params".to_string(),
        ToolbarAction::Reset,
    );
}

fn rebuild_materials_list(
    mut commands: Commands,
    catalog: Res<MaterialCatalog>,
    list: Query<(Entity, Option<&Children>), With<MaterialsList>>,
    mut built: Local<Vec<String>>,
) {
    if catalog.entries.is_empty() {
        return;
    }
    let names: Vec<String> = catalog
        .entries
        .iter()
        .map(|entry| entry.name.clone())
        .collect();
    if *built == names {
        return;
    }
    let Ok((list, children)) = list.single() else {
        return;
    };

    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }
    for (index, name) in names.iter().enumerate() {
        spawn_button(&mut commands, list, name.clone(), SelectMaterial(index));
    }
    *built = names;
}

#[allow(clippy::too_many_arguments)]
fn rebuild_params_panel(
    mut commands: Commands,
    catalog: Res<MaterialCatalog>,
    mut editor: ResMut<Editor>,
    panel: Query<(Entity, Option<&Children>), With<ParamsPanel>>,
    mut changed: MessageReader<MaterialsChanged>,
    mut built_for: Local<Option<usize>>,
    mut built_names: Local<Vec<String>>,
) {
    let Some(entry) = catalog.get(editor.active) else {
        return;
    };
    let material_reloaded = changed
        .read()
        .any(|message| message.reparsed.contains(&editor.active));

    if *built_for == Some(editor.active) && !material_reloaded {
        return;
    }
    let Ok((panel, children)) = panel.single() else {
        return;
    };

    let previous: Vec<(String, [f32; 4])> = if *built_for == Some(editor.active) {
        built_names
            .iter()
            .cloned()
            .zip(editor.values.iter().copied())
            .collect()
    } else {
        Vec::new()
    };

    editor.values = carry_over_values(&entry.layout, &previous);
    editor.bump();
    *built_for = Some(editor.active);
    *built_names = entry
        .layout
        .params
        .iter()
        .map(|spec| spec.name.clone())
        .collect();

    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    for (group, members) in entry.layout.groups() {
        if !group.is_empty() {
            commands.spawn((heading(&group), ChildOf(panel)));
        }

        for index in members {
            let spec = &entry.layout.params[index];
            let value = editor.values[index];

            match spec.widget {
                Widget::Hidden => {}
                Widget::Toggle => {
                    let mut entity = commands.spawn_scene(bsn! {
                        @FeathersCheckbox {
                            @caption: bsn! { Text({spec.label.clone()}) ThemedText },
                        }
                    });
                    entity.insert((ChildOf(panel), ParamToggle { index }));
                    if value[0] > 0.5 {
                        entity.insert(Checked);
                    }
                }
                Widget::Color => {
                    let line = commands
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: px(2),
                                ..default()
                            },
                            ChildOf(panel),
                        ))
                        .id();
                    commands.spawn((
                        Text::new(spec.label.clone()),
                        ThemedText,
                        ThemeTextColor(tokens::TEXT_DIM),
                        TextFont {
                            font_size: FontSize::Px(10.0),
                            ..default()
                        },
                        ChildOf(line),
                    ));

                    let base = Color::linear_rgb(value[0], value[1], value[2]);
                    for (component, channel) in [
                        (0usize, ColorChannel::Red),
                        (1, ColorChannel::Green),
                        (2, ColorChannel::Blue),
                    ] {
                        let shown = linear_to_srgb(value[component]);
                        commands
                            .spawn_scene(bsn! {
                                @FeathersColorSlider {
                                    @value: {shown},
                                    @channel: {channel},
                                }
                            })
                            .insert((
                                ChildOf(line),
                                SliderBaseColor(base),
                                ParamSlider { index, component },
                            ));
                    }
                }
                Widget::Slider { min, max, step } => {
                    let components = spec.kind.components();
                    let line = commands
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: px(2),
                                ..default()
                            },
                            ChildOf(panel),
                        ))
                        .id();
                    commands.spawn((
                        Text::new(spec.label.clone()),
                        ThemedText,
                        ThemeTextColor(tokens::TEXT_DIM),
                        TextFont {
                            font_size: FontSize::Px(10.0),
                            ..default()
                        },
                        ChildOf(line),
                    ));

                    for (component, start) in value.iter().copied().take(components).enumerate() {
                        let mut entity = commands.spawn_scene(bsn! {
                            @FeathersSlider {
                                @value: {start},
                                @min: {min},
                                @max: {max},
                            }
                            SliderPrecision(2)
                        });
                        entity.insert((ChildOf(line), ParamSlider { index, component }));
                        if let Some(step) = step {
                            entity.insert(SliderStep(step));
                        }
                    }
                }
            }
        }
    }

    if let Some(error) = &entry.error {
        commands.spawn((
            Text::new(error.clone()),
            ThemedText,
            ThemeTextColor(tokens::TEXT_MAIN),
            TextFont {
                font_size: FontSize::Px(10.0),
                ..default()
            },
            ChildOf(panel),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn on_button(
    activate: On<Activate>,
    mut commands: Commands,
    mut editor: ResMut<Editor>,
    catalog: Res<MaterialCatalog>,
    materials: Query<&SelectMaterial>,
    meshes: Query<&SelectMesh>,
    channels: Query<&SelectChannel>,
    actions: Query<&ToolbarAction>,
) {
    let target = activate.entity;

    if let Ok(SelectMaterial(index)) = materials.get(target) {
        if editor.active != *index && *index < catalog.entries.len() {
            editor.active = *index;
            editor.bump();
        }
        return;
    }

    if let Ok(SelectMesh(mesh)) = meshes.get(target) {
        editor.mesh = *mesh;
        editor.view = ViewMode::Mesh;
        return;
    }

    if let Ok(SelectChannel(channel)) = channels.get(target) {
        editor.channel = *channel;
        return;
    }

    let Ok(action) = actions.get(target) else {
        return;
    };

    match action {
        ToolbarAction::CyclePreview => {
            editor.preview_resolution = next_resolution(editor.preview_resolution);
            editor.bump();
        }
        ToolbarAction::CycleExport => {
            editor.export_resolution = next_resolution(editor.export_resolution);
        }
        ToolbarAction::Export => {
            commands.write_message(StartExport);
        }
        ToolbarAction::Randomise => {
            editor.seed = (editor.seed * 7.13 + 3.7).fract() + editor.generation as f32 * 0.017;
            editor.bump();
        }
        ToolbarAction::Reset => {
            if let Some(entry) = catalog.get(editor.active) {
                editor.values = entry.layout.defaults();
                editor.bump();
            }
        }
        ToolbarAction::ShowMesh => {
            editor.view = ViewMode::Mesh;
        }
        ToolbarAction::ToggleAnimate | ToolbarAction::ToggleTurntable => {}
    }
}

fn on_thumbnail_click(
    click: On<Pointer<Click>>,
    mut editor: ResMut<Editor>,
    thumbnails: Query<&SelectMapView>,
) {
    let Ok(SelectMapView(map)) = thumbnails.get(click.entity) else {
        return;
    };
    editor.view = match editor.view {
        ViewMode::Map(current) if current == *map => ViewMode::Mesh,
        _ => ViewMode::Map(*map),
    };
}

fn on_checkbox(
    change: On<ValueChange<bool>>,
    mut commands: Commands,
    mut editor: ResMut<Editor>,
    toggles: Query<&ParamToggle>,
    actions: Query<&ToolbarAction>,
) {
    let target = change.source;
    let value = change.value;

    let mut entity = commands.entity(target);
    if value {
        entity.insert(Checked);
    } else {
        entity.remove::<Checked>();
    }

    if let Ok(ParamToggle { index }) = toggles.get(target) {
        if let Some(slot) = editor.values.get_mut(*index) {
            slot[0] = if value { 1.0 } else { 0.0 };
            editor.bump();
        }
        return;
    }

    match actions.get(target) {
        Ok(ToolbarAction::ToggleAnimate) => editor.animate = value,
        Ok(ToolbarAction::ToggleTurntable) => editor.turntable = value,
        _ => {}
    }
}

fn on_slider(
    change: On<ValueChange<f32>>,
    mut commands: Commands,
    mut editor: ResMut<Editor>,
    params: Query<(&ParamSlider, Option<&SliderBaseColor>)>,
    sun: Query<&SunSlider>,
) {
    let target = change.source;
    commands.entity(target).insert(SliderValue(change.value));

    if let Ok((slider, base)) = params.get(target) {
        let stored = if base.is_some() {
            srgb_to_linear(change.value)
        } else {
            change.value
        };
        if let Some(slot) = editor.values.get_mut(slider.index) {
            slot[slider.component] = stored;
            editor.bump();
        }
        return;
    }

    if let Ok(SunSlider { elevation }) = sun.get(target) {
        if *elevation {
            editor.sun_elevation = change.value;
        } else {
            editor.sun_azimuth = change.value;
        }
    }
}

fn next_resolution(current: u32) -> u32 {
    let at = RESOLUTIONS.iter().position(|r| *r == current).unwrap_or(1);
    RESOLUTIONS[(at + 1) % RESOLUTIONS.len()]
}

fn update_dynamic_labels(
    editor: Res<Editor>,
    labels: Query<(&DynamicLabel, &Children)>,
    mut texts: Query<&mut Text>,
    children: Query<&Children>,
) {
    if !editor.is_changed() {
        return;
    }
    for (kind, roots) in &labels {
        let text = match kind {
            DynamicLabel::PreviewResolution => format!("Preview {}", editor.preview_resolution),
            DynamicLabel::ExportResolution => format!("Export {}", editor.export_resolution),
        };
        for root in roots.iter() {
            if let Some(entity) = find_text(root, &children, &texts) {
                if let Ok(mut slot) = texts.get_mut(entity) {
                    if slot.0 != text {
                        slot.0 = text.clone();
                    }
                }
            }
        }
    }
}

fn find_text(
    entity: Entity,
    children: &Query<&Children>,
    texts: &Query<&mut Text>,
) -> Option<Entity> {
    if texts.get(entity).is_ok() {
        return Some(entity);
    }
    for child in children.get(entity).ok()?.iter() {
        if let Some(found) = find_text(child, children, texts) {
            return Some(found);
        }
    }
    None
}

fn update_selection_marks(
    mut commands: Commands,
    editor: Res<Editor>,
    materials: Query<(Entity, &SelectMaterial)>,
    meshes: Query<(Entity, &SelectMesh)>,
    channels: Query<(Entity, &SelectChannel)>,
) {
    if !editor.is_changed() {
        return;
    }

    let mut mark = |entity: Entity, selected: bool| {
        if selected {
            commands.entity(entity).insert(InteractionDisabled);
        } else {
            commands.entity(entity).remove::<InteractionDisabled>();
        }
    };

    for (entity, SelectMaterial(index)) in &materials {
        mark(entity, *index == editor.active);
    }
    for (entity, SelectMesh(mesh)) in &meshes {
        mark(
            entity,
            *mesh == editor.mesh && editor.view == ViewMode::Mesh,
        );
    }
    for (entity, SelectChannel(channel)) in &channels {
        mark(entity, *channel == editor.channel);
    }
}

#[allow(clippy::too_many_arguments)]
fn update_status_text(
    editor: Res<Editor>,
    catalog: Res<MaterialCatalog>,
    status: Res<SharedStatus>,
    textures: Res<MapTextures>,
    progress: Res<ExportProgress>,
    last: Res<LastGenerated>,
    time: Res<Time>,
    mut lines: Query<&mut Text, (With<StatusText>, Without<ExportStatusText>)>,
    mut export_lines: Query<&mut Text, (With<ExportStatusText>, Without<StatusText>)>,
) {
    let file = catalog
        .get(editor.active)
        .map(|entry| entry.file.clone())
        .unwrap_or_else(|| "no material".to_string());

    let state = match status.get() {
        ShaderStatus::Loading => "compiling".to_string(),
        ShaderStatus::Ok => "compiled".to_string(),
        ShaderStatus::Failed(error) => format!("FAILED - {}", first_line(&error)),
    };

    let view = match editor.view {
        ViewMode::Mesh => editor.mesh.label().to_string(),
        ViewMode::Map(map) => format!("{} ({})", map.label(), map.channels()),
    };

    let age = (time.elapsed_secs_f64() - last.at).max(0.0);
    let line = format!(
        "{file} - {state} - {}x{} - {} maps - {view} - regenerated {age:.1}s ago",
        textures.resolution,
        textures.resolution,
        crate::gpu::maps::MAP_COUNT
    );

    for mut text in &mut lines {
        if text.0 != line {
            text.0 = line.clone();
        }
    }
    for mut text in &mut export_lines {
        if text.0 != progress.message {
            text.0 = progress.message.clone();
        }
    }
}

fn first_line(text: &str) -> String {
    text.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(text)
        .trim()
        .to_string()
}

fn scroll_left_panel(
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut panels: Query<(&mut ScrollPosition, &ComputedNode), With<LeftPanel>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        wheel.clear();
        return;
    };
    if cursor.x > PANEL_LEFT {
        wheel.clear();
        return;
    }

    let mut delta = 0.0;
    for event in wheel.read() {
        delta += match event.unit {
            MouseScrollUnit::Line => event.y * 21.0,
            MouseScrollUnit::Pixel => event.y,
        };
    }
    if delta == 0.0 {
        return;
    }

    for (mut position, computed) in &mut panels {
        let max = (computed.content_size().y - computed.size().y).max(0.0)
            * computed.inverse_scale_factor();
        position.y = (position.y - delta).clamp(0.0, max);
    }
}
