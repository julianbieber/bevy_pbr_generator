// @material Water
// @author bevy_pbr_generator

#import pbr_gen::maps::{Surface, default_surface, write_surface, globals}
#import pbr_gen::noise::{perlin_noise, simplex_noise, fbm_perlin}

struct Params {
    // @group Colour
    deep_color: vec3<f32>,     // @ui "Deep" color srgb(0.02, 0.15, 0.40)
    shallow_color: vec3<f32>,  // @ui "Shallow" color srgb(0.05, 0.35, 0.55)
    crest_color: vec3<f32>,    // @ui "Crest" color srgb(0.70, 0.85, 0.95)
    specular_tint: vec3<f32>,  // @ui "Specular tint" color srgb(0.90, 0.95, 1.00)
    // @group Waves
    wave_scale: f32,           // @ui 8.0 [0.1, 32.0]
    ripple_scale: f32,         // @ui 25.0 [0.1, 64.0]
    wave_frequency: f32,       // @ui 2.0 [0.1, 8.0]
    wave_octaves: i32,         // @ui 3 [1, 8]
    // @group Surface
    base_roughness: f32,       // @ui 0.15 [0.0, 1.0]
    crest_roughness: f32,      // @ui 0.40 [0.0, 1.0]
    specular: f32,             // @ui 0.5 [0.0, 1.0]
    // @group Transmission
    diffuse_transmission: f32,  // @ui "Diffuse" 0.90 [0.0, 1.0]
    specular_transmission: f32, // @ui "Specular" 0.50 [0.0, 1.0]
    thickness: f32,             // @ui 0.10 [0.0, 1.0]
    // @group Relief
    emissive_intensity: f32,    // @ui "Emissive" 0.05 [0.0, 1.0]
    depth_scale: f32,           // @ui "Parallax depth" 0.05 [0.0, 0.5]
    anisotropy_strength: f32,   // @ui "Anisotropy" 0.30 [0.0, 1.0]
}
@group(1) @binding(0) var<uniform> params: Params;

const NORMAL_EPSILON: f32 = 1.0 / 512.0;

fn height(uv: vec2<f32>) -> f32 {
    return perlin_noise(uv, params.wave_scale * 0.8) * 0.5
        + perlin_noise(uv, params.wave_scale * 1.5) * 0.3
        + perlin_noise(uv, params.ripple_scale) * 0.2
        + sin(uv.x * params.wave_scale * 0.5 + uv.y * params.wave_scale * 0.5) * 0.15;
}

fn crest_factor(uv: vec2<f32>) -> f32 {
    let n = simplex_noise(uv * params.wave_scale, params.wave_frequency);
    let t = (n + 1.0) * 0.5;
    return t * t;
}

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();

    let depth_factor = (perlin_noise(uv, params.wave_scale * 0.5) + 1.0) * 0.5;
    let ripple_factor = (perlin_noise(uv, params.ripple_scale) + 1.0) * 0.5;
    let crest = crest_factor(uv);

    let base = mix(params.deep_color, params.shallow_color, depth_factor);
    let crest_tinted = mix(params.crest_color, base, 0.7);
    let tinted = mix(base, crest_tinted, crest * 0.3);
    let ripple_variation = vec3<f32>(0.02) * (ripple_factor - 0.5);

    s.base_color = clamp(tinted + ripple_variation, vec3<f32>(0.0), vec3<f32>(1.0));
    s.opacity = 1.0;

    let deep_glow = vec3<f32>(0.1, 0.3, 0.6) * params.emissive_intensity;
    let crest_glow = vec3<f32>(0.7, 0.85, 0.95) * params.emissive_intensity * 1.5;
    s.emissive = mix(deep_glow, crest_glow, crest);

    let wave_factor = (fbm_perlin(uv, params.wave_scale, u32(max(params.wave_octaves, 1)), 2.0, 0.5) + 1.0) * 0.5;
    let sharp_crest = pow((simplex_noise(uv * params.wave_scale * 1.5, params.wave_frequency * 2.0) + 1.0) * 0.5, 3.0);
    let spread = params.crest_roughness - params.base_roughness;
    s.roughness = clamp(
        params.base_roughness + spread * sharp_crest + spread * wave_factor * 0.3,
        0.0,
        1.0,
    );
    s.metallic = 0.0;

    let shadow_factor = (perlin_noise(uv, params.wave_scale * 0.5) + 1.0) * 0.5;
    s.occlusion = clamp(0.95 - shadow_factor * 0.05, 0.85, 1.0);

    let dzdx = (height(uv + vec2<f32>(NORMAL_EPSILON, 0.0)) - height(uv - vec2<f32>(NORMAL_EPSILON, 0.0)))
        / (2.0 * NORMAL_EPSILON);
    let dzdy = (height(uv + vec2<f32>(0.0, NORMAL_EPSILON)) - height(uv - vec2<f32>(0.0, NORMAL_EPSILON)))
        / (2.0 * NORMAL_EPSILON);
    s.normal = normalize(vec3<f32>(-dzdx, -dzdy, 1.0));

    s.specular_tint = params.specular_tint;
    s.specular = params.specular;

    s.specular_transmission = params.specular_transmission;
    s.thickness = params.thickness;
    s.diffuse_transmission = params.diffuse_transmission;

    let wave_angle = uv.x * params.wave_scale * 0.5;
    s.anisotropy_direction = vec2<f32>(sin(wave_angle), cos(wave_angle));
    s.anisotropy_strength = clamp(
        (perlin_noise(uv, params.wave_scale) + 1.0) * 0.5 * params.anisotropy_strength,
        0.0,
        1.0,
    );

    s.depth = clamp((perlin_noise(uv, params.wave_scale) + 1.0) * 0.5 * params.depth_scale, 0.0, 1.0);

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
