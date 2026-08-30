// @material Rocky
// @author bevy_pbr_generator

#import pbr_gen::maps::{Surface, default_surface, write_surface, globals}
#import pbr_gen::noise::{perlin_noise, simplex_noise, fbm_perlin}

struct Params {
    // @group Colour
    base_color: vec3<f32>,      // @ui "Base" color srgb(0.25, 0.18, 0.12)
    mid_color: vec3<f32>,       // @ui "Mid" color srgb(0.45, 0.32, 0.20)
    highlight_color: vec3<f32>, // @ui "Highlight" color srgb(0.65, 0.55, 0.45)
    dry_color: vec3<f32>,       // @ui "Dry" color srgb(0.75, 0.70, 0.65)
    emissive: vec3<f32>,        // @ui "Emissive" color srgb(0.010, 0.005, 0.002)
    specular_tint: vec3<f32>,   // @ui "Specular tint" color srgb(0.80, 0.75, 0.70)
    // @group Formation
    rock_scale: f32,            // @ui "Scale" 12.0 [0.1, 48.0]
    detail_scale: f32,          // @ui "Detail" 30.0 [0.1, 96.0]
    grain_scale: f32,           // @ui "Grain" 50.0 [0.1, 128.0]
    octaves: i32,               // @ui 4 [1, 8]
    dry_amount: f32,            // @ui "Dryness" 0.20 [0.0, 1.0]
    // @group Surface
    base_roughness: f32,        // @ui "Roughness" 0.85 [0.0, 1.0]
    roughness_variation: f32,   // @ui "Variation" 0.10 [0.0, 1.0]
    metallic: f32,              // @ui 0.02 [0.0, 1.0]
    specular: f32,              // @ui 0.05 [0.0, 1.0]
    // @group Relief
    base_occlusion: f32,        // @ui "Occlusion" 0.60 [0.0, 1.0]
    depth_scale: f32,           // @ui "Parallax depth" 0.15 [0.0, 0.5]
    anisotropy_strength: f32,   // @ui "Anisotropy" 0.05 [0.0, 1.0]
}
@group(1) @binding(0) var<uniform> params: Params;

const NORMAL_EPSILON: f32 = 1.0 / 512.0;

fn height(uv: vec2<f32>) -> f32 {
    return perlin_noise(uv, params.rock_scale * 0.6) * 0.5
        + perlin_noise(uv, params.rock_scale * 1.2) * 0.3
        + perlin_noise(uv, params.detail_scale) * 0.15
        + perlin_noise(uv, params.grain_scale) * 0.05;
}

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();

    let formation_factor = (perlin_noise(uv, params.rock_scale * 0.8) + 1.0) * 0.5;
    let detail_factor = (perlin_noise(uv, params.detail_scale) + 1.0) * 0.5;
    let grain_factor = (perlin_noise(uv, params.grain_scale) + 1.0) * 0.5;

    let base = mix(params.base_color, params.mid_color, formation_factor);
    let detail = mix(params.mid_color, params.highlight_color, detail_factor);
    let mixed = mix(base, detail, 0.4);
    let grain_variation = vec3<f32>(0.03) * (grain_factor - 0.5);

    let dry_noise = simplex_noise(uv * params.detail_scale, 2.0);
    let dry_t = (dry_noise + 1.0) * 0.5;
    let tinted = mix(mixed, params.dry_color, dry_t * dry_t * params.dry_amount);

    s.base_color = clamp(tinted + grain_variation, vec3<f32>(0.0), vec3<f32>(1.0));
    s.opacity = 1.0;

    s.emissive = params.emissive;

    let noise_factor = (fbm_perlin(uv, params.rock_scale, u32(max(params.octaves, 1)), 2.0, 0.5) + 1.0) * 0.5;
    let spread = params.roughness_variation * 2.0;
    s.roughness = clamp(
        params.base_roughness + (noise_factor - 0.5) * spread,
        max(params.base_roughness - params.roughness_variation, 0.0),
        min(params.base_roughness + params.roughness_variation + 0.03, 1.0),
    );
    s.metallic = params.metallic;

    let formation_shadow = (perlin_noise(uv, params.rock_scale * 0.5) + 1.0) * 0.5;
    let detail_shadow = (perlin_noise(uv, params.detail_scale) + 1.0) * 0.5;
    s.occlusion = clamp(
        params.base_occlusion - formation_shadow * 0.25 - detail_shadow * 0.15,
        0.0,
        1.0,
    );

    let dzdx = (height(uv + vec2<f32>(NORMAL_EPSILON, 0.0)) - height(uv - vec2<f32>(NORMAL_EPSILON, 0.0)))
        / (2.0 * NORMAL_EPSILON);
    let dzdy = (height(uv + vec2<f32>(0.0, NORMAL_EPSILON)) - height(uv - vec2<f32>(0.0, NORMAL_EPSILON)))
        / (2.0 * NORMAL_EPSILON);
    s.normal = normalize(vec3<f32>(-dzdx, -dzdy, 1.0));

    s.specular_tint = params.specular_tint;
    s.specular = params.specular;

    s.anisotropy_strength = clamp(
        (perlin_noise(uv, params.rock_scale * 0.3) + 1.0) * 0.5 * params.anisotropy_strength,
        0.0,
        1.0,
    );

    s.depth = clamp((perlin_noise(uv, params.rock_scale * 0.8) + 1.0) * 0.5 * params.depth_scale, 0.0, 1.0);

    return s;
}

@compute @workgroup_size(8, 8, 1)
fn generate(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= globals.resolution.x || id.y >= globals.resolution.y {
        return;
    }
    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(globals.resolution);
    write_surface(id.xy, surface(uv));
}
