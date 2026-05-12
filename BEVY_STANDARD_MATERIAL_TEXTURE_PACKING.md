# Bevy StandardMaterial: Optimal Texture Packing

## Channel Usage per Attribute

| Attribute | Bevy Field | Channel |
|-----------|------------|---------|
| Base Color | `base_color_texture` | RGB |
| Opacity | `base_color_texture` | A |
| Normal | `normal_map_texture` | RGB |
| Occlusion | `occlusion_texture` | R |
| Roughness | `metallic_roughness_texture` | G |
| Metallic | `metallic_roughness_texture` | B |
| Emissive | `emissive_texture` | RGB |
| Specular | `specular_texture` | A |
| Specular Tint | `specular_tint_texture` | RGB |
| Diffuse Transmission | `diffuse_transmission_texture` | A |
| Specular Transmission | `specular_transmission_texture` | R |
| Thickness | `thickness_texture` | G |
| Clearcoat | `clearcoat_texture` | R |
| Clearcoat Roughness | `clearcoat_roughness_texture` | G |
| Clearcoat Normal | `clearcoat_normal_texture` | RGB |
| Anisotropy Dir | `anisotropy_texture` | RG |
| Anisotropy Strength | `anisotropy_texture` | B |
| Parallax Depth | `depth_map` | R |

---

## Optimal Packing

| Textures | Packing | Bevy Fields |
|----------|---------|-------------|
| **3** | Base(RGB+A), Normal(RGB), **ORM(R+G+B)** | `base_color_texture`, `normal_map_texture`, `occlusion_texture` + `metallic_roughness_texture` |
| **4** | + Emissive(RGB) | + `emissive_texture` |
| **5** | + Transmission(R+G+A) | + `specular_transmission_texture` + `thickness_texture` + `diffuse_transmission_texture` |
| **5** | + Specular(RGB+A) | + `specular_tint_texture` + `specular_texture` |
| **6** | + Clearcoat(R+G) | + `clearcoat_texture` + `clearcoat_roughness_texture` |
| **7** | + Clearcoat Normal(RGB) | + `clearcoat_normal_texture` |
| **8** | + Anisotropy(R+G+B) | + `anisotropy_texture` |

> **ORM** = Occlusion(R), Roughness(G), Metallic(B) in a single texture
