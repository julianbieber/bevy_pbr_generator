//! Bevy PBR Material Builder Example
use bevy::prelude::*;
use bevy_pbr_builder::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PbrMaterialBuilderPlugin)
        .add_systems(Startup, setup_pbr_material)
        .run();
}

fn setup_pbr_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let config = PbrMaterialConfig {
        name_prefix: "gold_material".to_string(),
        parameters: PbrParameters {
            base_color_factor: [1.0, 0.75, 0.33, 1.0],
            metallic_factor: 1.0,
            roughness_factor: 0.1,
            ..default()
        },
        ..default()
    };

    let builder = PbrMaterialBuilder::new(config);
    
    let base_color_texture = asset_server.load("textures/base_color.png");
    let metallic_roughness_texture = asset_server.load("textures/metallic_roughness.png");
    let normal_texture = asset_server.load("textures/normal.png");
    
    let builder = builder
        .with_texture(PbrTextureType::BaseColor, base_color_texture)
        .with_texture(PbrTextureType::MetallicRoughness, metallic_roughness_texture)
        .with_texture(PbrTextureType::Normal, normal_texture);
    
    let material = builder.build(&asset_server, &mut materials).unwrap();
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube::default())),
        material: materials.add(material),
        ..default()
    });
    
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            range: 10.0,
            ..default()
        },
        transform: Transform::from_xyz(2.0, 2.0, 2.0),
        ..default()
    });
    
    println!("PBR Material Builder initialized!");
    println!("Shader library has {} functions", builder.shader_library().functions.len());
}
