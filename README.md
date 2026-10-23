# Bevy PBR Texture Generator

An interactive editor in which each PBR map is produced by a WGSL compute
shader, materials hot-reload from disk, and the result is previewed live on a lit
3D mesh.

## Using it from your project

Materials live in the project that needs the textures, not here. Install the
two binaries into that project with [mise](https://mise.jdx.dev). Every push to
`main` that passes CI is tagged `v0.<n>`, counting up from `v0.1`; pin one of
those tags by running this in the project:

```bash
mise use "cargo:https://github.com/julianbieber/bevy_pbr_generator@tag:v0.x"
```

mise builds it from source with `cargo install`, so bevy's native build
dependencies have to be present. It records the pin in the project's
`mise.toml` and puts `bevy_pbr_generator` and `bevy_pbr_combine` on the `PATH` inside
the project (with `mise activate` in your shell; otherwise prefix the commands
with `mise exec --`). Nothing is added to the project's `Cargo.toml`: the game depends on
the exported PNGs, not on this crate.

Then, from the project:

```bash
bevy_pbr_generator init materials           # make materials/ a material project
bevy_pbr_generator new slate materials      # write materials/shaders/slate.wesl
bevy_pbr_generator materials                # open the editor on it
bevy_pbr_combine output/water output/rocky  # pack exports into output/combined
```

Every directory argument defaults to the current one, so inside `materials/`
these are `bevy_pbr_generator init`, `bevy_pbr_generator new slate` and
`bevy_pbr_generator`. The editor writes exports to `output/` under the
directory it was started from.

Edit a material, or any module it imports, while the editor is open: it
recompiles and re-renders without a restart. A shader that fails to compile
leaves the previous maps on screen and puts the error in the status bar. A
material file added while the editor is open is listed after a restart.

### The material project

A material project is any directory with a `wesl.toml`. `init` writes one when
there is none, and copies the WESL package `bevy_pbr_generator`, which materials
import, into a `bevy_pbr_generator/` directory beside it:

```toml
edition = "2026_pre"
root = "shaders"

[dependencies]
bevy_pbr_generator = { path = "bevy_pbr_generator" }
```

The editor reads the same `wesl.toml` with the same rules the language server
(wgsl-analyzer) applies — `root`, `include` and `exclude` — so `package::`
means the same thing in both. A few limits follow from that:

- The editor runs the package it was built with; the language server reads the
  copy the `path` names. After moving the pin to another tag, run
  `bevy_pbr_generator init` again: it refreshes the copy and leaves an existing `wesl.toml` alone. The
  editor warns at startup when the two differ.
- `bevy_pbr_generator` is the only dependency the editor provides, under that
  name. A material importing any other package will not compile, and the editor
  warns at startup about every other dependency `wesl.toml` lists.
- A project's own helper modules must be `.wesl`. File and directory names must be
  WESL identifiers; the scan skips anything else and logs it.
- The editor watches the whole package root, so keep it to shaders.

`example/` is such a project, holding the template and two example materials.
Its dependency points at this repo's `src/shaders` instead of a copy.


## Writing a material

A material is one WESL file anywhere under the project's package root, marked
by a `// @material <name>` line in its leading comment block. It declares its
parameters and fills in a `Surface`.

```wesl
// @material Slate

import bevy_pbr_generator::pbr_maps::{Surface, default_surface};
import bevy_pbr_generator::noise;

struct Params {
    // @group Colour
    tint:  vec3<f32>, // @ui color srgb(0.4, 0.4, 0.45)
    // @group Surface
    scale: f32,       // @ui 8.0 [0.1, 64.0]
}
@group(1) @binding(0) var<uniform> params: Params;

fn surface(uv: vec2<f32>) -> Surface {
    var s = default_surface();
    s.base_color = params.tint * (noise::perlin_noise(uv, params.scale) + 1.0) * 0.5;
    return s;
}
```

A material declares no entry point. The editor writes one for it, which imports
its `surface()` and stores the result into the maps. Importing a module, as with
`noise` above, and calling into it by path saves listing every function in the
import. The project's own modules are imported through `package::` or `super::`.

A file without the header is a helper module and is not listed. Neither is a
file whose name begins with `_`, which is what keeps `_template.wesl` out of the
material list.

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

### Using an export in a game

**Copy code** puts Rust on the clipboard that builds the active material in a
bevy project: one `material_<name>()` function returning a `bsn!` scene that
gives its entity a `MeshMaterial3d<StandardMaterial>`, with the same texture
slots and factors as the preview. Copy the exported `<material>/` folder
directly under the project's `assets/`, paste the code, and spawn it with the
mesh:

```rust
commands.spawn_scene(material_rocky()).insert(Mesh3d(mesh));
```

The code binds all ten maps, so the project needs bevy's
`pbr_transmission_textures`, `pbr_specular_textures`, `pbr_anisotropy_texture`
and `pbr_multi_layer_material_textures` features, or the slots it does not use
removed. The mesh needs tangents. On X11 the clipboard is served by the
editor, so paste before closing it.

## Combining materials

`bevy_pbr_combine [-o <dir>] <material folder>...` packs several exported materials
into one image per map, so a single mesh can give each region its own material
by UV. The output defaults to `output/combined/`, relative to the current
directory, and holds the same ten
`<map>.png` names as an export, plus `atlas.json`.

Materials are placed left to right in the order given, each in a square cell the
size of the largest input map; smaller maps are resized up with Lanczos3. If the
row would be wider than 8192px it becomes a grid of as many columns as fit, and
a combine that would need more than 8192px of height is refused. Pixels are
copied without colour-space conversion, so each combined map keeps the encoding
its export had.

`atlas.json` names each material after its folder:

```json
{
  "columns": 2, "rows": 1, "cell_size": 2048, "width": 4096, "height": 2048,
  "materials": [
    { "name": "water", "u_min": 0, "u_max": 0.5, "v_min": 0, "v_max": 1 },
    { "name": "rocky", "u_min": 0.5, "u_max": 1, "v_min": 0, "v_max": 1 }
  ]
}
```

`v = 0` is the top edge. The same folders in the same order always produce the
same regions; adding a material, or changing the largest resolution, moves them.

Cells have no padding. Filtering and mipmaps blend neighbouring materials at a
shared edge, and empty grid cells are transparent black, so keeping UVs clear of
region borders is the consumer's job.

A folder missing a map, a map that is not square 8-bit RGBA, two folders with
the same name, or `-o` pointing at one of the inputs stops the combine before
anything is written; a failure while writing leaves the output directory as it
was.

## Developing it

The `Justfile` is for working on this repo. Its recipes build from source and
take the same arguments as the installed binaries, resolved against the
directory `just` is called from:

```bash
just run                                 # opens the editor on example/
just run ../my_game/materials            # opens it on another material project
just new-material slate                  # writes example/shaders/slate.wesl
just combine output/water output/rocky   # runs bevy_pbr_combine
just check                               # build + clippy, the gate a change has to pass
just test
```

Edits to this package's own modules, `src/shaders`, hot-reload only under
`just run`, which is what tells bevy where the source is.

## Layout

| Path | Role |
|---|---|
| `src/shaders/pbr_maps.wesl` | `bevy_pbr_generator::pbr_maps`: `Surface` and channel packing; group-0 bindings |
| `src/shaders/noise.wesl` | `bevy_pbr_generator::noise`: the noise primitives materials sample |
| `src/shaders/mod.rs` | embedding of the package, and the entry module written for each material |
| `wesl.toml` | the package's root, for the language server of `example/` |
| `src/scaffold.rs` | `init` and `new`: the package copy and the material template |
| `example/` | a material project: the template and the example materials |
| `src/ui/map_view.wesl` | channel masking for the map thumbnails |
| `src/material/project.rs` | the material project the editor is opened on |
| `src/material/params.rs` | the `@ui` grammar and std140 layout |
| `src/material/catalog.rs` | what is on disk and which of it is being edited |
| `src/gpu/pipeline.rs` | per-material compute pipeline and bind groups |
| `src/gpu/dispatch.rs` | generation-gated dispatch, pipeline status |
| `src/gpu/export.rs` | readback and PNG encode |
| `src/codegen.rs` | the Rust **Copy code** puts on the clipboard |
| `src/bin/bevy_pbr_combine/` | the `bevy_pbr_combine` CLI: layout, compose, manifest |
| `src/app/` | camera, preview mesh, sun |
| `src/ui/` | the editor shell and the data-driven parameter panel |

## License

MIT. See [LICENSE](LICENSE).
