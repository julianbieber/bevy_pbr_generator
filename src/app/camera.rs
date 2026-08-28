//! The two cameras the editor renders with, and the orbit control over the 3D
//! one.

use bevy::camera::ClearColorConfig;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{Atmosphere, AtmosphereEnvironmentMapLight};
use bevy::prelude::*;

use crate::app::{EditorSettings, PreviewShape};

/// Orbit state of the preview camera.
#[derive(Component, Debug, Clone, Copy)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub distance: f32,
    /// Rotation about the world's up axis, in radians.
    pub yaw: f32,
    /// Rotation above the horizon, in radians, kept clear of the poles.
    pub pitch: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            distance: 3.0,
            yaw: 0.6,
            pitch: 0.45,
        }
    }
}

impl OrbitCamera {
    const PITCH_LIMIT: f32 = 1.54;
    const MIN_DISTANCE: f32 = 0.6;
    const MAX_DISTANCE: f32 = 20.0;

    /// The transform this orbit state puts the camera at.
    pub fn transform(&self) -> Transform {
        let direction = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        Transform::from_translation(self.focus + direction * self.distance)
            .looking_at(self.focus, Vec3::Y)
    }

    fn clamp(&mut self) {
        self.pitch = self.pitch.clamp(-Self::PITCH_LIMIT, Self::PITCH_LIMIT);
        self.distance = self.distance.clamp(Self::MIN_DISTANCE, Self::MAX_DISTANCE);
    }
}

/// The camera the preview mesh is drawn by, as opposed to the UI camera.
#[derive(Component, Debug)]
pub struct PreviewCamera;

/// Spawns the preview camera behind the UI camera.
///
/// The 3D camera covers the whole window and draws first; the UI camera draws
/// over it without clearing, so the opaque panels hide the 3D image and the
/// transparent centre panel lets it through. Confining the 3D camera to a
/// viewport rect instead renders nothing at all in that rect.
pub fn spawn_cameras(mut commands: Commands, mut media: ResMut<Assets<ScatteringMedium>>) {
    let orbit = OrbitCamera::default();
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        orbit.transform(),
        orbit,
        PreviewCamera,
        Atmosphere::earth(media.add(ScatteringMedium::earth(256, 128))),
        AtmosphereEnvironmentMapLight::default(),
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
}

/// Left drag orbits, middle drag pans, the wheel zooms.
pub fn orbit_camera(
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform), With<PreviewCamera>>,
) {
    let Ok((mut orbit, mut transform)) = cameras.single_mut() else {
        return;
    };

    let drag: Vec2 = motion.read().map(|m| m.delta).sum();
    let scroll: f32 = wheel.read().map(|w| w.y).sum();

    if drag == Vec2::ZERO && scroll == 0.0 {
        return;
    }

    if buttons.pressed(MouseButton::Left) {
        orbit.yaw -= drag.x * 0.005;
        orbit.pitch += drag.y * 0.005;
    }

    if buttons.pressed(MouseButton::Middle) {
        let right = transform.right() * -drag.x * orbit.distance * 0.002;
        let up = transform.up() * drag.y * orbit.distance * 0.002;
        orbit.focus += right + up;
    }

    if scroll != 0.0 {
        orbit.distance *= 1.0 - scroll * 0.1;
    }

    orbit.clamp();
    *transform = orbit.transform();
}

/// Spins the mesh rather than the camera, so the lighting sweeps across the
/// surface instead of travelling with the viewer.
pub fn turntable(
    settings: Res<EditorSettings>,
    time: Res<Time>,
    mut meshes: Query<&mut Transform, With<PreviewShapeMarker>>,
) {
    if !settings.turntable {
        return;
    }
    for mut transform in &mut meshes {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}

/// Marks the entity carrying the preview mesh.
#[derive(Component, Debug)]
pub struct PreviewShapeMarker(pub PreviewShape);
