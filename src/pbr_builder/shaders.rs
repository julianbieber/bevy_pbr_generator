//! WGSL Shader Library for PBR Materials
use bevy::render::render_resource::ShaderRef;
use std::collections::HashMap;
use super::PbrTextureType;

/// Shader function library for PBR rendering
pub struct PbrShaderLibrary {
    pub functions: HashMap<String, String>,
    pub modules: HashMap<PbrTextureType, String>,
    pub combined_shader: String,
}

impl Default for PbrShaderLibrary {
    fn default() -> Self {
        let mut library = Self {
            functions: HashMap::new(),
            modules: HashMap::new(),
            combined_shader: String::new(),
        };
        library.initialize_functions();
        library.initialize_modules();
        library.initialize_combined_shader();
        library
    }
}

impl PbrShaderLibrary {
    fn initialize_functions(&mut self) {
        self.functions.insert("pi".to_string(), PI.to_string());
        self.functions.insert("saturate".to_string(), SATURATE.to_string());
        self.functions.insert("lerp".to_string(), LERP.to_string());
        self.functions.insert("fresnel_schlick".to_string(), FRESNEL_SCHLICK.to_string());
        self.functions.insert("fresnel_schlick_roughness".to_string(), FRESNEL_SCHLICK_ROUGHNESS.to_string());
        self.functions.insert("distribution_ggx".to_string(), DISTRIBUTION_GGX.to_string());
        self.functions.insert("geometry_schlick_ggx".to_string(), GEOMETRY_SCHLICK_GGX.to_string());
        self.functions.insert("geometry_smith".to_string(), GEOMETRY_SMITH.to_string());
        self.functions.insert("pbr_brdf".to_string(), PBR_BRDF.to_string());
    }

    fn initialize_modules(&mut self) {
        for texture_type in PbrTextureType::all() {
            let module = self.create_texture_module(*texture_type);
            self.modules.insert(*texture_type, module);
        }
    }

    fn initialize_combined_shader(&mut self) {
        self.combined_shader = self.generate_combined_shader();
    }

    pub fn create_texture_module(&self, texture_type: PbrTextureType) -> String {
        match texture_type {
            PbrTextureType::BaseColor => BASE_COLOR_MODULE.to_string(),
            PbrTextureType::MetallicRoughness => METALLIC_ROUGHNESS_MODULE.to_string(),
            PbrTextureType::Normal => NORMAL_MODULE.to_string(),
            PbrTextureType::Occlusion => OCCLUSION_MODULE.to_string(),
            PbrTextureType::Emissive => EMISSIVE_MODULE.to_string(),
            PbrTextureType::Height => HEIGHT_MODULE.to_string(),
        }
    }

    pub fn generate_combined_shader(&self) -> String {
        let mut shader = String::new();
        for (name, function) in &self.functions {
            if !name.starts_with("sample_") {
                shader.push_str(&format!("// Function: {}\n{}\n\n", name, function));
            }
        }
        for (texture_type, module) in &self.modules {
            shader.push_str(&format!("// Module: {:?}\n{}\n\n", texture_type, module));
        }
        shader.push_str(&MAIN_PBR_SHADER);
        shader
    }

    pub fn get_function(&self, name: &str) -> Option<&str> {
        self.functions.get(name).map(|s| s.as_str())
    }

    pub fn get_module(&self, texture_type: PbrTextureType) -> Option<&str> {
        self.modules.get(&texture_type).map(|s| s.as_str())
    }

    pub fn get_combined_shader(&self) -> &str {
        &self.combined_shader
    }

    pub fn create_shader_ref(&self, shader: &str) -> ShaderRef {
        ShaderRef::from_wgsl(shader, None)
    }

    pub fn create_texture_shader_ref(&self, texture_type: PbrTextureType) -> ShaderRef {
        let module = self.modules.get(&texture_type).expect("Texture type module not found");
        ShaderRef::from_wgsl(module, None)
    }

    pub fn create_combined_shader_ref(&self) -> ShaderRef {
        ShaderRef::from_wgsl(&self.combined_shader, None)
    }
}

const PI: &str = r#"
const PI: f32 = 3.14159265359;
const PI_2: f32 = PI * 2.0;
const PI_4: f32 = PI * 4.0;
const INV_PI: f32 = 1.0 / PI;
const INV_PI_2: f32 = 1.0 / (PI * 2.0);
"#;

const SATURATE: &str = r#"
fn saturate(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}
fn saturate_vec3(x: vec3<f32>) -> vec3<f32> {
    return clamp(x, vec3<f32>(0.0), vec3<f32>(1.0));
}
"#;

const LERP: &str = r#"
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + (b - a) * t;
}
fn lerp_vec3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a + (b - a) * t;
}
fn lerp_vec4(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    return a + (b - a) * t;
}
"#;

const FRESNEL_SCHLICK: &str = r#"
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    let one_minus_cos_theta = 1.0 - saturate(cos_theta);
    let one_minus_cos_theta_pow5 = one_minus_cos_theta * one_minus_cos_theta * one_minus_cos_theta * one_minus_cos_theta * one_minus_cos_theta;
    return f0 + (1.0 - f0) * one_minus_cos_theta_pow5;
}
"#;

const DISTRIBUTION_GGX: &str = r#"
fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h = saturate(dot(n, h));
    let n_dot_h2 = n_dot_h * n_dot_h;
    let nom = a2;
    let denom = (n_dot_h2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    return nom / denom;
}
"#;

const GEOMETRY_SCHLICK_GGX: &str = r#"
fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let nom = n_dot_v;
    let denom = n_dot_v * (1.0 - k) + k;
    return nom / denom;
}
"#;

const GEOMETRY_SMITH: &str = r#"
fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let n_dot_v = saturate(dot(n, v));
    let n_dot_l = saturate(dot(n, l));
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}
"#;

const PBR_BRDF: &str = r#"
fn pbr_brdf(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, h: vec3<f32>, base_color: vec3<f32>, metallic: f32, roughness: f32, f0: vec3<f32>, occlusion: f32) -> vec3<f32> {
    let cos_theta = saturate(dot(h, v));
    let f = fresnel_schlick(cos_theta, f0);
    let ndf = distribution_ggx(n, h, roughness);
    let g = geometry_smith(n, v, l, roughness);
    let numerator = ndf * g * f;
    let denominator = 4.0 * saturate(dot(n, v)) * saturate(dot(n, l)) + 0.0001;
    let specular = numerator / denominator;
    let k_s = f;
    let k_d = vec3<f32>(1.0) - k_s;
    let diffuse = k_d * (1.0 - metallic) * base_color * INV_PI;
    let n_dot_l = saturate(dot(n, l));
    return (diffuse + specular) * n_dot_l * occlusion + (1.0 - metallic) * base_color * occlusion * 0.03;
}
"#;

const BASE_COLOR_MODULE: &str = r#"
struct BaseColorInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    color_factor: vec4<f32>,
};
struct BaseColorOutput {
    color: vec4<f32>,
    alpha: f32,
};
fn process_base_color(input: BaseColorInput) -> BaseColorOutput {
    let sampled_color = textureSample(input.texture, input.sampler, input.uv);
    let linear_color = vec3<f32>(sampled_color.r * sampled_color.r, sampled_color.g * sampled_color.g, sampled_color.b * sampled_color.b);
    let final_color = vec4<f32>(linear_color * input.color_factor.rgb, sampled_color.a * input.color_factor.a);
    return BaseColorOutput { color: final_color, alpha: final_color.a };
}
"#;

const METALLIC_ROUGHNESS_MODULE: &str = r#"
struct MetallicRoughnessInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
};
struct MetallicRoughnessOutput {
    metallic: f32,
    roughness: f32,
};
fn process_metallic_roughness(input: MetallicRoughnessInput) -> MetallicRoughnessOutput {
    let sampled_color = textureSample(input.texture, input.sampler, input.uv);
    let metallic = clamp(sampled_color.b * input.metallic_factor, 0.0, 1.0);
    let roughness = clamp(sampled_color.g * input.roughness_factor, 0.0, 1.0);
    return MetallicRoughnessOutput { metallic: metallic, roughness: roughness };
}
"#;

const NORMAL_MODULE: &str = r#"
struct NormalInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    normal_scale: f32,
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
    normal: vec3<f32>,
};
struct NormalOutput {
    normal: vec3<f32>,
};
fn process_normal(input: NormalInput) -> NormalOutput {
    let normal_xy = textureSample(input.texture, input.sampler, input.uv).xy * 2.0 - 1.0;
    let normal_z = sqrt(1.0 - dot(normal_xy, normal_xy));
    let tangent_normal = vec3<f32>(normal_xy, normal_z);
    let scaled_normal = normalize(tangent_normal * input.normal_scale);
    let tbn_matrix = mat3x3<f32>(normalize(input.tangent), normalize(input.bitangent), normalize(input.normal));
    let world_normal = tbn_matrix * scaled_normal;
    return NormalOutput { normal: normalize(world_normal) };
}
"#;

const OCCLUSION_MODULE: &str = r#"
struct OcclusionInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    strength: f32,
};
struct OcclusionOutput {
    occlusion: f32,
};
fn process_occlusion(input: OcclusionInput) -> OcclusionOutput {
    let occlusion = clamp(textureSample(input.texture, input.sampler, input.uv).r * input.strength, 0.0, 1.0);
    return OcclusionOutput { occlusion: occlusion };
}
"#;

const EMISSIVE_MODULE: &str = r#"
struct EmissiveInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    emissive_factor: vec3<f32>,
};
struct EmissiveOutput {
    emissive: vec3<f32>,
};
fn process_emissive(input: EmissiveInput) -> EmissiveOutput {
    let sampled_color = textureSample(input.texture, input.sampler, input.uv);
    let linear_color = vec3<f32>(sampled_color.r * sampled_color.r, sampled_color.g * sampled_color.g, sampled_color.b * sampled_color.b);
    return EmissiveOutput { emissive: linear_color * input.emissive_factor };
}
"#;

const HEIGHT_MODULE: &str = r#"
struct HeightInput {
    texture: texture_2d<f32>,
    sampler: sampler,
    uv: vec2<f32>,
    height_scale: f32,
};
struct HeightOutput {
    height: f32,
};
fn process_height(input: HeightInput) -> HeightOutput {
    let height = textureSample(input.texture, input.sampler, input.uv).r * input.height_scale;
    return HeightOutput { height: height };
}
"#;

const MAIN_PBR_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

@group(0) @binding(0) var<uniform> camera_view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> camera_position: vec3<f32>;
@group(0) @binding(2) var<uniform> model_matrix: mat4x4<f32>;
@group(0) @binding(3) var<uniform> normal_matrix: mat3x3<f32>;

@group(1) @binding(0) var base_color_texture: texture_2d<f32>;
@group(1) @binding(1) var base_color_sampler: sampler;
@group(1) @binding(2) var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(3) var metallic_roughness_sampler: sampler;
@group(1) @binding(4) var normal_texture: texture_2d<f32>;
@group(1) @binding(5) var normal_sampler: sampler;
@group(1) @binding(6) var occlusion_texture: texture_2d<f32>;
@group(1) @binding(7) var occlusion_sampler: sampler;
@group(1) @binding(8) var emissive_texture: texture_2d<f32>;
@group(1) @binding(9) var emissive_sampler: sampler;
@group(1) @binding(10) var height_texture: texture_2d<f32>;
@group(1) @binding(11) var height_sampler: sampler;

@group(2) @binding(0) var<uniform> material_params: MaterialParams;
@group(3) @binding(0) var<uniform> light_params: LightParams;

struct MaterialParams {
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    emissive_factor: vec3<f32>,
    normal_scale: f32,
    occlusion_strength: f32,
    alpha_cutoff: f32,
    double_sided: u32,
};

struct LightParams {
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    ambient_color: vec3<f32>,
    ambient_intensity: f32,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = camera_view_proj * model_matrix * vec4<f32>(input.position, 1.0);
    output.world_position = vec3<f32>(model_matrix * vec4<f32>(input.position, 1.0));
    output.world_normal = normalize(normal_matrix * input.normal);
    output.uv = input.uv;
    output.tangent = normalize(normal_matrix * input.tangent);
    output.bitangent = normalize(normal_matrix * input.bitangent);
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = process_base_color(BaseColorInput {
        texture: base_color_texture,
        sampler: base_color_sampler,
        uv: input.uv,
        color_factor: material_params.base_color_factor,
    });
    let metallic_roughness = process_metallic_roughness(MetallicRoughnessInput {
        texture: metallic_roughness_texture,
        sampler: metallic_roughness_sampler,
        uv: input.uv,
        metallic_factor: material_params.metallic_factor,
        roughness_factor: material_params.roughness_factor,
    });
    let normal = process_normal(NormalInput {
        texture: normal_texture,
        sampler: normal_sampler,
        uv: input.uv,
        normal_scale: material_params.normal_scale,
        tangent: input.tangent,
        bitangent: input.bitangent,
        normal: input.world_normal,
    });
    let occlusion = process_occlusion(OcclusionInput {
        texture: occlusion_texture,
        sampler: occlusion_sampler,
        uv: input.uv,
        strength: material_params.occlusion_strength,
    });
    let emissive = process_emissive(EmissiveInput {
        texture: emissive_texture,
        sampler: emissive_sampler,
        uv: input.uv,
        emissive_factor: material_params.emissive_factor,
    });
    let n = normal.normal;
    let v = normalize(camera_position - input.world_position);
    let l = normalize(light_params.direction);
    let h = normalize(v + l);
    let f0 = vec3<f32>(0.04);
    let f0_mixed = mix(f0, base_color.color.rgb, metallic_roughness.metallic);
    let brdf = pbr_brdf(n, v, l, h, base_color.color.rgb, metallic_roughness.metallic, metallic_roughness.roughness, f0_mixed, occlusion.occlusion);
    let final_color = brdf + emissive.emissive;
    if (base_color.alpha < material_params.alpha_cutoff) { discard; }
    return vec4<f32>(final_color, base_color.alpha);
}
"#;
