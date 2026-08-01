use crate::commands::lights::spawn_room_light;
use crate::commands::models::place_piece;
use crate::components::Indoor;
use crate::data::motion::{
    DOOR_STEP, PROP_GAP, PROP_TRIES, ROOM_CLEAR, ROOM_ENTRY, ROOM_LIGHT_HEIGHT, ROOM_MARGIN,
    ROOM_WALL_GAP,
};
use crate::data::rooms::{
    COLUMN, DOOR_LEAF, FLOOR, HOME_BEDROOM, HOME_KITCHEN, HOME_LOOSE, HOME_RUGS, LOBBY_LOOSE,
    LOBBY_WALLS, OFFICE_LOOSE, OFFICE_WALLS, Room, SHOP_COUNTER, SHOP_LOOSE, SHOP_SHELVES, WALL,
    WALL_DOORWAY, WALL_WINDOW, WAREHOUSE_LOOSE, WAREHOUSE_WALLS, WINDOW_CHANCE,
};
use crate::data::scale::{BLOCK_SCALE, ROOM_FLOOR, ROOM_SITE, ROOM_TILE, ROOM_TILES};
use crate::queries::collide::lots_overlap;
use crate::queries::footprints::{extent, kit_scale, lot_of, origin_for};
use crate::queries::grid::yaw_of;
use crate::resources::{Lot, Placement};
use nightshade::prelude::*;
use nightshade_kit::prelude::model;
use nightshade_random::prelude::*;

const NORTH: usize = 0;
const WEST: usize = 1;
const EAST: usize = 2;

pub fn spawn_room(
    commands: &mut Commands,
    random: &mut Random,
    room: Room,
    size: Vec2,
    serial: u32,
    light: f32,
) -> (Vec<Lot>, Vec3, Vec3) {
    let width = ((size.x / ROOM_TILE).round() as i32).clamp(ROOM_TILES[0], ROOM_TILES[1]);
    let depth = ((size.y / ROOM_TILE).round() as i32).clamp(ROOM_TILES[0], ROOM_TILES[1]);
    let origin = ROOM_SITE;
    let span = Vec2::new(width as f32 * ROOM_TILE, depth as f32 * ROOM_TILE);
    let mut lots = Vec::new();

    for column in 0..width {
        for row in 0..depth {
            let center = origin
                + Vec3::new(
                    (column as f32 + 0.5) * ROOM_TILE,
                    -ROOM_FLOOR,
                    (row as f32 + 0.5) * ROOM_TILE,
                );
            let floor = place_piece(commands, model(FLOOR), center, 0.0, BLOCK_SCALE);
            commands.set(floor, Indoor(serial));
        }
    }

    let door_column = width / 2;
    for row in 0..depth {
        let along = (row as f32 + 0.5) * ROOM_TILE;
        wall(
            commands,
            random,
            &mut lots,
            serial,
            origin + Vec3::new(0.0, 0.0, along),
            0.0,
            WALL,
        );
        wall(
            commands,
            random,
            &mut lots,
            serial,
            origin + Vec3::new(span.x, 0.0, along),
            0.0,
            WALL,
        );
    }
    for column in 0..width {
        let along = (column as f32 + 0.5) * ROOM_TILE;
        let front = if column == door_column {
            WALL_DOORWAY
        } else {
            WALL
        };
        wall(
            commands,
            random,
            &mut lots,
            serial,
            origin + Vec3::new(along, 0.0, 0.0),
            std::f32::consts::FRAC_PI_2,
            front,
        );
        wall(
            commands,
            random,
            &mut lots,
            serial,
            origin + Vec3::new(along, 0.0, span.y),
            std::f32::consts::FRAC_PI_2,
            WALL,
        );
    }
    let door_x = (door_column as f32 + 0.5) * ROOM_TILE;
    let leaf = place_piece(
        commands,
        model(DOOR_LEAF),
        origin + Vec3::new(door_x - 0.45 * BLOCK_SCALE, 0.0, 0.0),
        std::f32::consts::FRAC_PI_2,
        BLOCK_SCALE,
    );
    commands.set(leaf, Indoor(serial));
    for corner in [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(span.x, 0.0, 0.0),
        Vec3::new(0.0, 0.0, span.y),
        Vec3::new(span.x, 0.0, span.y),
    ] {
        let column = place_piece(commands, model(COLUMN), origin + corner, 0.0, BLOCK_SCALE);
        commands.set(column, Indoor(serial));
    }

    spawn_room_light(
        commands,
        origin + Vec3::new(span.x * 0.5, ROOM_LIGHT_HEIGHT, span.y * 0.5),
        light,
        span.x.max(span.y) * 2.0,
        serial,
    );

    furnish(commands, random, &mut lots, serial, origin, span, room);

    let exit = origin + Vec3::new(door_x, 0.0, DOOR_STEP);
    let entry = origin + Vec3::new(door_x, 0.0, ROOM_ENTRY);
    (lots, exit, entry)
}

fn wall(
    commands: &mut Commands,
    random: &mut Random,
    lots: &mut Vec<Lot>,
    serial: u32,
    position: Vec3,
    yaw: f32,
    name: &str,
) {
    let name = if name == WALL && chance(random, WINDOW_CHANCE) {
        WALL_WINDOW
    } else {
        name
    };
    let placement = Placement {
        model: model(name),
        position,
        yaw,
        scale: BLOCK_SCALE,
        prop: None,
    };
    lots.push(lot_of(&placement, name));
    let piece = place_piece(commands, placement.model, position, yaw, BLOCK_SCALE);
    commands.set(piece, Indoor(serial));
}

fn furnish(
    commands: &mut Commands,
    random: &mut Random,
    lots: &mut Vec<Lot>,
    serial: u32,
    origin: Vec3,
    span: Vec2,
    room: Room,
) {
    match room {
        Room::Shop => {
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                NORTH,
                &SHOP_COUNTER,
                [0.0, 0.0],
            );
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                WEST,
                &SHOP_SHELVES,
                [0.0, 0.3],
            );
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                EAST,
                &SHOP_SHELVES,
                [0.0, 0.3],
            );
            scatter(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                &SHOP_LOOSE,
                [3, 6],
            );
        }
        Room::Office => {
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                NORTH,
                &OFFICE_WALLS,
                [0.2, 0.8],
            );
            scatter(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                &OFFICE_LOOSE,
                [4, 8],
            );
        }
        Room::Warehouse => {
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                NORTH,
                &WAREHOUSE_WALLS,
                [0.0, 0.4],
            );
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                WEST,
                &WAREHOUSE_WALLS,
                [0.0, 0.6],
            );
            scatter(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                &WAREHOUSE_LOOSE,
                [4, 9],
            );
        }
        Room::Home => {
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                NORTH,
                &HOME_KITCHEN,
                [0.0, 0.0],
            );
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                EAST,
                &HOME_BEDROOM,
                [0.2, 0.6],
            );
            let rugs = count(random, 1, 2);
            for _ in 0..rugs {
                if let Some(rug) = pick(random, &HOME_RUGS) {
                    let scale = kit_scale(rug);
                    let spot = Vec2::new(
                        origin.x + between(random, span.x * 0.3, span.x * 0.7),
                        origin.z + between(random, span.y * 0.35, span.y * 0.65),
                    );
                    let yaw = whole(random, 0, 3) as f32 * std::f32::consts::FRAC_PI_2;
                    let piece = place_piece(
                        commands,
                        model(rug),
                        origin_for(rug, scale, spot, yaw),
                        yaw,
                        scale,
                    );
                    commands.set(piece, Indoor(serial));
                }
            }
            scatter(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                &HOME_LOOSE,
                [4, 7],
            );
        }
        Room::Lobby => {
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                WEST,
                &LOBBY_WALLS,
                [0.3, 1.0],
            );
            along_wall(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                EAST,
                &LOBBY_WALLS,
                [0.3, 1.0],
            );
            scatter(
                commands,
                random,
                lots,
                serial,
                origin,
                span,
                &LOBBY_LOOSE,
                [2, 5],
            );
        }
    }
}

fn along_wall(
    commands: &mut Commands,
    random: &mut Random,
    lots: &mut Vec<Lot>,
    serial: u32,
    origin: Vec3,
    span: Vec2,
    side: usize,
    models: &[&str],
    gap: [f32; 2],
) {
    let (start, along, inward, length) = match side {
        NORTH => (
            origin + Vec3::new(ROOM_MARGIN, 0.0, span.y),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            span.x - ROOM_MARGIN * 2.0,
        ),
        WEST => (
            origin + Vec3::new(0.0, 0.0, ROOM_CLEAR),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            span.y - ROOM_CLEAR - ROOM_MARGIN,
        ),
        _ => (
            origin + Vec3::new(span.x, 0.0, ROOM_CLEAR),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-1.0, 0.0, 0.0),
            span.y - ROOM_CLEAR - ROOM_MARGIN,
        ),
    };
    let yaw = yaw_of(inward);
    let mut cursor = 0.0;
    loop {
        let Some(name) = pick(random, models) else {
            return;
        };
        let scale = kit_scale(name);
        let size = extent(name, scale);
        if cursor + size.x > length {
            return;
        }
        let center =
            start + along * (cursor + size.x * 0.5) + inward * (size.z * 0.5 + ROOM_WALL_GAP);
        let position = origin_for(name, scale, Vec2::new(center.x, center.z), yaw);
        if !place_furniture(commands, lots, serial, name, position, yaw, 0.0) {
            return;
        }
        cursor += size.x + between(random, gap[0], gap[1]);
    }
}

fn scatter(
    commands: &mut Commands,
    random: &mut Random,
    lots: &mut Vec<Lot>,
    serial: u32,
    origin: Vec3,
    span: Vec2,
    models: &[&str],
    range: [usize; 2],
) {
    let wanted = count(random, range[0], range[1]);
    for _ in 0..wanted {
        let Some(name) = pick(random, models) else {
            return;
        };
        let scale = kit_scale(name);
        for _ in 0..PROP_TRIES {
            let spot = Vec2::new(
                origin.x + between(random, ROOM_MARGIN * 2.0, span.x - ROOM_MARGIN * 2.0),
                origin.z + between(random, ROOM_CLEAR, span.y - ROOM_MARGIN * 2.0),
            );
            let yaw = whole(random, 0, 3) as f32 * std::f32::consts::FRAC_PI_2;
            let position = origin_for(name, scale, spot, yaw);
            if place_furniture(commands, lots, serial, name, position, yaw, PROP_GAP) {
                break;
            }
        }
    }
}

fn place_furniture(
    commands: &mut Commands,
    lots: &mut Vec<Lot>,
    serial: u32,
    name: &str,
    position: Vec3,
    yaw: f32,
    gap: f32,
) -> bool {
    let scale = kit_scale(name);
    let placement = Placement {
        model: model(name),
        position,
        yaw,
        scale,
        prop: None,
    };
    let lot = lot_of(&placement, name);
    if lots.iter().any(|other| lots_overlap(&lot, other, gap)) {
        return false;
    }
    lots.push(lot);
    let piece = place_piece(commands, placement.model, position, yaw, scale);
    commands.set(piece, Indoor(serial));
    true
}
