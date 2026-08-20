// @material Template

#import pbr_gen::maps::{Surface, default_surface, write_surface, globals, uv_of, in_bounds, normal_from_slope, normal_epsilon}
#import pbr_gen::noise::{set_noise_seed, perlin_noise, fbm_perlin, to_unit}

struct Params {
    // @group Colour
    tint: vec3<f32>,        // @ui "Tint" color srgb(0.6, 0.6, 0.6)

    // @group Shape
    scale: f32,             // @ui "Scale" 8.0 [1.0, 64.0]
    octaves: i32,           // @ui "Octaves" 4 [1, 8]
    normal_strength: f32,   // @ui "Normal strength" 6.0 [0.0, 48.0]

    // @group Surface
    roughness: f32,         // @ui "Roughness" 0.6 [0.0, 1.0]
    metallic: f32,          // @ui "Metallic" 0.0 [0.0, 1.0]
}

@group(1) @binding(0) var<uniform> params: Params;

fn height(uv: vec2<f32>) -> f32 {
    return fbm_perlin(uv, params.scale, params.octaves, 2.0, 0.5);
}

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();

    let n = to_unit(height(uv));
    s.base_color = params.tint * (0.6 + 0.4 * n);
    s.roughness = params.roughness;
    s.metallic = params.metallic;

    let e = normal_epsilon();
    let dx = height(uv + vec2<f32>(e, 0.0)) - height(uv - vec2<f32>(e, 0.0));
    let dy = height(uv + vec2<f32>(0.0, e)) - height(uv - vec2<f32>(0.0, e));
    s.normal = normal_from_slope(dx, dy, params.normal_strength);

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
