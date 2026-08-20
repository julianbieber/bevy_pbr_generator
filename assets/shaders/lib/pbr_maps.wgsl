#define_import_path pbr_gen::maps

struct Globals {
    resolution: vec2<u32>,
    time: f32,
    seed: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var map_base_color: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var map_normal: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var map_orm: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4) var map_emissive: texture_storage_2d<rgba16float, write>;
@group(0) @binding(5) var map_transmission: texture_storage_2d<rgba16float, write>;
@group(0) @binding(6) var map_specular: texture_storage_2d<rgba16float, write>;
@group(0) @binding(7) var map_clearcoat: texture_storage_2d<rgba16float, write>;
@group(0) @binding(8) var map_clearcoat_normal: texture_storage_2d<rgba16float, write>;
@group(0) @binding(9) var map_anisotropy: texture_storage_2d<rgba16float, write>;
@group(0) @binding(10) var map_depth: texture_storage_2d<rgba16float, write>;

struct Surface {
    base_color: vec3<f32>,
    opacity: f32,
    normal: vec3<f32>,
    occlusion: f32,
    roughness: f32,
    metallic: f32,
    emissive: vec3<f32>,
    specular: f32,
    specular_tint: vec3<f32>,
    diffuse_transmission: f32,
    specular_transmission: f32,
    thickness: f32,
    clearcoat: f32,
    clearcoat_roughness: f32,
    clearcoat_normal: vec3<f32>,
    anisotropy_direction: vec2<f32>,
    anisotropy_strength: f32,
    depth: f32,
}

fn default_surface() -> Surface {
    var s: Surface;
    s.base_color = vec3<f32>(0.5, 0.5, 0.5);
    s.opacity = 1.0;
    s.normal = vec3<f32>(0.0, 0.0, 1.0);
    s.occlusion = 1.0;
    s.roughness = 0.5;
    s.metallic = 0.0;
    s.emissive = vec3<f32>(0.0, 0.0, 0.0);
    s.specular = 0.5;
    s.specular_tint = vec3<f32>(1.0, 1.0, 1.0);
    s.diffuse_transmission = 0.0;
    s.specular_transmission = 0.0;
    s.thickness = 0.0;
    s.clearcoat = 0.0;
    s.clearcoat_roughness = 0.0;
    s.clearcoat_normal = vec3<f32>(0.0, 0.0, 1.0);
    s.anisotropy_direction = vec2<f32>(1.0, 0.0);
    s.anisotropy_strength = 0.0;
    s.depth = 0.0;
    return s;
}

fn texel_size() -> vec2<f32> {
    return vec2<f32>(1.0) / vec2<f32>(globals.resolution);
}

fn uv_of(id: vec2<u32>) -> vec2<f32> {
    return (vec2<f32>(id) + vec2<f32>(0.5)) * texel_size();
}

fn in_bounds(id: vec2<u32>) -> bool {
    return id.x < globals.resolution.x && id.y < globals.resolution.y;
}

fn normal_epsilon() -> f32 {
    return 1.0 / 512.0;
}

fn normal_from_slope(dx: f32, dy: f32, strength: f32) -> vec3<f32> {
    return normalize(vec3<f32>(-dx * strength, -dy * strength, 1.0));
}

fn encode_normal(n: vec3<f32>) -> vec3<f32> {
    let unit = normalize(n);
    return vec3<f32>(unit.x, -unit.y, unit.z) * 0.5 + vec3<f32>(0.5);
}

fn encode_direction(d: vec2<f32>) -> vec2<f32> {
    let len = length(d);
    var unit = vec2<f32>(1.0, 0.0);
    if len > 0.000001 {
        unit = d / len;
    }
    return unit * 0.5 + vec2<f32>(0.5);
}

fn write_surface(id: vec2<u32>, s: Surface) {
    textureStore(map_base_color, id, vec4<f32>(s.base_color, s.opacity));
    textureStore(map_normal, id, vec4<f32>(encode_normal(s.normal), 1.0));
    textureStore(map_orm, id, vec4<f32>(s.occlusion, s.roughness, s.metallic, 1.0));
    textureStore(map_emissive, id, vec4<f32>(s.emissive, 1.0));
    textureStore(map_transmission, id, vec4<f32>(s.specular_transmission, s.thickness, 0.0, s.diffuse_transmission));
    textureStore(map_specular, id, vec4<f32>(s.specular_tint, s.specular));
    textureStore(map_clearcoat, id, vec4<f32>(s.clearcoat, s.clearcoat_roughness, 0.0, 1.0));
    textureStore(map_clearcoat_normal, id, vec4<f32>(encode_normal(s.clearcoat_normal), 1.0));
    textureStore(map_anisotropy, id, vec4<f32>(encode_direction(s.anisotropy_direction), s.anisotropy_strength, 1.0));
    textureStore(map_depth, id, vec4<f32>(s.depth, 0.0, 0.0, 1.0));
}
