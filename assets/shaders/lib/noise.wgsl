// Deterministic 2D noise primitives sampled by the material shaders.
//
// Every function is a pure function of its arguments with no seeding parameter,
// so a given coordinate yields the same value across dispatches and across runs.

#define_import_path pbr_gen::noise

var<private> PERM: array<u32, 256> = array<u32, 256>(
    151u, 160u, 137u,  91u,  90u,  15u, 131u,  13u, 201u,  95u,  96u,  53u, 194u, 233u,   7u, 225u,
    140u,  36u, 103u,  30u,  69u, 142u,   8u,  99u,  37u, 240u,  21u,  10u,  23u, 190u,   6u, 148u,
    247u, 120u, 234u,  75u,   0u,  26u, 197u,  62u,  94u, 252u, 219u, 203u, 117u,  35u,  11u,  32u,
     57u, 177u,  33u,  88u, 237u, 149u,  56u,  87u, 174u,  20u, 125u, 136u, 171u, 168u,  68u, 175u,
     74u, 165u,  71u, 134u, 139u,  48u,  27u, 166u,  77u, 146u, 158u, 231u,  83u, 111u, 229u, 122u,
     60u, 211u, 133u, 230u, 220u, 105u,  92u,  41u,  55u,  46u, 245u,  40u, 244u, 102u, 143u,  54u,
     65u,  25u,  63u, 161u,   1u, 216u,  80u,  73u, 209u,  76u, 132u, 187u, 208u,  89u,  18u, 169u,
    200u, 196u, 135u, 130u, 116u, 188u, 159u,  86u, 164u, 100u, 109u, 198u, 173u, 186u,   3u,  64u,
     52u, 217u, 226u, 250u, 124u, 123u,   5u, 202u,  38u, 147u, 118u, 126u, 255u,  82u,  85u, 212u,
    207u, 206u,  59u, 227u,  47u,  16u,  58u,  17u, 182u, 189u,  28u,  42u, 223u, 183u, 170u, 213u,
    119u, 248u, 152u,   2u,  44u, 154u, 163u,  70u, 221u, 153u, 101u, 155u, 167u,  43u, 172u,   9u,
    129u,  22u,  39u, 253u,  19u,  98u, 108u, 110u,  79u, 113u, 224u, 232u, 178u, 185u, 112u, 104u,
    218u, 246u,  97u, 228u, 251u,  34u, 242u, 193u, 238u, 210u, 144u,  12u, 191u, 179u, 162u, 241u,
     81u,  51u, 145u, 235u, 249u,  14u, 239u, 107u,  49u, 192u, 214u,  31u, 181u, 199u, 106u, 157u,
    184u,  84u, 204u, 176u, 115u, 121u,  50u,  45u, 127u,   4u, 150u, 254u, 138u, 236u, 205u,  93u,
    222u, 114u,  67u,  29u,  24u,  72u, 243u, 141u, 128u, 195u,  78u,  66u, 215u,  61u, 156u, 180u,
);

fn integer_hash(seed: i32) -> i32 {
    var h = seed;
    h = (h ^ (h >> 16u)) * bitcast<i32>(0x85ebca6bu);
    h = (h ^ (h >> 13u)) * bitcast<i32>(0xc2b2ae35u);
    return h ^ (h >> 16u);
}

fn unit_from_hash(h: i32) -> f32 {
    return f32(bitcast<u32>(h) & 0x00ffffffu) / 16777216.0;
}

fn lattice_hash(x: i32, y: i32) -> i32 {
    return integer_hash(x * 123456791 + y * 987654321);
}

fn cell_offset(cell: vec2<f32>) -> vec2<f32> {
    let h = lattice_hash(i32(cell.x), i32(cell.y));
    let k = integer_hash(h ^ bitcast<i32>(0x27d4eb2du));
    return vec2<f32>(unit_from_hash(h), unit_from_hash(k));
}

fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn perm_at(i: u32) -> u32 {
    return PERM[i & 255u];
}

fn perm_lattice(x: i32, y: i32) -> u32 {
    var xi = bitcast<u32>(x);
    var yi = bitcast<u32>(y);
    xi = (xi + perm_at(yi + perm_at(xi))) & 255u;
    yi = (yi + PERM[xi]) & 255u;
    return perm_at(xi + PERM[yi]);
}

fn perlin_grad(x: f32, y: f32, h: u32) -> f32 {
    let g = h & 15u;
    let u = select(y, x, g < 8u);
    var v = 0.0;
    if (g < 4u) {
        v = y;
    } else if (g == 12u || g == 14u) {
        v = x;
    }
    return select(-u, u, (g & 1u) == 0u) + select(-v, v, (g & 2u) == 0u);
}

fn simplex_hash(x: i32, y: i32) -> u32 {
    var h = x * 32661 + y * 65537;
    h = (h ^ (h >> 13u)) * 69069;
    h = (h ^ (h >> 17u)) * 3571;
    return bitcast<u32>(h ^ (h >> 10u)) & 255u;
}

fn simplex_grad(h: u32) -> vec2<f32> {
    let g = h & 15u;
    let u = select(-1.0, 1.0, g < 8u);
    var v = 0.0;
    if (g < 4u) {
        v = 1.0;
    } else if (g == 12u || g == 14u) {
        v = -1.0;
    }
    return normalize(vec2<f32>(u, v));
}

// Uncorrelated value in [-1, 1] hashed from the truncated integer part of `uv`,
// so it is constant across each unit cell: callers wanting per-pixel noise must
// scale `uv` up first.
fn white_noise(uv: vec2<f32>) -> f32 {
    return unit_from_hash(lattice_hash(i32(uv.x), i32(uv.y))) * 2.0 - 1.0;
}

// `white_noise` sampled on a lattice of `scale` cells per unit of UV and
// bilinearly interpolated between them. Range [-1, 1]; the linear blend leaves
// visible creases along cell boundaries.
fn value_noise(uv: vec2<f32>, scale: f32) -> f32 {
    let scaled = uv * scale;
    let cell = floor(scaled);
    let f = scaled - cell;

    let x0 = i32(cell.x);
    let y0 = i32(cell.y);

    let n00 = unit_from_hash(lattice_hash(x0, y0)) * 2.0 - 1.0;
    let n10 = unit_from_hash(lattice_hash(x0 + 1, y0)) * 2.0 - 1.0;
    let n01 = unit_from_hash(lattice_hash(x0, y0 + 1)) * 2.0 - 1.0;
    let n11 = unit_from_hash(lattice_hash(x0 + 1, y0 + 1)) * 2.0 - 1.0;

    let ix0 = mix(n00, n10, f.x);
    let ix1 = mix(n01, n11, f.x);
    return mix(ix0, ix1, f.y);
}

// Gradient (Perlin) noise on a lattice of `scale` cells per unit of UV, roughly
// in [-1, 1] and zero at every lattice point. Smoother than `value_noise` at
// the same scale.
fn perlin_noise(uv: vec2<f32>, scale: f32) -> f32 {
    let scaled = uv * scale;
    let cell = floor(scaled);
    let f = scaled - cell;

    let x0 = i32(cell.x);
    let y0 = i32(cell.y);

    let u = fade(f.x);
    let v = fade(f.y);

    let a = perm_lattice(x0, y0);
    let b = perm_lattice(x0 + 1, y0);
    let c = perm_lattice(x0, y0 + 1);
    let d = perm_lattice(x0 + 1, y0 + 1);

    let ix0 = mix(perlin_grad(f.x, f.y, a), perlin_grad(f.x - 1.0, f.y, b), u);
    let ix1 = mix(perlin_grad(f.x, f.y - 1.0, c), perlin_grad(f.x - 1.0, f.y - 1.0, d), u);
    return mix(ix0, ix1, v);
}

// Simplex noise on a grid of `scale` cells per unit of UV, roughly in [-1, 1].
// Cheaper than `perlin_noise` at higher scales and free of its axis-aligned
// directional bias.
fn simplex_noise(uv: vec2<f32>, scale: f32) -> f32 {
    let scaled = uv * scale;

    let skew = (1.0 / 3.0) * (sqrt(2.0) + 1.0);
    let s = (scaled.x + scaled.y) * skew;
    let i = i32(floor(s));
    let j = i32(floor(scaled.y + f32(i) * (1.0 / skew)));

    let t = f32(i + j) * skew;
    let p0 = vec2<f32>(scaled.x - f32(i) + t, scaled.y - f32(j) + t);

    var step1 = vec2<i32>(0, 1);
    if (p0.x > p0.y) {
        step1 = vec2<i32>(1, 0);
    }

    let p1 = p0 - vec2<f32>(f32(step1.x), f32(step1.y)) + skew;
    let p2 = p0 - 1.0 + 2.0 * skew;

    let n0 = dot(simplex_grad(simplex_hash(i, j)), p0);
    let n1 = dot(simplex_grad(simplex_hash(i + step1.x, j + step1.y)), p1);
    let n2 = dot(simplex_grad(simplex_hash(i + 1, j + 1)), p2);

    let a0 = 0.5 - dot(p0, p0);
    let a1 = 0.5 - dot(p1, p1);
    let a2 = 0.5 - dot(p2, p2);

    var value = 0.0;
    if (a0 > 0.0) {
        let q = a0 * a0;
        value += q * q * n0;
    }
    if (a1 > 0.0) {
        let q = a1 * a1;
        value += q * q * n1;
    }
    if (a2 > 0.0) {
        let q = a2 * a2;
        value += q * q * n2;
    }

    return value * 70.0;
}

// Distances from `uv * scale` to the nearest and second-nearest feature point,
// one point per lattice cell. Component 0 is the nearest, component 1 the
// second; both are in [0, sqrt(2) * 2] and smallest at the feature points.
fn worley_distances(uv: vec2<f32>, scale: f32) -> vec2<f32> {
    let scaled = uv * scale;
    let base = floor(scaled);

    var nearest = 8.0;
    var second = 8.0;

    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            let cell = base + vec2<f32>(f32(dx), f32(dy));
            let feature = cell + cell_offset(cell);
            let d = distance(scaled, feature);
            if (d < nearest) {
                second = nearest;
                nearest = d;
            } else if (d < second) {
                second = d;
            }
        }
    }

    return vec2<f32>(nearest, second);
}

// Distance from `uv * scale` to the nearest feature point, clamped above at 1.0
// (F1 Worley noise). Range [0, 1], smallest at the feature points.
fn worley_noise(uv: vec2<f32>, scale: f32) -> f32 {
    return min(worley_distances(uv, scale).x, 1.0);
}

// Sums `octaves` samples of `value_noise`, each octave `lacunarity` times finer
// and `persistence` times weaker than the last, starting at `scale`. With
// `persistence` below 1.0 the result stays within roughly 1 / (1 - persistence)
// times the range of the sampled function; it is not renormalised.
fn fbm_value(uv: vec2<f32>, scale: f32, octaves: u32, lacunarity: f32, persistence: f32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * value_noise(uv, frequency);
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return value;
}

// `fbm_value` over `perlin_noise`.
fn fbm_perlin(uv: vec2<f32>, scale: f32, octaves: u32, lacunarity: f32, persistence: f32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * perlin_noise(uv, frequency);
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return value;
}

// `fbm_value` over `simplex_noise`.
fn fbm_simplex(uv: vec2<f32>, scale: f32, octaves: u32, lacunarity: f32, persistence: f32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * simplex_noise(uv, frequency);
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return value;
}

// The unweighted mean of all four noise functions at a fixed scale of 5.0, as a
// ready-made starting point for material experiments.
fn combined_noise(uv: vec2<f32>) -> f32 {
    let p = perlin_noise(uv, 5.0);
    let s = simplex_noise(uv, 5.0);
    let v = value_noise(uv, 5.0);
    let w = worley_noise(uv, 5.0);

    return (p + s + v + w) / 4.0;
}
