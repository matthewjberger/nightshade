use crate::components::EnemyKind;
use crate::data;
use crate::data::arsenal::{Gun, gun};
use crate::data::bestiary::beast;
use crate::resources::PlayerState;
use nightshade::prelude::{Vec2, Vec3};

pub fn held(player: &PlayerState) -> &'static Gun {
    gun(player.gun)
}

pub fn rounds(player: &PlayerState) -> u32 {
    player.rounds(held(player).ammo)
}

pub fn can_fire(player: &PlayerState) -> bool {
    !player.dead && player.cooldown <= 0.0 && rounds(player) > 0
}

pub fn beast_height(kind: EnemyKind) -> f32 {
    beast(kind).height
}

pub fn hit_center(kind: EnemyKind) -> f32 {
    beast(kind).height * data::HIT_CENTER_SHARE
}

pub fn hit_radius(kind: EnemyKind) -> f32 {
    (beast(kind).height * data::HIT_RADIUS_SHARE).max(data::HIT_RADIUS_FLOOR)
}

pub fn ray_hits_column(
    origin: Vec3,
    ray: Vec3,
    foot: Vec3,
    height: f32,
    radius: f32,
    limit: f32,
) -> Option<f32> {
    let top = foot.y + height + data::HIT_OVERHANG;
    let bottom = foot.y - data::HIT_OVERHANG;
    let to = Vec2::new(foot.x - origin.x, foot.z - origin.z);
    let flat = Vec2::new(ray.x, ray.z);
    let reach = flat.magnitude();
    if reach < 0.0001 {
        if to.magnitude() > radius || ray.y.abs() < 0.0001 {
            return None;
        }
        let range = if ray.y > 0.0 {
            (bottom - origin.y) / ray.y
        } else {
            (top - origin.y) / ray.y
        };
        return (range > 0.0 && range <= limit).then_some(range);
    }
    let heading = flat / reach;
    let along = to.dot(&heading);
    let side = (to - heading * along).magnitude();
    if side > radius {
        return None;
    }
    let back = (radius * radius - side * side).sqrt();
    let range = ((along - back) / reach).max(0.0);
    if range > limit {
        return None;
    }
    let lift = origin.y + ray.y * range;
    (lift >= bottom && lift <= top).then_some(range)
}

pub fn point_in_column(point: Vec3, foot: Vec3, height: f32, radius: f32) -> bool {
    let flat = Vec2::new(point.x - foot.x, point.z - foot.z).magnitude();
    flat <= radius
        && point.y >= foot.y - data::HIT_OVERHANG
        && point.y <= foot.y + height + data::HIT_OVERHANG
}

pub fn ceiling(ammo: crate::data::arsenal::Ammo) -> u32 {
    match ammo {
        crate::data::arsenal::Ammo::Shells => data::MAX_SHELLS,
        crate::data::arsenal::Ammo::Bullets => data::MAX_BULLETS,
        crate::data::arsenal::Ammo::Rockets => data::MAX_ROCKETS,
        crate::data::arsenal::Ammo::Cells => data::MAX_CELLS,
    }
}

pub fn falloff(range: f32) -> f32 {
    let start = data::SHOTGUN_FALLOFF_START;
    let end = data::SHOTGUN_FALLOFF_END;
    if range <= start {
        return 1.0;
    }
    if range >= end {
        return data::SHOTGUN_FAR_SHARE;
    }
    let share = (range - start) / (end - start);
    1.0 - share * (1.0 - data::SHOTGUN_FAR_SHARE)
}

pub fn keys_line(player: &PlayerState) -> String {
    match (player.red_key, player.blue_key) {
        (true, true) => "red blue".to_string(),
        (true, false) => "red".to_string(),
        (false, true) => "blue".to_string(),
        (false, false) => String::new(),
    }
}

pub fn share(part: u32, whole: u32) -> u32 {
    if whole == 0 {
        return 100;
    }
    ((part as f32 / whole as f32) * 100.0).round() as u32
}

pub fn clock_line(seconds: f32) -> String {
    let whole = seconds.max(0.0) as u32;
    format!("{}:{:02}", whole / 60, whole % 60)
}
