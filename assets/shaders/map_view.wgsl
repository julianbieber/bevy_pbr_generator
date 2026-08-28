// Draws one generated map into a UI node, masked to a single channel.

#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<uniform> settings: vec4<f32>;
@group(1) @binding(1) var map_texture: texture_2d<f32>;
@group(1) @binding(2) var map_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(map_texture, map_sampler, in.uv);
    let channel = i32(settings.x);

    var rgb: vec3<f32>;
    if channel == 1 {
        rgb = vec3<f32>(texel.r);
    } else if channel == 2 {
        rgb = vec3<f32>(texel.g);
    } else if channel == 3 {
        rgb = vec3<f32>(texel.b);
    } else if channel == 4 {
        rgb = vec3<f32>(texel.a);
    } else {
        rgb = texel.rgb;
    }

    return vec4<f32>(rgb, 1.0);
}
