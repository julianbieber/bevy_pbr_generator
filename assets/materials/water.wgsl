// @material Water

#import pbr_gen::maps::{Surface, default_surface, write_surface, globals, uv_of, in_bounds, normal_from_slope, normal_epsilon}
#import pbr_gen::noise::{set_noise_seed, perlin_noise, simplex_noise, fbm_perlin, to_unit}

struct Params {
    // @group Colour
    deep_color: vec3<f32>,          // @ui "Deep" color srgb(0.02, 0.15, 0.40)
    shallow_color: vec3<f32>,       // @ui "Shallow" color srgb(0.05, 0.35, 0.55)
    crest_color: vec3<f32>,         // @ui "Crest" color srgb(0.70, 0.85, 0.95)
    specular_tint: vec3<f32>,       // @ui "Specular tint" color srgb(0.90, 0.95, 1.00)
    emissive_deep: vec3<f32>,       // @ui "Glow, deep" color srgb(0.10, 0.30, 0.60)
    emissive_crest: vec3<f32>,      // @ui "Glow, crest" color srgb(0.70, 0.85, 0.95)

    // @group Waves
    wave_scale: f32,                // @ui "Wave scale" 8.0 [1.0, 64.0]
    ripple_scale: f32,              // @ui "Ripple scale" 25.0 [1.0, 128.0]
    wave_frequency: f32,            // @ui "Crest frequency" 2.0 [0.5, 16.0]
    wave_octaves: i32,              // @ui "Octaves" 3 [1, 8]
    crest_sharpness: f32,           // @ui "Crest sharpness" 2.0 [1.0, 8.0]
    normal_strength: f32,           // @ui "Normal strength" 6.0 [0.0, 48.0]

    // @group Surface
    base_roughness: f32,            // @ui "Roughness, calm" 0.15 [0.0, 1.0]
    crest_roughness: f32,           // @ui "Roughness, crest" 0.40 [0.0, 1.0]
    specular: f32,                  // @ui "Specular" 0.5 [0.0, 1.0]
    occlusion_floor: f32,           // @ui "Occlusion floor" 0.90 [0.0, 1.0]
    anisotropy_strength: f32,       // @ui "Anisotropy" 0.30 [0.0, 1.0]
    depth_scale: f32,               // @ui "Parallax depth" 0.05 [0.0, 0.5]

    // @group Transmission
    diffuse_transmission: f32,      // @ui "Diffuse" 0.90 [0.0, 1.0]
    specular_transmission: f32,     // @ui "Specular" 0.50 [0.0, 1.0]
    thickness: f32,                 // @ui "Thickness" 0.10 [0.0, 1.0]
    emissive_intensity: f32,        // @ui "Emissive" 0.05 [0.0, 2.0]
}

@group(1) @binding(0) var<uniform> params: Params;

fn wave_height(uv: vec2<f32>) -> f32 {
    let drift = globals.time * 0.05;
    let p = uv + vec2<f32>(drift, drift * 0.37);
    return perlin_noise(p, params.wave_scale * 0.8) * 0.5
        + perlin_noise(p, params.wave_scale * 1.5) * 0.3
        + perlin_noise(p, params.ripple_scale) * 0.2
        + sin((p.x + p.y) * params.wave_scale * 0.5) * 0.15;
}

fn crest(uv: vec2<f32>, frequency_gain: f32) -> f32 {
    let n = simplex_noise(uv * params.wave_scale, params.wave_frequency * frequency_gain);
    return pow(to_unit(n), params.crest_sharpness);
}

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();

    let depth_factor = to_unit(perlin_noise(uv, params.wave_scale * 0.5));
    let ripple_factor = to_unit(perlin_noise(uv, params.ripple_scale));
    let crest_factor = crest(uv, 1.0);

    let base = mix(params.deep_color, params.shallow_color, depth_factor);
    let crest_mix = mix(params.crest_color, base, 0.7);
    let tinted = mix(base, crest_mix, crest_factor * 0.3);
    s.base_color = clamp(tinted + vec3<f32>(0.02 * (ripple_factor - 0.5)), vec3<f32>(0.0), vec3<f32>(1.0));
    s.opacity = 1.0;

    let e = normal_epsilon();
    let dx = wave_height(uv + vec2<f32>(e, 0.0)) - wave_height(uv - vec2<f32>(e, 0.0));
    let dy = wave_height(uv + vec2<f32>(0.0, e)) - wave_height(uv - vec2<f32>(0.0, e));
    s.normal = normal_from_slope(dx, dy, params.normal_strength);

    let wave_factor = to_unit(fbm_perlin(uv, params.wave_scale, params.wave_octaves, 2.0, 0.5));
    let sharp_crest = crest(uv, 2.0);
    let spread = params.crest_roughness - params.base_roughness;
    s.roughness = clamp(params.base_roughness + spread * sharp_crest + spread * wave_factor * 0.3, 0.0, 1.0);
    s.metallic = 0.0;
    s.occlusion = clamp(params.occlusion_floor + 0.05 - depth_factor * 0.05, 0.0, 1.0);

    let deep_glow = params.emissive_deep * params.emissive_intensity;
    let crest_glow = params.emissive_crest * params.emissive_intensity * 1.5;
    s.emissive = mix(deep_glow, crest_glow, crest_factor);

    s.specular = params.specular;
    s.specular_tint = params.specular_tint;

    s.diffuse_transmission = params.diffuse_transmission;
    s.specular_transmission = params.specular_transmission;
    s.thickness = params.thickness;

    let wave_angle = uv.x * params.wave_scale * 0.5;
    s.anisotropy_direction = vec2<f32>(sin(wave_angle), cos(wave_angle));
    s.anisotropy_strength = clamp(to_unit(perlin_noise(uv, params.wave_scale)) * params.anisotropy_strength, 0.0, 1.0);

    s.depth = clamp(to_unit(perlin_noise(uv, params.wave_scale)) * params.depth_scale, 0.0, 1.0);

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
