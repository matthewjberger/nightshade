use crate::contract::Surface;
use crate::data::rooms::{ROOMS, Room};
use crate::data::route::WAYPOINTS;
use crate::data::surfaces;
use nightshade::ecs::{ChildOf, Entity, Storage};
use nightshade::prelude::Vec3;
use nightshade_platform::prelude::Input;
use nightshade_render::data::material::{Material, TextureTransform};

pub fn span_key(span: f32) -> u32 {
    (span * 2.0).round().clamp(1.0, 120.0) as u32
}

pub fn surface_name(kind: Surface, span: f32) -> String {
    format!("blackmarrow_surface_{kind:?}_{}", span_key(span))
}

pub fn surface_source(kind: Surface) -> (&'static str, [f32; 4]) {
    match kind {
        Surface::Rock => (surfaces::ROCK, surfaces::ROCK_TINT),
        Surface::Wall => (surfaces::WALL, surfaces::WALL_TINT),
        Surface::Floor => (surfaces::FLOOR, surfaces::FLOOR_TINT),
        Surface::Plate => (surfaces::PLATE, surfaces::PLATE_TINT),
        Surface::Ceiling => (surfaces::CEILING, surfaces::CEILING_TINT),
        Surface::Metal => (surfaces::METAL, surfaces::METAL_TINT),
        Surface::Rust => (surfaces::RUST, surfaces::RUST_TINT),
        Surface::Door => (surfaces::DOOR, surfaces::DOOR_TINT),
        Surface::Drawer => (surfaces::DRAWER, surfaces::DRAWER_TINT),
        Surface::Grille => (surfaces::GRILLE, surfaces::GRILLE_TINT),
    }
}

pub fn surface_material(kind: Surface, span: f32) -> Material {
    let (texture, tint) = surface_source(kind);
    tiled(texture, tint, span_key(span) as f32 * 0.5)
}

pub fn tile_name(kind: Surface, across: f32, up: f32, offset: [f32; 2]) -> String {
    format!(
        "blackmarrow_tile_{kind:?}_{}_{}_{}_{}",
        hundredths(across),
        hundredths(up),
        hundredths(offset[0]),
        hundredths(offset[1])
    )
}

fn hundredths(value: f32) -> i32 {
    (value * 100.0).round() as i32
}

pub fn tile_material(kind: Surface, across: f32, up: f32, offset: [f32; 2]) -> Material {
    let (texture, tint) = surface_source(kind);
    Material {
        base_color: tint,
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            offset: [
                offset[0] * surfaces::SURFACE_TILING,
                offset[1] * surfaces::SURFACE_TILING,
            ],
            scale: [
                across * surfaces::SURFACE_TILING,
                up * surfaces::SURFACE_TILING,
            ],
            ..Default::default()
        },
        roughness: 0.93,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn tiled(texture: &str, tint: [f32; 4], span: f32) -> Material {
    Material {
        base_color: tint,
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: [
                span * surfaces::SURFACE_TILING,
                span * surfaces::SURFACE_TILING,
            ],
            ..Default::default()
        },
        roughness: 0.93,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn plain(tint: [f32; 4], glow: [f32; 3], roughness: f32) -> Material {
    Material {
        base_color: tint,
        emissive_factor: glow,
        roughness,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn descends_from(storage: &Storage, entity: Entity, root: Entity) -> bool {
    let mut cursor = entity;
    for _depth in 0..64 {
        if cursor == root {
            return true;
        }
        match storage.get::<ChildOf>(cursor) {
            Some(parent) => cursor = parent.0,
            None => return false,
        }
    }
    false
}

pub fn room_at(point: Vec3) -> Option<&'static Room> {
    ROOMS.iter().find(|room| {
        point.x >= room.x0 && point.x <= room.x1 && point.z >= room.z0 && point.z <= room.z1
    })
}

pub fn hold_push(input: &Input) -> f32 {
    input.mouse.wheel_delta.y
}

pub fn nearest_waypoint(point: Vec3) -> usize {
    let mut best = 0;
    let mut distance = f32::MAX;
    for (index, waypoint) in WAYPOINTS.iter().enumerate() {
        let reach = (waypoint.at.xz() - point.xz()).magnitude();
        if reach < distance {
            distance = reach;
            best = index;
        }
    }
    best
}

pub fn next_hop(from: usize, to: usize) -> usize {
    if from == to || from >= WAYPOINTS.len() || to >= WAYPOINTS.len() {
        return to.min(WAYPOINTS.len() - 1);
    }
    let mut came_from = vec![usize::MAX; WAYPOINTS.len()];
    let mut queue = std::collections::VecDeque::new();
    came_from[from] = from;
    queue.push_back(from);
    while let Some(here) = queue.pop_front() {
        if here == to {
            break;
        }
        for &link in WAYPOINTS[here].links {
            if link < WAYPOINTS.len() && came_from[link] == usize::MAX {
                came_from[link] = here;
                queue.push_back(link);
            }
        }
    }
    if came_from[to] == usize::MAX {
        return from;
    }
    let mut step = to;
    while came_from[step] != from && came_from[step] != usize::MAX {
        step = came_from[step];
    }
    step
}
