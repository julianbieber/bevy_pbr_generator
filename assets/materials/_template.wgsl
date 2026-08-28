// @material Template
// @author

#import pbr_gen::maps::{Surface, default_surface, write_surface, globals}
#import pbr_gen::noise::{perlin_noise, simplex_noise, worley_noise, value_noise, fbm_perlin}

struct Params {
    // @group Colour
    tint: vec3<f32>, // @ui color srgb(0.5, 0.5, 0.5)
    // @group Surface
    scale: f32,      // @ui 8.0 [0.1, 64.0]
    roughness: f32,  // @ui 0.5 [0.0, 1.0]
}
@group(1) @binding(0) var<uniform> params: Params;

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();
    let n = (perlin_noise(uv, params.scale) + 1.0) * 0.5;
    s.base_color = params.tint * n;
    s.roughness = params.roughness;
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
