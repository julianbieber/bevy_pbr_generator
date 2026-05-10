//! Manual implementations of noise functions for texture generation.
//!
//! Note: These functions are not used by default. Users should edit the texture functions
//! in `textures.rs` to use them.

use glam::Vec2;

#[allow(dead_code)]
/// White noise: random value per pixel (pseudo-random based on coordinates).
#[inline]
pub fn white_noise(uv: Vec2) -> f32 {
    let x = uv.x as i32;
    let y = uv.y as i32;
    // Simple hash function for pseudo-randomness
    let mut hash = x.wrapping_mul(123456791).wrapping_add(y.wrapping_mul(987654321));
    hash = (hash ^ (hash >> 16)).wrapping_mul(0x85ebca6b_u32 as i32);
    hash = (hash ^ (hash >> 13)).wrapping_mul(0xc2b2ae35_u32 as i32);
    hash = hash ^ (hash >> 16);
    (hash as f32) / (i32::MAX as f32) * 2.0 - 1.0
}

#[allow(dead_code)]
/// Value noise: interpolated white noise.
#[inline]
pub fn value_noise(uv: Vec2, scale: f32) -> f32 {
    let scaled_uv = uv * scale;
    let x0 = scaled_uv.x.floor() as i32;
    let y0 = scaled_uv.y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let sx = scaled_uv.x - x0 as f32;
    let sy = scaled_uv.y - y0 as f32;

    // Compute white noise at corners
    let n00 = white_noise(Vec2::new(x0 as f32, y0 as f32));
    let n01 = white_noise(Vec2::new(x0 as f32, y1 as f32));
    let n10 = white_noise(Vec2::new(x1 as f32, y0 as f32));
    let n11 = white_noise(Vec2::new(x1 as f32, y1 as f32));

    // Linear interpolation
    let ix0 = n00 + sx * (n10 - n00);
    let ix1 = n01 + sx * (n11 - n01);
    ix0 + sy * (ix1 - ix0)
}

#[allow(dead_code)]
#[inline]
fn smoothstep(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[allow(dead_code)]
/// Permutation table for Perlin noise.
static PERM: [u8; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23,
    190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20,
    125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230, 220,
    105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169,
    200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147,
    118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2,
    44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185,
    112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239,
    107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
];

#[allow(dead_code)]
/// Perlin noise implementation.
#[inline]
pub fn perlin_noise(uv: Vec2, scale: f32) -> f32 {
    let scaled_uv = uv * scale;
    let x0 = scaled_uv.x.floor() as i32;
    let y0 = scaled_uv.y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let sx = scaled_uv.x - x0 as f32;
    let sy = scaled_uv.y - y0 as f32;

    // Compute fade curves
    let u = smoothstep(sx);
    let v = smoothstep(sy);

    // Hash coordinates to get pseudo-random gradient vectors
    let a = p(x0, y0);
    let b = p(x1, y0);
    let c = p(x0, y1);
    let d = p(x1, y1);

    // Compute dot products
    let grad = |x: f32, y: f32, a: u8| -> f32 {
        let h = a & 15;
        let u = if h < 8 { x } else { y };
        let v_val = if h < 4 { y } else { if h == 12 || h == 14 { x } else { 0.0 } };
        (if (h & 1) == 0 { u } else { -u }) + (if (h & 2) == 0 { v_val } else { -v_val })
    };

    let x1_val = grad(sx, sy, a);
    let x2_val = grad(sx - 1.0, sy, b);
    let y1_val = grad(sx, sy - 1.0, c);
    let y2_val = grad(sx - 1.0, sy - 1.0, d);

    // Interpolate
    let ix0 = x1_val + u * (x2_val - x1_val);
    let ix1 = y1_val + u * (y2_val - y1_val);
    ix0 + v * (ix1 - ix0)
}

#[allow(dead_code)]
fn p(x: i32, y: i32) -> u8 {
    let mut x = x as usize;
    let mut y = y as usize;
    x = (x + PERM[(y + PERM[x & 255] as usize) & 255] as usize) & 255;
    y = (y + PERM[x] as usize) & 255;
    PERM[(x + PERM[y] as usize) & 255]
}

#[allow(dead_code)]
/// Simplex noise implementation (2D).
#[inline]
pub fn simplex_noise(uv: Vec2, scale: f32) -> f32 {
    let scaled_uv = uv * scale;
    
    // Skew the input space to determine which simplex cell we're in
    let skew = (1.0 / 3.0) * (2.0f32.sqrt() + 1.0);
    let s = (scaled_uv.x + scaled_uv.y) * skew;
    let i = s.floor() as i32;
    let j = (scaled_uv.y + (i as f32) * (1.0 / skew)).floor() as i32;
    
    // Unskew the cell origin back to (x,y) space
    let t = (i + j) as f32 * skew;
    let x0 = scaled_uv.x - i as f32 + t;
    let y0 = scaled_uv.y - j as f32 + t;
    
    // Integer coordinates of the simplex cell
    let (i1, j1) = if x0 > y0 {
        (1, 0)
    } else {
        (0, 1)
    };
    
    // Offsets for the other two vertices
    let x1 = x0 - i1 as f32 + skew;
    let y1 = y0 - j1 as f32 + skew;
    let x2 = x0 - 1.0 + 2.0 * skew;
    let y2 = y0 - 1.0 + 2.0 * skew;
    
    // Calculate hashes for the three vertices
    let h0 = hash(i, j);
    let h1 = hash(i + i1, j + j1);
    let h2 = hash(i + 1, j + 1);
    
    // Calculate the contribution from each vertex
    let n0 = dot(grad(h0), Vec2::new(x0, y0));
    let n1 = dot(grad(h1), Vec2::new(x1, y1));
    let n2 = dot(grad(h2), Vec2::new(x2, y2));
    
    // Calculate the falloff for each vertex
    let a0 = 0.5 - x0 * x0 - y0 * y0;
    let a1 = 0.5 - x1 * x1 - y1 * y1;
    let a2 = 0.5 - x2 * x2 - y2 * y2;
    
    // Apply the falloff
    let mut value = 0.0;
    if a0 > 0.0 {
        value += a0.powi(4) * n0;
    }
    if a1 > 0.0 {
        value += a1.powi(4) * n1;
    }
    if a2 > 0.0 {
        value += a2.powi(4) * n2;
    }
    
    value * 70.0 // Scale to roughly match Perlin noise range
}

#[allow(dead_code)]
fn hash(x: i32, y: i32) -> u8 {
    let mut h = x.wrapping_mul(32661).wrapping_add(y.wrapping_mul(65537));
    h = (h ^ (h >> 13)).wrapping_mul(69069);
    h = (h ^ (h >> 17)).wrapping_mul(3571);
    (h ^ (h >> 10)) as u8
}

#[allow(dead_code)]
fn grad(hash: u8) -> Vec2 {
    let h = hash & 15;
    let u = if h < 8 { 1.0 } else { -1.0 };
    let v = if h < 4 { 1.0 } else { if h == 12 || h == 14 { -1.0 } else { 0.0 } };
    Vec2::new(u, v).normalize()
}

#[allow(dead_code)]
fn dot(g: Vec2, v: Vec2) -> f32 {
    g.x * v.x + g.y * v.y
}

#[allow(dead_code)]
/// Worley noise (F1 - distance to nearest point).
#[inline]
pub fn worley_noise(uv: Vec2, scale: f32, points: usize) -> f32 {
    let scaled_uv = uv * scale;
    let mut min_dist = 1.0;
    
    // Use a fixed seed for reproducibility
    let seed: i32 = 42;
    
    for i in 0..points {
        // Generate pseudo-random point
        let mut hash = seed.wrapping_add(i as i32).wrapping_mul(123456791);
        hash = (hash ^ (hash >> 16)).wrapping_mul(0x85ebca6b_u32 as i32);
        hash = (hash ^ (hash >> 13)).wrapping_mul(0xc2b2ae35_u32 as i32);
        hash = hash ^ (hash >> 16);
        
        let px = (hash as f32) / (i32::MAX as f32);
        let py = ((hash >> 16) as f32) / (i32::MAX as f32);
        
        let dx = scaled_uv.x - px;
        let dy = scaled_uv.y - py;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist < min_dist {
            min_dist = dist;
        }
    }
    
    min_dist
}

#[allow(dead_code)]
/// Fractional Brownian Motion (fBm) for multi-octave noise.
#[inline]
pub fn fbm(
    uv: Vec2,
    scale: f32,
    octaves: usize,
    lacunarity: f32,
    persistence: f32,
    noise_fn: fn(Vec2, f32) -> f32,
) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = scale;
    
    for _ in 0..octaves {
        value += amplitude * noise_fn(uv, frequency);
        frequency *= lacunarity;
        amplitude *= persistence;
    }
    
    value
}

#[allow(dead_code)]
/// Combined noise function for testing.
#[inline]
pub fn combined_noise(uv: Vec2) -> f32 {
    let perlin = perlin_noise(uv, 5.0);
    let simplex = simplex_noise(uv, 5.0);
    let value = value_noise(uv, 5.0);
    let worley = worley_noise(uv, 5.0, 20);
    
    // Normalize and combine
    (perlin + simplex + value + worley) / 4.0
}
