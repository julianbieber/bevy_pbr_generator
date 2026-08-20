#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<uniform> channel_mask: vec4<f32>;
@group(1) @binding(1) var<uniform> options: vec4<f32>;
@group(1) @binding(2) var map_texture: texture_2d<f32>;
@group(1) @binding(3) var map_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(map_texture, map_sampler, in.uv) * channel_mask;
    var rgb = texel.rgb;
    if options.x > 0.5 {
        rgb = vec3<f32>(texel.r + texel.g + texel.b + texel.a);
    }
    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
