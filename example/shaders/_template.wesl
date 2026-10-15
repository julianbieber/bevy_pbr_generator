// @material Template
// @author

import bevy_pbr_generator::pbr_maps::{Surface, default_surface};
import bevy_pbr_generator::noise;

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
    let n = (noise::perlin_noise(uv, params.scale) + 1.0) * 0.5;
    s.base_color = params.tint * n;
    s.roughness = params.roughness;
    return s;
}
