use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{
    camera::{Exposure, Hdr, Viewport},
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    light::{
        atmosphere::ScatteringMedium, light_consts::lux, Atmosphere, AtmosphereEnvironmentMapLight,
    },
    pbr::AtmosphereSettings,
    prelude::*,
};

use crate::editor::{Editor, PreviewMesh, ViewMode};
use crate::gpu::maps::PbrMap;
use crate::gpu::MapTextures;

pub const PANEL_LEFT: f32 = 280.0;
pub const PANEL_RIGHT: f32 = 240.0;
pub const BAR_TOP: f32 = 44.0;
pub const BAR_BOTTOM: f32 = 28.0;

#[derive(Component)]
pub struct PreviewCamera;

#[derive(Component)]
pub struct PreviewSubject;

#[derive(Component)]
pub struct Sun;

#[derive(Component)]
pub struct Orbit {
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub focus: Vec3,
}

impl Default for Orbit {
    fn default() -> Self {
        Self {
            distance: 4.6,
            yaw: 0.6,
            pitch: 0.35,
            focus: Vec3::ZERO,
        }
    }
}

#[derive(Resource)]
pub struct PreviewAssets {
    pub meshes: [Handle<Mesh>; 4],
}

pub struct PreviewPlugin;

impl Plugin for PreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(crate::gpu::create_map_textures))
            .add_systems(
                Update,
                (
                    orbit_camera,
                    turntable,
                    follow_sun_controls,
                    swap_preview_mesh,
                    fit_viewport,
                    toggle_subject_visibility,
                ),
            );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mediums: ResMut<Assets<ScatteringMedium>>,
    textures: Res<MapTextures>,
    editor: Res<Editor>,
) {
    let map = |which: PbrMap| textures.images[which.index()].clone();

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(map(PbrMap::BaseColor)),
        normal_map_texture: Some(map(PbrMap::Normal)),
        occlusion_texture: Some(map(PbrMap::Orm)),
        metallic_roughness_texture: Some(map(PbrMap::Orm)),
        emissive: LinearRgba::WHITE,
        emissive_texture: Some(map(PbrMap::Emissive)),
        specular_tint_texture: Some(map(PbrMap::Specular)),
        specular_texture: Some(map(PbrMap::Specular)),
        diffuse_transmission: 1.0,
        diffuse_transmission_texture: Some(map(PbrMap::Transmission)),
        specular_transmission: 1.0,
        specular_transmission_texture: Some(map(PbrMap::Transmission)),
        thickness: 1.0,
        thickness_texture: Some(map(PbrMap::Transmission)),
        clearcoat: 1.0,
        clearcoat_texture: Some(map(PbrMap::Clearcoat)),
        clearcoat_perceptual_roughness: 1.0,
        clearcoat_roughness_texture: Some(map(PbrMap::Clearcoat)),
        clearcoat_normal_texture: Some(map(PbrMap::ClearcoatNormal)),
        anisotropy_strength: 1.0,
        anisotropy_texture: Some(map(PbrMap::Anisotropy)),
        depth_map: Some(map(PbrMap::Depth)),
        parallax_depth_scale: 0.05,
        flip_normal_map_y: false,
        perceptual_roughness: 1.0,
        metallic: 1.0,
        ..default()
    });

    let mesh_handles = [
        meshes.add(Cuboid::new(1.6, 1.6, 1.6).mesh().build()),
        meshes.add(Sphere::new(1.0).mesh().uv(48, 24)),
        meshes.add(Plane3d::default().mesh().size(2.4, 2.4)),
        meshes.add(Rectangle::new(2.0, 2.0).mesh().build()),
    ];

    commands.spawn((
        PreviewSubject,
        Mesh3d(mesh_handles[0].clone()),
        MeshMaterial3d(material),
        Transform::IDENTITY,
    ));

    commands.spawn((
        Sun,
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default().looking_at(-editor.sun_direction(), Vec3::Y),
    ));

    commands.spawn(Atmosphere::earth(
        mediums.add(ScatteringMedium::earth(256, 256)),
    ));

    commands.spawn((
        PreviewCamera,
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        Hdr,
        Tonemapping::AcesFitted,
        Exposure { ev100: 13.0 },
        AtmosphereSettings::default(),
        AtmosphereEnvironmentMapLight::default(),
        Orbit::default(),
        Transform::default(),
    ));

    commands.insert_resource(PreviewAssets {
        meshes: mesh_handles,
    });
}

fn orbit_camera(
    mut camera: Single<(&mut Orbit, &mut Transform), With<PreviewCamera>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    editor: Res<Editor>,
    windows: Query<&Window>,
) {
    let (orbit, transform) = &mut *camera;

    if editor.view != ViewMode::Mesh {
        return;
    }

    let over_viewport = windows
        .single()
        .ok()
        .and_then(|window| window.cursor_position().map(|cursor| (window, cursor)))
        .is_some_and(|(window, cursor)| {
            cursor.x > PANEL_LEFT
                && cursor.x < window.width() - PANEL_RIGHT
                && cursor.y > BAR_TOP
                && cursor.y < window.height() - BAR_BOTTOM
        });

    if over_viewport {
        if buttons.pressed(MouseButton::Left) {
            orbit.yaw -= motion.delta.x * 0.006;
            orbit.pitch =
                (orbit.pitch + motion.delta.y * 0.006).clamp(-FRAC_PI_2 + 0.05, FRAC_PI_2 - 0.05);
        }
        if buttons.pressed(MouseButton::Middle) {
            let right = transform.right() * -motion.delta.x * 0.004 * orbit.distance;
            let up = transform.up() * motion.delta.y * 0.004 * orbit.distance;
            orbit.focus += right + up;
        }
        if scroll.delta.y != 0.0 {
            orbit.distance = (orbit.distance * (1.0 - scroll.delta.y * 0.12)).clamp(0.6, 40.0);
        }
    }

    let offset = Vec3::new(
        orbit.pitch.cos() * orbit.yaw.sin(),
        orbit.pitch.sin(),
        orbit.pitch.cos() * orbit.yaw.cos(),
    ) * orbit.distance;
    transform.translation = orbit.focus + offset;
    transform.look_at(orbit.focus, Vec3::Y);
}

fn turntable(
    editor: Res<Editor>,
    time: Res<Time>,
    mut subject: Single<&mut Transform, With<PreviewSubject>>,
) {
    if editor.turntable {
        subject.rotate_y(time.delta_secs() * TAU * 0.08);
    }
}

fn follow_sun_controls(editor: Res<Editor>, mut sun: Single<&mut Transform, With<Sun>>) {
    if !editor.is_changed() {
        return;
    }
    **sun = Transform::default().looking_at(-editor.sun_direction(), Vec3::Y);
}

fn swap_preview_mesh(
    editor: Res<Editor>,
    assets: Res<PreviewAssets>,
    mut subject: Single<&mut Mesh3d, With<PreviewSubject>>,
) {
    if !editor.is_changed() {
        return;
    }
    let wanted = match editor.mesh {
        PreviewMesh::Cube => 0,
        PreviewMesh::Sphere => 1,
        PreviewMesh::Plane => 2,
        PreviewMesh::Quad => 3,
    };
    let handle = assets.meshes[wanted].clone();
    if subject.0 != handle {
        subject.0 = handle;
    }
}

fn toggle_subject_visibility(
    editor: Res<Editor>,
    mut subject: Single<&mut Visibility, With<PreviewSubject>>,
    mut camera: Single<&mut Camera, With<PreviewCamera>>,
) {
    let showing_mesh = editor.view == ViewMode::Mesh;
    let wanted = if showing_mesh {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if **subject != wanted {
        **subject = wanted;
    }
    if camera.is_active != showing_mesh {
        camera.is_active = showing_mesh;
    }
}

fn fit_viewport(windows: Query<&Window>, mut camera: Single<&mut Camera, With<PreviewCamera>>) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = window.scale_factor();
    let width = window.physical_width() as f32;
    let height = window.physical_height() as f32;

    let left = PANEL_LEFT * scale;
    let right = PANEL_RIGHT * scale;
    let top = BAR_TOP * scale;
    let bottom = BAR_BOTTOM * scale;

    let size = Vec2::new(width - left - right, height - top - bottom);
    if size.x < 8.0 || size.y < 8.0 {
        camera.viewport = None;
        return;
    }

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(left as u32, top as u32),
        physical_size: size.as_uvec2(),
        ..default()
    });
}
