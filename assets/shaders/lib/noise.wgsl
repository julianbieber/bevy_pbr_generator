#define_import_path pbr_gen::noise

var<private> NOISE_SEED: u32 = 0u;

fn set_noise_seed(seed: f32) {
    NOISE_SEED = hash_u32(bitcast<u32>(seed) ^ 0x9e3779b9u);
}

fn hash_u32(x: u32) -> u32 {
    var h = x;
    h ^= h >> 16u;
    h *= 0x7feb352du;
    h ^= h >> 15u;
    h *= 0x846ca68bu;
    h ^= h >> 16u;
    return h;
}

fn wrap_cell(c: vec2<i32>, period: i32) -> vec2<i32> {
    return ((c % period) + vec2<i32>(period)) % period;
}

fn hash_cell(c: vec2<i32>, period: i32) -> u32 {
    let w = wrap_cell(c, period);
    return hash_u32(u32(w.x) ^ hash_u32(u32(w.y) ^ NOISE_SEED));
}

fn hash_to_unit(h: u32) -> f32 {
    return f32(h) * (1.0 / 4294967295.0);
}

fn hash_to_signed(h: u32) -> f32 {
    return hash_to_unit(h) * 2.0 - 1.0;
}

fn hash_to_vec2(h: u32) -> vec2<f32> {
    return vec2<f32>(hash_to_unit(h), hash_to_unit(hash_u32(h ^ 0x68bc21ebu)));
}

fn gradient_of(h: u32) -> vec2<f32> {
    let angle = hash_to_unit(h) * 6.28318530718;
    return vec2<f32>(cos(angle), sin(angle));
}

fn quintic(t: vec2<f32>) -> vec2<f32> {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn period_of(scale: f32) -> i32 {
    return max(1, i32(round(scale)));
}

fn white_noise(uv: vec2<f32>) -> f32 {
    let c = vec2<i32>(floor(uv * 4096.0));
    return hash_to_signed(hash_cell(c, 4096));
}

fn value_noise(uv: vec2<f32>, scale: f32) -> f32 {
    let p = uv * scale;
    let period = period_of(scale);
    let cell = vec2<i32>(floor(p));
    let f = quintic(fract(p));

    let v00 = hash_to_signed(hash_cell(cell + vec2<i32>(0, 0), period));
    let v10 = hash_to_signed(hash_cell(cell + vec2<i32>(1, 0), period));
    let v01 = hash_to_signed(hash_cell(cell + vec2<i32>(0, 1), period));
    let v11 = hash_to_signed(hash_cell(cell + vec2<i32>(1, 1), period));

    return mix(mix(v00, v10, f.x), mix(v01, v11, f.x), f.y);
}

fn perlin_noise(uv: vec2<f32>, scale: f32) -> f32 {
    let p = uv * scale;
    let period = period_of(scale);
    let cell = vec2<i32>(floor(p));
    let frac = fract(p);
    let f = quintic(frac);

    let g00 = gradient_of(hash_cell(cell + vec2<i32>(0, 0), period));
    let g10 = gradient_of(hash_cell(cell + vec2<i32>(1, 0), period));
    let g01 = gradient_of(hash_cell(cell + vec2<i32>(0, 1), period));
    let g11 = gradient_of(hash_cell(cell + vec2<i32>(1, 1), period));

    let n00 = dot(g00, frac - vec2<f32>(0.0, 0.0));
    let n10 = dot(g10, frac - vec2<f32>(1.0, 0.0));
    let n01 = dot(g01, frac - vec2<f32>(0.0, 1.0));
    let n11 = dot(g11, frac - vec2<f32>(1.0, 1.0));

    return mix(mix(n00, n10, f.x), mix(n01, n11, f.x), f.y) * 1.4142136;
}

fn simplex_noise(uv: vec2<f32>, scale: f32) -> f32 {
    let f2 = 0.3660254;
    let g2 = 0.2113249;

    let p = uv * scale;
    let period = period_of(scale) * 3;

    let skewed = p + vec2<f32>((p.x + p.y) * f2);
    let base = floor(skewed);
    let origin = base - vec2<f32>((base.x + base.y) * g2);
    let d0 = p - origin;

    var step1 = vec2<f32>(0.0, 1.0);
    if d0.x > d0.y {
        step1 = vec2<f32>(1.0, 0.0);
    }

    let d1 = d0 - step1 + vec2<f32>(g2);
    let d2 = d0 - vec2<f32>(1.0) + vec2<f32>(2.0 * g2);

    let c0 = vec2<i32>(base);
    let c1 = c0 + vec2<i32>(step1);
    let c2 = c0 + vec2<i32>(1, 1);

    let n0 = dot(gradient_of(hash_cell(c0, period)), d0);
    let n1 = dot(gradient_of(hash_cell(c1, period)), d1);
    let n2 = dot(gradient_of(hash_cell(c2, period)), d2);

    let w0 = max(0.0, 0.5 - dot(d0, d0));
    let w1 = max(0.0, 0.5 - dot(d1, d1));
    let w2 = max(0.0, 0.5 - dot(d2, d2));

    let t0 = w0 * w0 * w0 * w0;
    let t1 = w1 * w1 * w1 * w1;
    let t2 = w2 * w2 * w2 * w2;

    return 70.0 * (t0 * n0 + t1 * n1 + t2 * n2);
}

fn worley_distances(uv: vec2<f32>, scale: f32, jitter: f32) -> vec2<f32> {
    let p = uv * scale;
    let period = period_of(scale);
    let cell = vec2<i32>(floor(p));
    let frac = fract(p);

    var nearest = 8.0;
    var second = 8.0;

    for (var dy = -1; dy <= 1; dy += 1) {
        for (var dx = -1; dx <= 1; dx += 1) {
            let offset = vec2<f32>(f32(dx), f32(dy));
            let site = offset + vec2<f32>(0.5) + (hash_to_vec2(hash_cell(cell + vec2<i32>(dx, dy), period)) - vec2<f32>(0.5)) * jitter;
            let d = length(site - frac);
            if d < nearest {
                second = nearest;
                nearest = d;
            } else if d < second {
                second = d;
            }
        }
    }

    return vec2<f32>(nearest, second);
}

fn worley_noise(uv: vec2<f32>, scale: f32) -> f32 {
    return worley_distances(uv, scale, 1.0).x;
}

fn worley_edges(uv: vec2<f32>, scale: f32) -> f32 {
    let d = worley_distances(uv, scale, 1.0);
    return d.y - d.x;
}

fn fbm_perlin(uv: vec2<f32>, scale: f32, octaves: i32, lacunarity: f32, persistence: f32) -> f32 {
    var total = 0.0;
    var normalization = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0; i < octaves; i += 1) {
        total += amplitude * perlin_noise(uv, frequency);
        normalization += amplitude;
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return total / max(normalization, 0.0001);
}

fn fbm_value(uv: vec2<f32>, scale: f32, octaves: i32, lacunarity: f32, persistence: f32) -> f32 {
    var total = 0.0;
    var normalization = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0; i < octaves; i += 1) {
        total += amplitude * value_noise(uv, frequency);
        normalization += amplitude;
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return total / max(normalization, 0.0001);
}

fn fbm_simplex(uv: vec2<f32>, scale: f32, octaves: i32, lacunarity: f32, persistence: f32) -> f32 {
    var total = 0.0;
    var normalization = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0; i < octaves; i += 1) {
        total += amplitude * simplex_noise(uv, frequency);
        normalization += amplitude;
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return total / max(normalization, 0.0001);
}

fn fbm_worley(uv: vec2<f32>, scale: f32, octaves: i32, lacunarity: f32, persistence: f32) -> f32 {
    var total = 0.0;
    var normalization = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0; i < octaves; i += 1) {
        total += amplitude * (worley_noise(uv, frequency) * 2.0 - 1.0);
        normalization += amplitude;
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return total / max(normalization, 0.0001);
}

fn ridged_perlin(uv: vec2<f32>, scale: f32, octaves: i32, lacunarity: f32, persistence: f32) -> f32 {
    var total = 0.0;
    var normalization = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0; i < octaves; i += 1) {
        total += amplitude * (1.0 - abs(perlin_noise(uv, frequency)));
        normalization += amplitude;
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return total / max(normalization, 0.0001);
}

fn turbulence(uv: vec2<f32>, scale: f32, octaves: i32, lacunarity: f32, persistence: f32) -> f32 {
    var total = 0.0;
    var normalization = 0.0;
    var amplitude = 1.0;
    var frequency = scale;

    for (var i = 0; i < octaves; i += 1) {
        total += amplitude * abs(perlin_noise(uv, frequency));
        normalization += amplitude;
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    return total / max(normalization, 0.0001);
}

fn domain_warp(uv: vec2<f32>, scale: f32, strength: f32) -> vec2<f32> {
    let wx = perlin_noise(uv, scale);
    let wy = perlin_noise(uv + vec2<f32>(0.37, 0.71), scale);
    return uv + vec2<f32>(wx, wy) * (strength / max(scale, 0.0001));
}

fn to_unit(n: f32) -> f32 {
    return n * 0.5 + 0.5;
}
