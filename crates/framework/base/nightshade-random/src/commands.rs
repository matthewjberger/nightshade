use crate::resources::Random;
use nalgebra_glm::{Vec2, Vec3, vec2, vec3};

pub fn chance(random: &mut Random, likelihood: f32) -> bool {
    random.0.f32() < likelihood
}

pub fn between(random: &mut Random, low: f32, high: f32) -> f32 {
    low + random.0.f32() * (high - low)
}

pub fn span(random: &mut Random, reach: f32) -> f32 {
    between(random, -reach, reach)
}

pub fn count(random: &mut Random, low: usize, high: usize) -> usize {
    if high <= low {
        return low;
    }
    low + random.0.usize(0..high - low)
}

pub fn index(random: &mut Random, length: usize) -> usize {
    if length == 0 {
        return 0;
    }
    random.0.usize(0..length)
}

pub fn ring(random: &mut Random, radius: f32) -> Vec2 {
    let angle = between(random, 0.0, std::f32::consts::TAU);
    vec2(angle.cos() * radius, angle.sin() * radius)
}

pub fn disc(random: &mut Random, radius: f32) -> Vec2 {
    let reach = radius * random.0.f32().sqrt();
    ring(random, reach)
}

pub fn direction(random: &mut Random) -> Vec3 {
    let height = between(random, -1.0, 1.0);
    let angle = between(random, 0.0, std::f32::consts::TAU);
    let reach = (1.0 - height * height).max(0.0).sqrt();
    vec3(angle.cos() * reach, height, angle.sin() * reach)
}

pub fn annulus(random: &mut Random, inner: f32, outer: f32) -> Vec2 {
    let reach = between(random, inner.min(outer), outer.max(inner));
    ring(random, reach)
}

pub fn pick<'items, T>(random: &mut Random, items: &'items [T]) -> Option<&'items T> {
    if items.is_empty() {
        return None;
    }
    items.get(index(random, items.len()))
}

pub fn shuffle<T>(random: &mut Random, items: &mut [T]) {
    for position in (1..items.len()).rev() {
        items.swap(position, index(random, position + 1));
    }
}

pub fn whole(random: &mut Random, low: i32, high: i32) -> i32 {
    low + random.0.i32(0..(high - low).max(1))
}

pub fn around(random: &mut Random, center: Vec2, inner: f32, outer: f32) -> Vec2 {
    center + annulus(random, inner, outer)
}
