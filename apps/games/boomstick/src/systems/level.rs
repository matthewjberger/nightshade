use crate::commands::{spawn_block, spawn_plate};
use crate::components::{Group, Health, Lock, OpenShare, ShutHeight, Travel};
use crate::data;
use crate::data::levels::{Gap, LevelPlan};
use crate::systems::textures;
use crate::tags::{Door, ExitPad, LevelPiece, Lever, Secret};
use nightshade::prelude::*;
use nightshade_graphics::prelude::spawn_light_entity;
use nightshade_render::contract::{Light, LightType};

const DECK_TINT: [f32; 4] = [0.86, 0.88, 0.94, 1.0];
const WALL_TINT: [f32; 4] = [0.66, 0.72, 0.86, 1.0];
const LEDGE_TINT: [f32; 4] = [0.84, 0.82, 0.80, 1.0];
const ROOF_TINT: [f32; 4] = [0.30, 0.31, 0.38, 1.0];
const PILLAR_TINT: [f32; 4] = [0.60, 0.60, 0.64, 1.0];
const CRATE_TINT: [f32; 4] = [0.78, 0.70, 0.46, 1.0];
const SLAG_TINT: [f32; 4] = [1.0, 0.50, 0.22, 1.0];
const KEYED: [f32; 4] = [0.96, 0.90, 0.80, 1.0];
const DECK_TILE: f32 = 0.12;
const WALL_TILE: f32 = 0.16;
const ROOF_TILE: f32 = 0.10;
const LEDGE_TILE: f32 = 0.16;
const PILLAR_TILE: f32 = 0.22;
const CRATE_TILE: f32 = 0.70;
const WALL_THICKNESS: f32 = 1.0;
const EDGE_TOLERANCE: f32 = 0.6;
const JAMB_CLEARANCE: f32 = 0.08;
const JAMB_BITE: f32 = 0.12;
const SEAM_BITE: f32 = 0.06;

struct Builder {
    made: Vec<[i32; 6]>,
    count: usize,
}

impl Builder {
    fn new() -> Self {
        Self {
            made: Vec::new(),
            count: 0,
        }
    }

    fn wall(&mut self, commands: &mut Commands, position: Vec3, size: Vec3) {
        let key = [
            (position.x * 20.0) as i32,
            (position.y * 20.0) as i32,
            (position.z * 20.0) as i32,
            (size.x * 20.0) as i32,
            (size.y * 20.0) as i32,
            (size.z * 20.0) as i32,
        ];
        if self.made.contains(&key) {
            return;
        }
        self.made.push(key);
        let run = size.x.max(size.z);
        let entity = spawn_block(
            commands,
            &format!("boomstick_wall_{}", self.count),
            position,
            size,
            textures::tiled(
                textures::BRICK,
                textures::tiles(run, size.y, WALL_TILE),
                WALL_TINT,
            ),
        );
        commands.add_tag_type::<LevelPiece>(entity);
        self.count += 1;
    }
}

pub fn build(commands: &mut Commands, plan: &LevelPlan) {
    let mut builder = Builder::new();

    for (index, room) in plan.rooms.iter().enumerate() {
        let (center, footprint, height, roofed) = *room;
        floor(
            commands,
            index,
            Vec3::new(center.x, center.y - 0.5, center.z),
            Vec3::new(footprint.x, 1.0, footprint.y),
        );
        if roofed {
            let entity = spawn_block(
                commands,
                &format!("boomstick_roof_{index}"),
                Vec3::new(center.x, center.y + height + 0.5 - SEAM_BITE, center.z),
                Vec3::new(footprint.x, 1.0, footprint.y),
                textures::tiled(
                    textures::DECK,
                    textures::tiles(footprint.x, footprint.y, ROOF_TILE),
                    ROOF_TINT,
                ),
            );
            commands.add_tag_type::<LevelPiece>(entity);
        }
        let span = footprint.x.max(footprint.y);
        let fill = spawn_light_entity(
            commands,
            Vec3::new(center.x, center.y + height * data::LIGHT_SHARE, center.z),
            &format!("boomstick_fill_{index}"),
        );
        commands.set(
            fill,
            Light {
                light_type: LightType::Point,
                color: Vec3::new(1.0, 0.94, 0.88),
                intensity: data::FILL_BASE + footprint.x * footprint.y * data::FILL_PER_AREA,
                range: span * 1.4,
                cast_shadows: false,
                ..Default::default()
            },
        );
        commands.add_tag_type::<LevelPiece>(fill);
    }
    shells(commands, &mut builder, plan);

    for (index, (position, size)) in plan.floors.iter().enumerate() {
        floor(commands, plan.rooms.len() + index, *position, *size);
    }

    for (index, (position, size)) in plan.ledges.iter().enumerate() {
        let run = size.x.max(size.z);
        let entity = spawn_block(
            commands,
            &format!("boomstick_ledge_{index}"),
            *position,
            *size,
            textures::tiled(
                textures::GRATE,
                textures::tiles(run, size.z.min(size.x).max(size.y), LEDGE_TILE),
                LEDGE_TINT,
            ),
        );
        commands.add_tag_type::<LevelPiece>(entity);
    }

    for (index, (position, size)) in plan.pillars.iter().enumerate() {
        let entity = spawn_block(
            commands,
            &format!("boomstick_pillar_{index}"),
            *position - Vec3::new(0.0, SEAM_BITE, 0.0),
            *size,
            textures::tiled(
                textures::RUST,
                textures::tiles(size.x, size.y, PILLAR_TILE),
                PILLAR_TINT,
            ),
        );
        commands.add_tag_type::<LevelPiece>(entity);
    }

    for (index, base) in plan.crates.iter().enumerate() {
        let size = data::CRATE_SIZE;
        let entity = spawn_block(
            commands,
            &format!("boomstick_crate_{index}"),
            Vec3::new(base.x, base.y + size.y * 0.5, base.z),
            size,
            textures::tiled(
                textures::RUST,
                textures::tiles(size.x, size.y, CRATE_TILE),
                CRATE_TINT,
            ),
        );
        commands.add_tag_type::<LevelPiece>(entity);
    }

    for (index, (position, size, lock, group)) in plan.gates.iter().enumerate() {
        let tint = match lock {
            Lock::Red => [1.0, 0.42, 0.36, 1.0],
            Lock::Blue => [0.48, 0.66, 1.0, 1.0],
            Lock::Lever => KEYED,
        };
        let door = spawn_block(
            commands,
            &format!("boomstick_gate_{index}"),
            *position,
            fitted(*size),
            textures::glowing(
                textures::HATCH,
                [2.0, 1.5],
                tint,
                [tint[0] * 0.3, tint[1] * 0.3, tint[2] * 0.3],
            ),
        );
        commands.set(door, *lock);
        commands.set(door, Group(*group));
        commands.set(door, ShutHeight(position.y));
        commands.set(door, Travel(-(size.y + 0.4)));
        commands.set(door, OpenShare(0.0));
        commands.add_tag_type::<Door>(door);
        commands.add_tag_type::<LevelPiece>(door);
    }

    for (index, (position, size, group)) in plan.panels.iter().enumerate() {
        let lever = spawn_block(
            commands,
            &format!("boomstick_lever_{index}"),
            *position,
            *size,
            textures::glowing(
                textures::SIGN,
                [1.0, 1.0],
                [1.0, 0.86, 0.42, 1.0],
                [1.6, 0.82, 0.14],
            ),
        );
        commands.set(lever, Group(*group));
        commands.set(lever, Health(data::LEVER_HEALTH));
        commands.add_tag_type::<Lever>(lever);
        commands.add_tag_type::<LevelPiece>(lever);
    }

    for (index, (position, size)) in plan.secrets.iter().enumerate() {
        let wall = spawn_block(
            commands,
            &format!("boomstick_secret_{index}"),
            *position,
            fitted(*size),
            textures::tiled(textures::SIGN, [1.6, 1.2], [0.84, 0.82, 0.92, 1.0]),
        );
        commands.set(wall, Health(data::SECRET_HEALTH));
        commands.add_tag_type::<Secret>(wall);
        commands.add_tag_type::<LevelPiece>(wall);
    }

    for (index, (position, half, _)) in plan.hazards.iter().enumerate() {
        let pool = spawn_plate(
            commands,
            &format!("boomstick_slag_{index}"),
            Vec3::new(position.x, position.y - half.y + 0.08, position.z),
            Vec3::new(half.x * 2.0, 0.16, half.z * 2.0),
            textures::glowing(
                textures::RUST,
                textures::tiles(half.x * 2.0, half.z * 2.0, DECK_TILE),
                SLAG_TINT,
                [2.2, 0.60, 0.10],
            ),
        );
        commands.add_tag_type::<LevelPiece>(pool);
    }

    for (index, (position, color, intensity, range)) in plan.lamps.iter().enumerate() {
        let lamp = spawn_light_entity(
            commands,
            under_the_roof(plan, *position),
            &format!("boomstick_lamp_{index}"),
        );
        commands.set(
            lamp,
            Light {
                light_type: LightType::Point,
                color: Vec3::new(color[0], color[1], color[2]),
                intensity: *intensity * data::LAMP_SHARE,
                range: *range,
                cast_shadows: false,
                ..Default::default()
            },
        );
        commands.add_tag_type::<LevelPiece>(lamp);
    }

    let pad = spawn_plate(
        commands,
        "boomstick_exit",
        plan.exit + Vec3::new(0.0, 0.14, 0.0),
        Vec3::new(3.2, 0.2, 3.2),
        textures::glowing(
            textures::SIGN,
            [1.0, 1.0],
            [0.66, 1.0, 0.72, 1.0],
            [0.24, 2.20, 0.60],
        ),
    );
    commands.add_tag_type::<ExitPad>(pad);
    commands.add_tag_type::<LevelPiece>(pad);

    let beacon = spawn_plate(
        commands,
        "boomstick_exit_beacon",
        plan.exit + Vec3::new(0.0, data::EXIT_BEACON * 0.5, 0.0),
        Vec3::new(0.35, data::EXIT_BEACON, 0.35),
        textures::glowing(
            textures::SIGN,
            [1.0, 4.0],
            [0.66, 1.0, 0.72, 1.0],
            [0.18, 1.80, 0.48],
        ),
    );
    commands.add_tag_type::<LevelPiece>(beacon);

    let glow = spawn_light_entity(
        commands,
        plan.exit + Vec3::new(0.0, 2.0, 0.0),
        "boomstick_exit_lamp",
    );
    commands.set(
        glow,
        Light {
            light_type: LightType::Point,
            color: Vec3::new(0.42, 1.0, 0.58),
            intensity: 30.0,
            range: 12.0,
            cast_shadows: false,
            ..Default::default()
        },
    );
    commands.add_tag_type::<LevelPiece>(glow);
}

fn under_the_roof(plan: &LevelPlan, at: Vec3) -> Vec3 {
    let mut ceiling = f32::MAX;
    for (center, footprint, height, _) in plan.rooms {
        let half = Vec2::new(footprint.x * 0.5, footprint.y * 0.5);
        if (at.x - center.x).abs() <= half.x && (at.z - center.z).abs() <= half.y {
            ceiling = ceiling.min(center.y + height * data::LIGHT_SHARE);
        }
    }
    Vec3::new(at.x, at.y.min(ceiling), at.z)
}

fn fitted(size: Vec3) -> Vec3 {
    Vec3::new(
        size.x - JAMB_CLEARANCE,
        size.y + JAMB_BITE,
        size.z - JAMB_CLEARANCE,
    )
}

fn floor(commands: &mut Commands, index: usize, position: Vec3, size: Vec3) {
    let entity = spawn_block(
        commands,
        &format!("boomstick_floor_{index}"),
        position,
        size,
        textures::tiled(
            textures::DECK,
            textures::tiles(size.x, size.z, DECK_TILE),
            DECK_TINT,
        ),
    );
    commands.add_tag_type::<LevelPiece>(entity);
}

struct Face {
    along_x: bool,
    fixed: f32,
    base: f32,
    height: f32,
    low: f32,
    high: f32,
}

fn shells(commands: &mut Commands, builder: &mut Builder, plan: &LevelPlan) {
    let mut faces: Vec<Face> = Vec::new();
    for room in plan.rooms {
        let (center, footprint, height, _) = *room;
        let half = Vec2::new(footprint.x * 0.5, footprint.y * 0.5);
        for side in 0..4 {
            let along_x = side < 2;
            let fixed = match side {
                0 => center.z + half.y,
                1 => center.z - half.y,
                2 => center.x + half.x,
                _ => center.x - half.x,
            };
            let span = if along_x { half.x } else { half.y };
            let middle = if along_x { center.x } else { center.z };
            faces.push(Face {
                along_x,
                fixed,
                base: center.y,
                height,
                low: middle - span - WALL_THICKNESS * 0.5,
                high: middle + span + WALL_THICKNESS * 0.5,
            });
        }
    }
    faces.sort_by(|left, right| right.height.total_cmp(&left.height));

    let mut claimed: Vec<(bool, i32, f32, f32)> = Vec::new();
    for face in &faces {
        let key = (face.fixed * 20.0).round() as i32;
        let mut free = vec![(face.low, face.high)];
        for (along_x, held, low, high) in &claimed {
            if *along_x != face.along_x || *held != key {
                continue;
            }
            free = without(free, *low, *high);
        }
        for (low, high) in &free {
            if high - low <= 0.05 {
                continue;
            }
            raise_face(
                commands,
                builder,
                face,
                low - SEAM_BITE,
                high + SEAM_BITE,
                plan.gaps,
            );
        }
        claimed.push((face.along_x, key, face.low, face.high));
    }
}

fn without(spans: Vec<(f32, f32)>, low: f32, high: f32) -> Vec<(f32, f32)> {
    let mut left = Vec::new();
    for (start, end) in spans {
        if high <= start || low >= end {
            left.push((start, end));
            continue;
        }
        if low > start {
            left.push((start, low));
        }
        if high < end {
            left.push((high, end));
        }
    }
    left
}

fn raise_face(
    commands: &mut Commands,
    builder: &mut Builder,
    face: &Face,
    low: f32,
    high: f32,
    gaps: &[Gap],
) {
    let mut holes: Vec<(f32, f32, f32)> = gaps
        .iter()
        .filter(|(at, _, _)| {
            let across = if face.along_x { at.z } else { at.x };
            (across - face.fixed).abs() < EDGE_TOLERANCE
        })
        .filter(|(at, width, _)| {
            let along = if face.along_x { at.x } else { at.z };
            along - width * 0.5 > low - 0.01 && along + width * 0.5 < high + 0.01
        })
        .map(|(at, width, gap_height)| {
            let along = if face.along_x { at.x } else { at.z };
            (along - width * 0.5, along + width * 0.5, *gap_height)
        })
        .collect();
    holes.sort_by(|left, right| left.0.total_cmp(&right.0));

    let edge = Edge {
        along_x: face.along_x,
        fixed: face.fixed,
        base: face.base,
        height: face.height,
    };
    let mut cursor = low;
    for (hole_low, hole_high, hole_height) in &holes {
        if hole_low - cursor > 0.05 {
            segment(builder, commands, &edge, cursor, *hole_low);
        }
        if face.height - hole_height > 0.05 {
            let lintel = Edge {
                base: face.base + hole_height,
                height: face.height - hole_height,
                ..edge
            };
            segment(
                builder,
                commands,
                &lintel,
                hole_low - SEAM_BITE,
                hole_high + SEAM_BITE,
            );
        }
        cursor = *hole_high;
    }
    if high - cursor > 0.05 {
        segment(builder, commands, &edge, cursor, high);
    }
}

#[derive(Clone, Copy)]
struct Edge {
    along_x: bool,
    fixed: f32,
    base: f32,
    height: f32,
}

fn segment(builder: &mut Builder, commands: &mut Commands, edge: &Edge, low: f32, high: f32) {
    let middle = (low + high) * 0.5;
    let length = high - low;
    let height = edge.height + SEAM_BITE;
    let lift = edge.base - SEAM_BITE + height * 0.5;
    let position = if edge.along_x {
        Vec3::new(middle, lift, edge.fixed)
    } else {
        Vec3::new(edge.fixed, lift, middle)
    };
    let size = if edge.along_x {
        Vec3::new(length, height, WALL_THICKNESS)
    } else {
        Vec3::new(WALL_THICKNESS, height, length)
    };
    builder.wall(commands, position, size);
}
