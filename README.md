# Bevy PBR Texture Generator

<!-- TODO(jb-doc): one-paragraph description of what this tool is and who it is for -->

## Running

```bash
just run
```

## Interface

```
┌──────────────────────────────────────────────────────────────────────────┐
│ mesh: Cube Sphere Plane Quad │ 3D │ ☐ Animate ☐ Turntable │ Preview 512  │
│ Export 1024 │ Export PNGs                                                │
├───────────────┬────────────────────────────────────┬─────────────────────┤
│ MATERIALS     │                                    │ MAPS                │
│ PARAMETERS    │          3D preview                │  10 thumbnails      │
│               │   orbit / zoom / pan               │ CHANNEL RGB R G B A │
│               │   procedural sky + IBL             │ SUN  elevation      │
│               │                                    │      azimuth        │
│               │                                    │ SEED Randomise      │
├───────────────┴────────────────────────────────────┴─────────────────────┤
│ <file> - <compile status> - <size> - 10 maps - <view> - regenerated <t>   │
└──────────────────────────────────────────────────────────────────────────┘
```

| Input | Action |
|---|---|
| Left drag in viewport | Orbit |
| Middle drag in viewport | Pan |
| Wheel in viewport | Zoom |
| Wheel over left panel | Scroll parameters |
| Click a map thumbnail | Toggle full-screen view of that map |
| `RGB` / `R` / `G` / `B` / `A` | Channel isolation, thumbnails and full view |

Preview resolution and export resolution are independent. Editing any
`assets/materials/*.wgsl` or `assets/shaders/lib/*.wgsl` regenerates the maps
without restarting; compile errors appear in the status bar and the previous
textures stay on screen.

## Generated Maps

| File | Channels | Colour space | Bevy `StandardMaterial` fields |
|---|---|---|---|
| `base_color.png` | RGB = base colour, A = opacity | sRGB | `base_color_texture` |
| `normal.png` | RGB = tangent-space normal | Linear | `normal_map_texture` |
| `orm.png` | R = occlusion, G = roughness, B = metallic | Linear | `occlusion_texture`, `metallic_roughness_texture` |
| `emissive.png` | RGB = emissive | sRGB | `emissive_texture` |
| `transmission.png` | R = specular transmission, G = thickness, A = diffuse transmission | Linear | `specular_transmission_texture`, `thickness_texture`, `diffuse_transmission_texture` |
| `specular.png` | RGB = specular tint, A = specular | sRGB RGB, linear A | `specular_tint_texture`, `specular_texture` |
| `clearcoat.png` | R = clearcoat, G = clearcoat roughness | Linear | `clearcoat_texture`, `clearcoat_roughness_texture` |
| `clearcoat_normal.png` | RGB = clearcoat normal | Linear | `clearcoat_normal_texture` |
| `anisotropy.png` | RG = direction, B = strength | Linear | `anisotropy_texture` |
| `depth.png` | R = parallax depth | Linear | `depth_map` |

Exports land in `<output dir>/<material>/`.

## Writing a Material

```bash
just new-material my_material
```

A material is one WGSL file in `assets/materials/`. It declares its parameters,
implements `surface(uv) -> Surface`, and ends with the compute entry point.

```wgsl
// @material My Material

#import pbr_gen::maps::{Surface, default_surface, write_surface, globals, uv_of, in_bounds, normal_from_slope, normal_epsilon}
#import pbr_gen::noise::{set_noise_seed, perlin_noise, fbm_perlin, to_unit}

struct Params {
    // @group Colour
    tint: vec3<f32>,   // @ui "Tint" color srgb(0.6, 0.6, 0.6)

    // @group Shape
    scale: f32,        // @ui "Scale" 8.0 [1.0, 64.0]
    octaves: i32,      // @ui "Octaves" 4 [1, 8]
}
@group(1) @binding(0) var<uniform> params: Params;

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();
    s.base_color = params.tint * to_unit(fbm_perlin(uv, params.scale, params.octaves, 2.0, 0.5));
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
```

Files whose name starts with `_` are skipped by the material scanner.

### `@ui` Annotations

One annotation per field, on the same line. Fields without one get a `0.0..1.0`
slider.

| Form | Field types | Widget |
|---|---|---|
| `@ui <default> [<min>, <max>]` | `f32`, `i32`, `u32` | Slider |
| `@ui <default> [<min>, <max>] step <s>` | `f32`, `i32`, `u32` | Slider with step |
| `@ui (<x>, <y>) [<min>, <max>]` | `vec2<f32>` | One slider per component |
| `@ui vec (<..>) [<min>, <max>]` | `vec2`/`vec3`/`vec4` | One slider per component |
| `@ui color srgb(<r>, <g>, <b>)` | `vec3<f32>`, `vec4<f32>` | RGB colour sliders |
| `@ui color linear(<r>, <g>, <b>)` | `vec3<f32>`, `vec4<f32>` | RGB colour sliders |
| `@ui toggle <true\|false>` | `u32`, `i32` | Toggle switch |
| `@ui hidden` | any | None; default is used |
| `@ui "Label" ...` | any | Overrides the displayed name |
| `// @group <Name>` on its own line | — | Starts a panel section |

Supported field types are `f32`, `i32`, `u32`, `vec2<f32>`, `vec3<f32>` and
`vec4<f32>`. `srgb(...)` values are converted to linear before upload. The struct
must fit in 1024 bytes.

File-level annotations:

| Form | Effect |
|---|---|
| `// @material <Name>` | Display name in the material list |

### `Surface`

`default_surface()` returns neutral values. `write_surface` performs all channel
packing and encoding.

| Field | Type |
|---|---|
| `base_color` | `vec3<f32>` |
| `opacity` | `f32` |
| `normal` | `vec3<f32>` |
| `occlusion` | `f32` |
| `roughness` | `f32` |
| `metallic` | `f32` |
| `emissive` | `vec3<f32>` |
| `specular` | `f32` |
| `specular_tint` | `vec3<f32>` |
| `diffuse_transmission` | `f32` |
| `specular_transmission` | `f32` |
| `thickness` | `f32` |
| `clearcoat` | `f32` |
| `clearcoat_roughness` | `f32` |
| `clearcoat_normal` | `vec3<f32>` |
| `anisotropy_direction` | `vec2<f32>` |
| `anisotropy_strength` | `f32` |
| `depth` | `f32` |

### Globals

| Field | Type |
|---|---|
| `globals.resolution` | `vec2<u32>` |
| `globals.time` | `f32` |
| `globals.seed` | `f32` |

### Noise Library

`pbr_gen::noise`. Lattice-based functions treat `scale` as their tiling period.
Output is seamless when `scale` is a whole number.

| Function | Returns |
|---|---|
| `set_noise_seed(seed: f32)` | — |
| `white_noise(uv) -> f32` | `[-1, 1]` |
| `value_noise(uv, scale) -> f32` | `[-1, 1]` |
| `perlin_noise(uv, scale) -> f32` | `[-1, 1]` |
| `simplex_noise(uv, scale) -> f32` | `[-1, 1]` |
| `worley_noise(uv, scale) -> f32` | `[0, ~1.4]` |
| `worley_edges(uv, scale) -> f32` | `[0, ~1.4]` |
| `fbm_perlin(uv, scale, octaves, lacunarity, persistence) -> f32` | `[-1, 1]` |
| `fbm_value(...)` / `fbm_simplex(...)` / `fbm_worley(...)` | `[-1, 1]` |
| `ridged_perlin(...)` | `[0, 1]` |
| `turbulence(...)` | `[0, 1]` |
| `domain_warp(uv, scale, strength) -> vec2<f32>` | warped UV |
| `to_unit(n: f32) -> f32` | maps `[-1, 1]` to `[0, 1]` |

## Using the Output in Bevy

```rust
StandardMaterial {
    base_color_texture: Some(asset_server.load("output/rocky/base_color.png")),
    normal_map_texture: Some(asset_server.load_with_settings(
        "output/rocky/normal.png",
        |s: &mut ImageLoaderSettings| s.is_srgb = false,
    )),
    occlusion_texture: Some(asset_server.load_with_settings(
        "output/rocky/orm.png",
        |s: &mut ImageLoaderSettings| s.is_srgb = false,
    )),
    metallic_roughness_texture: Some(asset_server.load_with_settings(
        "output/rocky/orm.png",
        |s: &mut ImageLoaderSettings| s.is_srgb = false,
    )),
    emissive: LinearRgba::WHITE,
    emissive_texture: Some(asset_server.load("output/rocky/emissive.png")),
    depth_map: Some(asset_server.load_with_settings(
        "output/rocky/depth.png",
        |s: &mut ImageLoaderSettings| s.is_srgb = false,
    )),
    ..default()
}
```

Cargo features required for the texture-driven fields:
`pbr_transmission_textures`, `pbr_specular_textures`,
`pbr_multi_layer_material_textures`, `pbr_anisotropy_texture`.

Bevy multiplies each textured field by its scalar factor. A factor of zero
disables the texture.

## Layout

| Path | Contents |
|---|---|
| `assets/materials/*.wgsl` | One file per material |
| `assets/materials/_template.wgsl` | Scaffold used by `just new-material` |
| `assets/shaders/lib/pbr_maps.wgsl` | `Surface`, bindings, channel packing |
| `assets/shaders/lib/noise.wgsl` | Noise library |
| `assets/shaders/map_view.wgsl` | Map inspector UI material |
| `src/material/params.rs` | `@ui` parser and std140 layout |
| `src/gpu/` | Compute pipeline, dispatch, PNG export |
| `src/preview.rs` | Camera, meshes, lighting |
| `src/ui/` | Feathers UI |

## License

MIT License - see [LICENSE](LICENSE) for details.
