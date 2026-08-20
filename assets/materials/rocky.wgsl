// @material Rocky Ground

#import pbr_gen::maps::{Surface, default_surface, write_surface, globals, uv_of, in_bounds, normal_from_slope, normal_epsilon}
#import pbr_gen::noise::{set_noise_seed, perlin_noise, simplex_noise, fbm_perlin, worley_edges, to_unit}

struct Params {
    // @group Colour
    base_color: vec3<f32>,          // @ui "Rock, dark" color srgb(0.25, 0.18, 0.12)
    mid_color: vec3<f32>,           // @ui "Rock, mid" color srgb(0.45, 0.32, 0.20)
    highlight_color: vec3<f32>,     // @ui "Rock, highlight" color srgb(0.65, 0.55, 0.45)
    dry_color: vec3<f32>,           // @ui "Weathered" color srgb(0.75, 0.70, 0.65)
    specular_tint: vec3<f32>,       // @ui "Specular tint" color srgb(0.80, 0.75, 0.70)
    emissive: vec3<f32>,            // @ui "Emissive" color srgb(0.01, 0.005, 0.002)

    // @group Structure
    formation_scale: f32,           // @ui "Formation scale" 12.0 [1.0, 64.0]
    detail_scale: f32,              // @ui "Detail scale" 30.0 [1.0, 128.0]
    grain_scale: f32,               // @ui "Grain scale" 50.0 [1.0, 256.0]
    octaves: i32,                   // @ui "Octaves" 4 [1, 8]
    crack_scale: f32,               // @ui "Crack scale" 10.0 [1.0, 64.0]
    crack_depth: f32,               // @ui "Crack depth" 0.35 [0.0, 1.0]
    normal_strength: f32,           // @ui "Normal strength" 6.0 [0.0, 48.0]

    // @group Surface
    base_roughness: f32,            // @ui "Roughness" 0.85 [0.0, 1.0]
    roughness_variation: f32,       // @ui "Roughness variation" 0.10 [0.0, 0.5]
    metallic: f32,                  // @ui "Metallic" 0.02 [0.0, 1.0]
    specular: f32,                  // @ui "Specular" 0.05 [0.0, 1.0]
    occlusion_base: f32,            // @ui "Occlusion base" 0.60 [0.0, 1.0]
    weathering: f32,                // @ui "Weathering" 0.20 [0.0, 1.0]
    anisotropy_strength: f32,       // @ui "Anisotropy" 0.05 [0.0, 1.0]
    depth_scale: f32,               // @ui "Parallax depth" 0.15 [0.0, 0.5]
}

@group(1) @binding(0) var<uniform> params: Params;

fn rock_height(uv: vec2<f32>) -> f32 {
    let bulk = perlin_noise(uv, params.formation_scale * 0.6) * 0.5
        + perlin_noise(uv, params.formation_scale * 1.2) * 0.3
        + perlin_noise(uv, params.detail_scale) * 0.15
        + perlin_noise(uv, params.grain_scale) * 0.05;
    let crack = 1.0 - smoothstep(0.0, 0.25, worley_edges(uv, params.crack_scale));
    return bulk - crack * params.crack_depth;
}

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();

    let formation = to_unit(perlin_noise(uv, params.formation_scale * 0.8));
    let detail = to_unit(perlin_noise(uv, params.detail_scale));
    let grain = to_unit(perlin_noise(uv, params.grain_scale));

    let low = mix(params.base_color, params.mid_color, formation);
    let high = mix(params.mid_color, params.highlight_color, detail);
    let mixed = mix(low, high, 0.4);

    let dry = pow(to_unit(simplex_noise(uv * params.detail_scale, 2.0)), 2.0);
    let weathered = mix(mixed, params.dry_color, dry * params.weathering);
    s.base_color = clamp(weathered + vec3<f32>(0.03 * (grain - 0.5)), vec3<f32>(0.0), vec3<f32>(1.0));
    s.opacity = 1.0;

    let e = normal_epsilon();
    let dx = rock_height(uv + vec2<f32>(e, 0.0)) - rock_height(uv - vec2<f32>(e, 0.0));
    let dy = rock_height(uv + vec2<f32>(0.0, e)) - rock_height(uv - vec2<f32>(0.0, e));
    s.normal = normal_from_slope(dx, dy, params.normal_strength);

    let variation = to_unit(fbm_perlin(uv, params.formation_scale, params.octaves, 2.0, 0.5));
    s.roughness = clamp(params.base_roughness + (variation - 0.5) * params.roughness_variation * 2.0, 0.0, 1.0);
    s.metallic = params.metallic;

    let formation_shadow = to_unit(perlin_noise(uv, params.formation_scale * 0.5));
    let detail_shadow = to_unit(perlin_noise(uv, params.detail_scale));
    let crack_shadow = 1.0 - smoothstep(0.0, 0.2, worley_edges(uv, params.crack_scale));
    s.occlusion = clamp(
        params.occlusion_base - formation_shadow * 0.25 - detail_shadow * 0.15 - crack_shadow * 0.2,
        0.0,
        1.0,
    );

    s.emissive = params.emissive;
    s.specular = params.specular;
    s.specular_tint = params.specular_tint;

    s.diffuse_transmission = 0.0;
    s.specular_transmission = 0.0;
    s.thickness = 0.0;

    s.anisotropy_direction = vec2<f32>(1.0, 0.0);
    s.anisotropy_strength = clamp(to_unit(perlin_noise(uv, params.formation_scale * 0.3)) * params.anisotropy_strength, 0.0, 1.0);

    s.depth = clamp(to_unit(perlin_noise(uv, params.formation_scale * 0.8)) * params.depth_scale, 0.0, 1.0);

    return s;
}

@compute @workgroup_size(8, 8, 1)
fn generate(@builtin(global_invocation_id) id: vec3<u32>) {
    if !in_bounds(id.xy) {
        return;
    }
    set_noise_seed(globals.seed);
    write_surface(id.xy, surface(uv_of(id.xy)));
}
