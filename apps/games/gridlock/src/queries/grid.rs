use crate::data::districts::{
    COMMERCIAL_REACH, DOWNTOWN_REACH, District, INDUSTRIAL_FROM, INDUSTRIAL_SPAN,
};
use crate::data::motion::LIGHT_YELLOW;
use crate::data::scale::{
    AHEAD_LOAD, ASPHALT, CELL, CELL_TILES, LANE, ROAD_LIFT, SIDEWALK, TILE, VIEW_SECTORS,
};
use nightshade::prelude::{Vec2, Vec3};

pub fn tile_center(key: [i32; 2], column: i32, row: i32) -> Vec3 {
    Vec3::new(
        ((key[0] * CELL_TILES + column) as f32 + 0.5) * TILE,
        0.0,
        ((key[1] * CELL_TILES + row) as f32 + 0.5) * TILE,
    )
}

pub fn cell_of(position: Vec3) -> [i32; 2] {
    [
        (position.x / CELL).floor() as i32,
        (position.z / CELL).floor() as i32,
    ]
}

pub fn surface_height(position: Vec3) -> f32 {
    let across = (position.x.rem_euclid(CELL) - TILE * 0.5).abs();
    let along = (position.z.rem_euclid(CELL) - TILE * 0.5).abs();
    if across <= ASPHALT || along <= ASPHALT {
        ROAD_LIFT
    } else {
        0.0
    }
}

pub fn district_of(key: [i32; 2]) -> District {
    let reach = key[0].abs().max(key[1].abs());
    if reach <= DOWNTOWN_REACH {
        District::Downtown
    } else if reach <= COMMERCIAL_REACH {
        District::Commercial
    } else if key[0] >= INDUSTRIAL_FROM && key[1].abs() <= INDUSTRIAL_SPAN {
        District::Industrial
    } else {
        District::Suburban
    }
}

pub fn heading_vector(heading: u8) -> Vec3 {
    match heading % 4 {
        0 => Vec3::new(1.0, 0.0, 0.0),
        1 => Vec3::new(0.0, 0.0, 1.0),
        2 => Vec3::new(-1.0, 0.0, 0.0),
        _ => Vec3::new(0.0, 0.0, -1.0),
    }
}

pub fn right_of(heading: Vec3) -> Vec3 {
    Vec3::new(-heading.z, 0.0, heading.x)
}

pub fn yaw_of(direction: Vec3) -> f32 {
    direction.x.atan2(direction.z)
}

pub fn heading_of_yaw(yaw: f32) -> u8 {
    let direction = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let mut best = 0;
    let mut best_dot = f32::MIN;
    for heading in 0..4 {
        let dot = heading_vector(heading).dot(&direction);
        if dot > best_dot {
            best_dot = dot;
            best = heading;
        }
    }
    best
}

pub fn lane_position(intersection: [i32; 2], heading: u8, along: f32) -> Vec3 {
    let center = tile_center(intersection, 0, 0);
    let direction = heading_vector(heading);
    center + direction * along + right_of(direction) * LANE
}

pub fn intersection_ahead(position: Vec3, heading: u8) -> [i32; 2] {
    let cell = cell_of(position);
    let center = tile_center(cell, 0, 0);
    match heading % 4 {
        0 => [
            if position.x > center.x {
                cell[0] + 1
            } else {
                cell[0]
            },
            cell[1],
        ],
        1 => [
            cell[0],
            if position.z > center.z {
                cell[1] + 1
            } else {
                cell[1]
            },
        ],
        2 => [
            if position.x < center.x {
                cell[0] - 1
            } else {
                cell[0]
            },
            cell[1],
        ],
        _ => [
            cell[0],
            if position.z < center.z {
                cell[1] - 1
            } else {
                cell[1]
            },
        ],
    }
}

pub fn intersection_after(intersection: [i32; 2], heading: u8) -> [i32; 2] {
    match heading % 4 {
        0 => [intersection[0] + 1, intersection[1]],
        1 => [intersection[0], intersection[1] + 1],
        2 => [intersection[0] - 1, intersection[1]],
        _ => [intersection[0], intersection[1] - 1],
    }
}

pub fn corner_offset(corner: i32) -> Vec3 {
    match corner.rem_euclid(4) {
        0 => Vec3::new(-SIDEWALK, 0.0, -SIDEWALK),
        1 => Vec3::new(SIDEWALK, 0.0, -SIDEWALK),
        2 => Vec3::new(SIDEWALK, 0.0, SIDEWALK),
        _ => Vec3::new(-SIDEWALK, 0.0, SIDEWALK),
    }
}

pub fn corner_position(node: [i32; 3]) -> Vec3 {
    tile_center([node[0], node[1]], 0, 0) + corner_offset(node[2])
}

pub fn corner_neighbors(node: [i32; 3]) -> [[i32; 3]; 4] {
    let [x, z, corner] = node;
    let corner = corner.rem_euclid(4);
    let before = [x, z, (corner + 3) % 4];
    let after = [x, z, (corner + 1) % 4];
    let far = match corner {
        0 => [x - 1, z, 1],
        1 => [x + 1, z, 0],
        2 => [x + 1, z, 3],
        _ => [x - 1, z, 2],
    };
    let side = match corner {
        0 => [x, z - 1, 3],
        1 => [x, z - 1, 2],
        2 => [x, z + 1, 1],
        _ => [x, z + 1, 0],
    };
    [before, after, far, side]
}

pub fn crossing_open(intersection: [i32; 2], along_z: bool, seconds: f32, cycle: f32) -> bool {
    let salt = ((intersection[0].wrapping_mul(73) ^ intersection[1].wrapping_mul(151))
        .rem_euclid(1000)) as f32
        / 1000.0;
    let phase = (seconds / cycle.max(1.0) + salt).fract();
    let green_z = phase < 0.5;
    let yellow = (phase % 0.5) > 0.5 - LIGHT_YELLOW;
    !yellow && green_z == along_z
}

pub fn view_sector(forward: Vec3) -> i32 {
    let angle = forward.x.atan2(forward.z);
    ((angle / std::f32::consts::TAU + 0.5) * VIEW_SECTORS as f32).floor() as i32 % VIEW_SECTORS
}

pub fn ahead_of(across: i32, along: i32, forward: Vec3) -> f32 {
    let reach = across * across + along * along;
    if reach == 0 {
        return 1.0;
    }
    let flat = Vec2::new(forward.x, forward.z);
    let length = flat.norm().max(1e-4);
    (flat.x * across as f32 + flat.y * along as f32) / (length * (reach as f32).sqrt())
}

pub fn view_rank(across: i32, along: i32, forward: Vec3, behind: i32) -> i32 {
    let reach = across * across + along * along;
    if ahead_of(across, along, forward) >= AHEAD_LOAD {
        reach
    } else {
        reach * behind
    }
}

pub fn in_ring(key: [i32; 2], center: [i32; 2], radius: i32) -> bool {
    (key[0] - center[0]).abs() <= radius && (key[1] - center[1]).abs() <= radius
}

pub fn in_keep(
    key: [i32; 2],
    center: [i32; 2],
    forward: Vec3,
    keep: i32,
    ring: i32,
    ahead: f32,
) -> bool {
    if in_ring(key, center, ring) {
        return true;
    }
    if !in_ring(key, center, keep) {
        return false;
    }
    ahead_of(key[0] - center[0], key[1] - center[1], forward) >= ahead
}
