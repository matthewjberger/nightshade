use crate::contract::{Ask, NearAsks};
use nalgebra_glm::Vec3;

pub fn ask_what_is_near(asks: &mut NearAsks, tag: u32, point: Vec3, radius: f32) {
    asks.asks.retain(|held| held.tag != tag);
    asks.asks.push(Ask { tag, point, radius });
}

pub fn stop_asking(asks: &mut NearAsks, tag: u32) {
    asks.asks.retain(|held| held.tag != tag);
}
