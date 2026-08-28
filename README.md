# Bevy PBR Texture Generator

An interactive editor in which each PBR map is produced by a WGSL compute
shader, materials hot-reload from disk, and the result is previewed live on a lit
3D mesh.

TODO(jb-doc): prose.

## Running

```bash
just run                    # opens the editor
just new-material slate     # scaffolds assets/materials/slate.wgsl
just check                  # build + clippy, the gate a change has to pass
just test
```

Edit a file under `assets/materials/` while the editor is open: it recompiles and
re-renders without a restart. A shader that fails to compile leaves the previous
maps on screen and puts the error in the status bar.

## Writing a material

A material is one WGSL file in `assets/materials/`. It declares its parameters,
fills in a `Surface`, and ends with the entry point that writes it.

```wgsl
// @material Slate

#import pbr_gen::maps::{Surface, default_surface, write_surface, globals}
#import pbr_gen::noise::{perlin_noise, fbm_perlin}

struct Params {
    // @group Colour
    tint:  vec3<f32>, // @ui color srgb(0.4, 0.4, 0.45)
    // @group Surface
    scale: f32,       // @ui 8.0 [0.1, 64.0]
}
@group(1) @binding(0) var<uniform> params: Params;

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();
    s.base_color = params.tint * (perlin_noise(uv, params.scale) + 1.0) * 0.5;
    return s;
}

@compute @workgroup_size(8, 8, 1)
fn generate(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= globals.resolution.x || id.y >= globals.resolution.y { return; }
    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(globals.resolution);
    write_surface(id.xy, surface(uv));
}
```

The trailing entry point is unavoidable: WGSL has no forward declaration, so an
entry point living in the imported library could not call a `surface()` defined
in the importer. `just new-material <name>` scaffolds it.

Files whose name begins with `_` are not listed in the editor, which is what
keeps `_template.wgsl` out of the material list.

### `@ui` annotations

One annotation per field, on the same line. A field without one is a parse
error, reported in the status bar; the previously parsed parameters stay in use.

| Form | Field types | Widget |
|---|---|---|
| `@ui <default> [<min>, <max>]` | `f32`, `i32`, `u32` | slider |
| `@ui <default> [<min>, <max>] step <s>` | `f32`, `i32`, `u32` | slider with step |
| `@ui (<x>, <y>) [<min>, <max>]` | `vec2<f32>` | two sliders |
| `@ui color srgb(<r>, <g>, <b>)` | `vec3<f32>`, `vec4<f32>` | colour sliders |
| `@ui vec (<..>) [<min>, <max>]` | `vec2`/`vec3`/`vec4` | N sliders |
| `@ui toggle <true\|false>` | `u32`, `i32` | toggle |
| `@ui hidden` | any | none; the default is used |
| `@ui "Label" ...` | any | overrides the displayed name |
| `// @group <Name>` on its own line | — | starts a section |

A `color` annotation is written in sRGB and stored linear: the whole pipeline is
linear, and the transfer function is applied once, at PNG encode time.

## The map set

Ten maps, all allocated as `Rgba16Float` with
`STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC`. The same allocation is both the
compute target and the texture `StandardMaterial` samples, so the preview and the
export agree up to 8-bit quantisation.

| Map | Channels | Colour space on export |
|---|---|---|
| `base_color` | RGB = base colour, A = opacity | sRGB |
| `normal` | RGB = encoded normal | linear |
| `orm` | R = occlusion, G = roughness, B = metallic | linear |
| `emissive` | RGB = emissive | sRGB |
| `transmission` | R = specular transmission, G = thickness, A = diffuse transmission | linear |
| `specular` | RGB = specular tint, A = specular | sRGB tint, linear A |
| `clearcoat` | R = clearcoat, G = clearcoat roughness | linear |
| `clearcoat_normal` | RGB = encoded normal | linear |
| `anisotropy` | RG = encoded direction, B = strength | linear |
| `depth` | R = parallax depth | linear |

`Surface` carries unencoded values; the normal remap, the anisotropy direction
encoding and the ORM interleave all live in `write_surface`. The layout is
derived in [BEVY_STANDARD_MATERIAL_TEXTURE_PACKING.md](BEVY_STANDARD_MATERIAL_TEXTURE_PACKING.md).

## Export

The export button generates a second map set at the export resolution, reads it
back, and writes `<output>/<material>/<map>.png`. Preview and export resolutions
are independent, so the preview can stay cheap while parameters are dialled in.

## Layout

| Path | Role |
|---|---|
| `assets/shaders/lib/pbr_maps.wgsl` | `Surface` and channel packing; group-0 bindings |
| `assets/shaders/lib/noise.wgsl` | the noise primitives materials sample |
| `assets/shaders/map_view.wgsl` | channel masking for the map thumbnails |
| `assets/materials/*.wgsl` | the hot-reloaded unit |
| `src/material/params.rs` | the `@ui` grammar and std140 layout |
| `src/material/catalog.rs` | what is on disk and which of it is being edited |
| `src/gpu/pipeline.rs` | per-material compute pipeline and bind groups |
| `src/gpu/dispatch.rs` | generation-gated dispatch, pipeline status |
| `src/gpu/export.rs` | readback and PNG encode |
| `src/app/` | camera, preview mesh, sun |
| `src/ui/` | the editor shell and the data-driven parameter panel |

## License

MIT. See [LICENSE](LICENSE).
