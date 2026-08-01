pub fn hash(value: u32) -> u32 {
    let mut state = value.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    state = ((state >> ((state >> 28).wrapping_add(4))) ^ state).wrapping_mul(277_803_737);
    (state >> 22) ^ state
}

pub fn hash_two(across: i32, down: i32, seed: u32) -> u32 {
    hash(
        (across as u32)
            .wrapping_mul(374_761_393)
            .wrapping_add((down as u32).wrapping_mul(668_265_263))
            .wrapping_add(seed),
    )
}

pub fn unit(across: i32, down: i32, seed: u32) -> f32 {
    hash_two(across, down, seed) as f32 / u32::MAX as f32
}

pub fn value(across: f32, down: f32, seed: u32) -> f32 {
    let left = across.floor();
    let top = down.floor();
    let inside_across = smooth(across - left);
    let inside_down = smooth(down - top);
    let one = unit(left as i32, top as i32, seed);
    let other = unit(left as i32 + 1, top as i32, seed);
    let third = unit(left as i32, top as i32 + 1, seed);
    let fourth = unit(left as i32 + 1, top as i32 + 1, seed);
    let upper = one + (other - one) * inside_across;
    let lower = third + (fourth - third) * inside_across;
    upper + (lower - upper) * inside_down
}

pub fn layered(across: f32, down: f32, seed: u32, layers: u32, fade: f32) -> f32 {
    let mut sum = 0.0;
    let mut weight = 1.0;
    let mut total = 0.0;
    let mut scale = 1.0;
    for layer in 0..layers.max(1) {
        sum += value(across * scale, down * scale, seed.wrapping_add(layer)) * weight;
        total += weight;
        weight *= fade;
        scale *= 2.0;
    }
    if total <= 0.0 {
        return 0.0;
    }
    sum / total
}

pub fn ridged(across: f32, down: f32, seed: u32, layers: u32, fade: f32) -> f32 {
    let raw = layered(across, down, seed, layers, fade);
    1.0 - (raw * 2.0 - 1.0).abs()
}

pub fn seed_of(across: i32, down: i32) -> u64 {
    let one = hash_two(across, down, 0) as u64;
    let other = hash_two(down, across, 1) as u64;
    (one << 32) | other
}

fn smooth(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}
