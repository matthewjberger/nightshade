use crate::data::footprints::FOOTPRINTS;
use crate::data::motion::POST_HALF;
use crate::data::scale::{BLOCK_SCALE, FURNITURE_SCALE, KIT_SCALE, URBAN_SCALE};
use crate::resources::{Lot, Placement};
use nightshade::prelude::{Vec2, Vec3};

pub fn footprint(name: &str) -> ([f32; 3], [f32; 3]) {
    FOOTPRINTS
        .iter()
        .find(|(model, _, _)| *model == name)
        .map(|(_, minimum, maximum)| (*minimum, *maximum))
        .unwrap_or(([-0.5, 0.0, -0.5], [0.5, 1.0, 0.5]))
}

pub fn extent(name: &str, scale: f32) -> Vec3 {
    let (minimum, maximum) = footprint(name);
    Vec3::new(
        (maximum[0] - minimum[0]) * scale,
        (maximum[1] - minimum[1]) * scale,
        (maximum[2] - minimum[2]) * scale,
    )
}

pub fn kit_scale(name: &str) -> f32 {
    if name.starts_with("furn-") {
        FURNITURE_SCALE
    } else if name.starts_with("urban-") {
        URBAN_SCALE
    } else if name.starts_with("block-") {
        BLOCK_SCALE
    } else {
        KIT_SCALE
    }
}

fn quarter_turn(offset: Vec2, yaw: f32) -> Vec2 {
    let quarter = (yaw / std::f32::consts::FRAC_PI_2).round() as i32;
    match quarter.rem_euclid(4) {
        0 => offset,
        1 => Vec2::new(offset.y, -offset.x),
        2 => -offset,
        _ => Vec2::new(-offset.y, offset.x),
    }
}

fn middle_of(name: &str, scale: f32) -> (Vec2, Vec2) {
    let (minimum, maximum) = footprint(name);
    let center = Vec2::new(
        (minimum[0] + maximum[0]) * 0.5,
        (minimum[2] + maximum[2]) * 0.5,
    ) * scale;
    let half = Vec2::new(
        (maximum[0] - minimum[0]) * 0.5,
        (maximum[2] - minimum[2]) * 0.5,
    ) * scale;
    (center, half)
}

pub fn lot_of(placement: &Placement, name: &str) -> Lot {
    let (center, half) = middle_of(name, placement.scale);
    let turned = quarter_turn(center, placement.yaw);
    let quarter = (placement.yaw / std::f32::consts::FRAC_PI_2).round() as i32;
    let half = if quarter.rem_euclid(2) == 0 {
        half
    } else {
        Vec2::new(half.y, half.x)
    };
    let middle = Vec2::new(placement.position.x, placement.position.z) + turned;
    Lot {
        minimum: middle - half,
        maximum: middle + half,
    }
}

pub fn origin_for(name: &str, scale: f32, center: Vec2, yaw: f32) -> Vec3 {
    let (local, _) = middle_of(name, scale);
    let turned = quarter_turn(local, yaw);
    Vec3::new(center.x - turned.x, 0.0, center.y - turned.y)
}

pub fn post_lot(position: Vec3) -> Lot {
    let center = Vec2::new(position.x, position.z);
    Lot {
        minimum: center - Vec2::new(POST_HALF, POST_HALF),
        maximum: center + Vec2::new(POST_HALF, POST_HALF),
    }
}
