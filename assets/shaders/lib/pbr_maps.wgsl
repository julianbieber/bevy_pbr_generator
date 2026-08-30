// The PBR surface description a material produces, and its packing into the ten
// storage textures `StandardMaterial` samples.
//
// The channel layout committed to here is derived in
// BEVY_STANDARD_MATERIAL_TEXTURE_PACKING.md.

#define_import_path pbr_gen::maps

struct Globals {
    resolution: vec2<u32>,
    time: f32,
    seed: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var base_color_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var normal_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var orm_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4) var emissive_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(5) var transmission_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(6) var specular_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(7) var clearcoat_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(8) var clearcoat_normal_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(9) var anisotropy_map: texture_storage_2d<rgba16float, write>;
@group(0) @binding(10) var depth_map: texture_storage_2d<rgba16float, write>;

// Every PBR property at one point on the surface, in its natural range and
// before any channel packing.
//
// `normal` and `clearcoat_normal` are tangent-space vectors in [-1, 1] with +X
// right, +Y up and +Z out of the surface; `write_surface` applies the remap to
// [0, 1] and the green flip Bevy's OpenGL convention expects, so a material must
// not pre-encode them. `anisotropy_direction` is likewise a raw direction in
// [-1, 1], and must never be the zero vector: Bevy normalises it, so a zero
// direction is a NaN that blackens every lit fragment. `anisotropy_strength` is
// what turns anisotropy off. Everything else is in [0, 1] except `emissive`,
// which is unbounded above.
struct Surface {
    base_color: vec3<f32>,
    opacity: f32,
    normal: vec3<f32>,
    occlusion: f32,
    roughness: f32,
    metallic: f32,
    emissive: vec3<f32>,
    specular_transmission: f32,
    thickness: f32,
    diffuse_transmission: f32,
    specular_tint: vec3<f32>,
    specular: f32,
    clearcoat: f32,
    clearcoat_roughness: f32,
    clearcoat_normal: vec3<f32>,
    anisotropy_direction: vec2<f32>,
    anisotropy_strength: f32,
    depth: f32,
}

// A neutral mid-grey dielectric: opaque, flat-normalled, fully lit, no
// transmission, no clearcoat and no anisotropy. Materials are expected to start
// here and overwrite only the properties they care about, so that a property
// added to `Surface` later has a defined value in every existing material.
fn default_surface() -> Surface {
    var s: Surface;
    s.base_color = vec3<f32>(0.5, 0.5, 0.5);
    s.opacity = 1.0;
    s.normal = vec3<f32>(0.0, 0.0, 1.0);
    s.occlusion = 1.0;
    s.roughness = 0.5;
    s.metallic = 0.0;
    s.emissive = vec3<f32>(0.0, 0.0, 0.0);
    s.specular_transmission = 0.0;
    s.thickness = 0.0;
    s.diffuse_transmission = 0.0;
    s.specular_tint = vec3<f32>(1.0, 1.0, 1.0);
    s.specular = 0.5;
    s.clearcoat = 0.0;
    s.clearcoat_roughness = 0.0;
    s.clearcoat_normal = vec3<f32>(0.0, 0.0, 1.0);
    s.anisotropy_direction = vec2<f32>(1.0, 0.0);
    s.anisotropy_strength = 0.0;
    s.depth = 0.0;
    return s;
}

fn encode_normal(n: vec3<f32>) -> vec3<f32> {
    let unit = normalize(n);
    return vec3<f32>(unit.x * 0.5 + 0.5, -unit.y * 0.5 + 0.5, unit.z * 0.5 + 0.5);
}

// Packs `s` into the ten storage textures at `coord`, applying the normal
// encoding, the anisotropy direction encoding and the ORM interleave. Values are
// written linearly: the sRGB transfer function belongs at PNG encode time, not
// here, so that the preview and the export agree.
fn write_surface(coord: vec2<u32>, s: Surface) {
    let c = vec2<i32>(coord);

    textureStore(base_color_map, c, vec4<f32>(s.base_color, s.opacity));
    textureStore(normal_map, c, vec4<f32>(encode_normal(s.normal), 1.0));
    textureStore(orm_map, c, vec4<f32>(s.occlusion, s.roughness, s.metallic, 1.0));
    textureStore(emissive_map, c, vec4<f32>(s.emissive, 1.0));
    textureStore(
        transmission_map,
        c,
        vec4<f32>(s.specular_transmission, s.thickness, 0.0, s.diffuse_transmission),
    );
    textureStore(specular_map, c, vec4<f32>(s.specular_tint, s.specular));
    textureStore(clearcoat_map, c, vec4<f32>(s.clearcoat, s.clearcoat_roughness, 0.0, 1.0));
    textureStore(clearcoat_normal_map, c, vec4<f32>(encode_normal(s.clearcoat_normal), 1.0));

    let dir = s.anisotropy_direction * 0.5 + 0.5;
    textureStore(anisotropy_map, c, vec4<f32>(dir, s.anisotropy_strength, 1.0));

    textureStore(depth_map, c, vec4<f32>(s.depth, 0.0, 0.0, 1.0));
}
