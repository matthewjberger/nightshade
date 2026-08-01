pub fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

pub fn smooth_step(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

pub fn smoother_step(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

pub fn approach(value: f32, target: f32, step: f32) -> f32 {
    if (target - value).abs() <= step {
        return target;
    }
    value + (target - value).signum() * step
}

pub fn damp(value: f32, target: f32, rate: f32, delta: f32) -> f32 {
    lerp(value, target, 1.0 - (-rate * delta).exp())
}

pub fn share(rate: f32, delta: f32) -> f32 {
    (rate * delta).clamp(0.0, 1.0)
}

pub fn remap(value: f32, from_low: f32, from_high: f32, to_low: f32, to_high: f32) -> f32 {
    if (from_high - from_low).abs() <= f32::EPSILON {
        return to_low;
    }
    let amount = (value - from_low) / (from_high - from_low);
    lerp(to_low, to_high, amount.clamp(0.0, 1.0))
}
