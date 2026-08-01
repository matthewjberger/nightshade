use crate::commands::models::place_model;
use crate::commands::scenery::{prop, turned};
use crate::data::{FURNITURE_SCALE, HOUSE_CELL, Interior};
use crate::resources::Palette;
use crate::tags::Lamp;
use nightshade::prelude::*;
use nightshade_graphics::prelude::{Light, LightType};
use nightshade_random::prelude::*;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
const PI: f32 = std::f32::consts::PI;
const LEFT: f32 = FRAC_PI_2;
const RIGHT: f32 = -FRAC_PI_2;
const HEARTH: Vec3 = Vec3::new(1.0, 0.72, 0.42);
const CANDLE: Vec3 = Vec3::new(1.0, 0.85, 0.6);
const ARCANE: Vec3 = Vec3::new(0.7, 0.55, 1.0);

struct Room {
    spot: Vec2,
    yaw: f32,
    floor: f32,
    half: f32,
}

impl Room {
    fn world(&self, local: Vec2) -> Vec3 {
        let flat = self.spot + turned(local, self.yaw);
        Vec3::new(flat.x, self.floor, flat.y)
    }

    fn put(&self, commands: &mut Commands, name: &str, local: Vec2, turn: f32, scale: f32) {
        place_model(commands, name, self.world(local), self.yaw + turn, scale);
    }

    fn piece(&self, commands: &mut Commands, name: &str, local: Vec2, turn: f32) {
        self.put(commands, name, local, turn, FURNITURE_SCALE);
    }

    fn raised(
        &self,
        commands: &mut Commands,
        name: &str,
        local: Vec2,
        lift: f32,
        turn: f32,
        scale: f32,
    ) {
        let mut position = self.world(local);
        position.y += lift;
        place_model(commands, name, position, self.yaw + turn, scale);
    }
}

fn light(commands: &mut Commands, position: Vec3, color: Vec3, intensity: f32, range: f32) {
    let transform = LocalTransform {
        translation: position,
        ..Default::default()
    };
    commands.spawn((
        Name("Hearth Light".to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        Light {
            light_type: LightType::Point,
            color,
            intensity,
            range,
            ..Default::default()
        },
    ));
}

fn glow(commands: &mut Commands, palette: &Palette, position: Vec3, size: f32) {
    let ball = prop(
        commands,
        "Sphere",
        position,
        Vec3::new(size, size, size),
        palette.lantern,
    );
    commands.add_tag_type::<Lamp>(ball);
}

fn table_and_chairs(room: &Room, commands: &mut Commands, at: Vec2, cloth: bool) {
    let table = if cloth {
        "furn-tableCloth"
    } else {
        "furn-table"
    };
    room.piece(commands, table, at, 0.0);
    room.piece(commands, "furn-chair", at + Vec2::new(0.3, 0.5), PI);
    room.piece(commands, "furn-chair", at + Vec2::new(1.1, -0.8), 0.0);
    room.piece(commands, "furn-chairCushion", at + Vec2::new(1.5, 0.5), PI);
}

fn bed(room: &Room, commands: &mut Commands, at: Vec2, double: bool) {
    let piece = if double {
        "furn-bedDouble"
    } else {
        "furn-bedSingle"
    };
    room.put(commands, piece, at, 0.0, FURNITURE_SCALE * 0.85);
    room.piece(commands, "furn-cabinetBed", at + Vec2::new(2.4, 0.0), 0.0);
}

fn shelves(room: &Room, commands: &mut Commands, at: Vec2, turn: f32, count: usize, wide: bool) {
    let piece = if wide {
        "furn-bookcaseClosedWide"
    } else {
        "furn-bookcaseOpen"
    };
    let width = if wide { 1.36 } else { 0.68 };
    for index in 0..count {
        let step = turned(Vec2::new(width * index as f32, 0.0), turn);
        room.piece(commands, piece, at + step, turn);
        if !wide {
            room.raised(
                commands,
                "furn-books",
                at + step + turned(Vec2::new(0.1, -0.1), turn),
                0.9,
                turn,
                FURNITURE_SCALE,
            );
        }
    }
}

fn hearth(room: &Room, commands: &mut Commands, palette: &Palette, at: Vec2, size: f32) {
    room.put(commands, "fire-basket", at, 0.0, size);
    let position = room.world(at) + Vec3::new(0.0, 0.5, 0.0);
    glow(commands, palette, position, 0.2);
    light(commands, position, HEARTH, 9.0, 11.0);
}

fn home(room: &Room, commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let h = room.half;
    bed(room, commands, Vec2::new(-h + 0.1, -h + 2.9), random.bool());
    room.piece(commands, "furn-rugRound", Vec2::new(-0.8, 0.9), 0.0);
    table_and_chairs(room, commands, Vec2::new(-0.2, 0.2), random.bool());
    shelves(
        room,
        commands,
        Vec2::new(h - 0.45, -h + 1.4),
        LEFT,
        1,
        false,
    );
    room.piece(
        commands,
        "furn-coatRackStanding",
        Vec2::new(h - 0.4, h - 0.5),
        0.0,
    );
    room.piece(
        commands,
        "furn-pottedPlant",
        Vec2::new(-h + 0.3, h - 0.5),
        0.0,
    );
    room.piece(
        commands,
        "furn-sideTable",
        Vec2::new(h - 1.0, -h + 0.4),
        0.0,
    );
    room.raised(
        commands,
        "furn-lampRoundTable",
        Vec2::new(h - 0.8, -h + 0.3),
        0.65,
        0.0,
        FURNITURE_SCALE,
    );
    let lamp = room.world(Vec2::new(h - 0.7, -h + 0.25)) + Vec3::new(0.0, 1.3, 0.0);
    glow(commands, palette, lamp, 0.14);
    if random.f32() < 0.4 {
        room.piece(commands, "furn-bear", Vec2::new(-h + 1.2, -h + 3.2), PI);
    }
    if h > 2.6 {
        room.piece(
            commands,
            "furn-kitchenCabinet",
            Vec2::new(0.8, -h + 0.85),
            0.0,
        );
        room.piece(
            commands,
            "furn-kitchenStove",
            Vec2::new(1.6, -h + 0.85),
            0.0,
        );
        room.piece(commands, "furn-kitchenSink", Vec2::new(2.4, -h + 0.85), 0.0);
        hearth(room, commands, palette, Vec2::new(h - 0.9, 0.0), 2.2);
    }
}

fn inn(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    for index in 0..4 {
        room.piece(
            commands,
            "furn-kitchenBar",
            Vec2::new(-h + 0.3 + index as f32 * 0.73, -h + 1.3),
            0.0,
        );
    }
    room.piece(
        commands,
        "furn-kitchenBarEnd",
        Vec2::new(-h + 3.22, -h + 1.3),
        0.0,
    );
    for index in 0..3 {
        room.piece(
            commands,
            "furn-stoolBar",
            Vec2::new(-h + 0.5 + index as f32 * 0.9, -h + 1.9),
            0.0,
        );
    }
    shelves(
        room,
        commands,
        Vec2::new(-h + 0.2, -h + 0.45),
        0.0,
        3,
        false,
    );
    room.raised(
        commands,
        "food-mug",
        Vec2::new(-h + 1.0, -h + 1.0),
        0.72,
        0.0,
        1.8,
    );
    room.raised(
        commands,
        "food-loaf",
        Vec2::new(-h + 2.0, -h + 1.0),
        0.72,
        0.6,
        1.8,
    );
    room.raised(
        commands,
        "food-cheese",
        Vec2::new(-h + 2.7, -h + 1.05),
        0.72,
        0.0,
        1.6,
    );
    table_and_chairs(room, commands, Vec2::new(0.4, -h + 2.6), true);
    table_and_chairs(room, commands, Vec2::new(0.4, 0.4), false);
    table_and_chairs(room, commands, Vec2::new(-h + 0.4, 1.6), true);
    room.piece(commands, "furn-rugRectangle", Vec2::new(-1.4, h - 0.5), 0.0);
    room.put(commands, "barrel", Vec2::new(h - 0.6, -h + 0.6), 0.0, 2.4);
    room.put(
        commands,
        "barrel-open",
        Vec2::new(h - 0.6, -h + 1.5),
        0.4,
        2.4,
    );
    hearth(room, commands, palette, Vec2::new(h - 0.8, 1.0), 2.4);
    room.put(
        commands,
        "furn-stairsOpenSingle",
        Vec2::new(h - 0.2, h - 1.6),
        PI,
        HOUSE_CELL / 1.34,
    );
    room.raised(
        commands,
        "furn-bedSingle",
        Vec2::new(-h + 0.2, -h + 2.6),
        HOUSE_CELL,
        0.0,
        1.5,
    );
    room.raised(
        commands,
        "furn-bedSingle",
        Vec2::new(-h + 3.0, -h + 2.6),
        HOUSE_CELL,
        0.0,
        1.5,
    );
    room.raised(
        commands,
        "furn-bedBunk",
        Vec2::new(-h + 0.2, h - 0.4),
        HOUSE_CELL,
        0.0,
        1.5,
    );
    room.raised(
        commands,
        "furn-rugRound",
        Vec2::new(-0.4, 0.6),
        HOUSE_CELL,
        0.0,
        FURNITURE_SCALE,
    );
    room.raised(
        commands,
        "furn-lampSquareFloor",
        Vec2::new(0.8, -h + 0.5),
        HOUSE_CELL,
        0.0,
        FURNITURE_SCALE,
    );
}

fn auction(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    room.put(
        commands,
        "stairs-stone-round",
        Vec2::new(0.0, -h + 1.3),
        0.0,
        1.8,
    );
    room.raised(
        commands,
        "furn-desk",
        Vec2::new(-0.7, -h + 1.4),
        0.5,
        0.0,
        FURNITURE_SCALE,
    );
    room.raised(
        commands,
        "furn-chairDesk",
        Vec2::new(-0.2, -h + 0.5),
        0.5,
        0.0,
        FURNITURE_SCALE,
    );
    for row in 0..3 {
        for column in 0..3 {
            room.piece(
                commands,
                "furn-benchCushion",
                Vec2::new(-h + 0.6 + column as f32 * 1.1, -h + 3.0 + row as f32 * 0.9),
                0.0,
            );
        }
    }
    room.piece(
        commands,
        "furn-rugRectangle",
        Vec2::new(-1.5, -h + 2.6),
        0.0,
    );
    shelves(room, commands, Vec2::new(h - 0.45, -h + 0.4), LEFT, 3, true);
    for index in 0..3 {
        room.put(
            commands,
            "chest",
            Vec2::new(-h + 0.6, h - 0.8 - index as f32 * 0.9),
            RIGHT,
            2.0,
        );
    }
    room.put(commands, "box", Vec2::new(h - 0.8, h - 0.8), 0.3, 2.0);
    room.put(commands, "box-open", Vec2::new(h - 1.6, h - 0.7), 0.8, 2.0);
    room.raised(
        commands,
        "banner-green",
        Vec2::new(-h + 1.0, -h + 0.05),
        HOUSE_CELL * 1.1,
        PI - FRAC_PI_2,
        HOUSE_CELL,
    );
    room.raised(
        commands,
        "banner-green",
        Vec2::new(h - 1.0, -h + 0.05),
        HOUSE_CELL * 1.1,
        PI - FRAC_PI_2,
        HOUSE_CELL,
    );
    room.raised(commands, "coin", Vec2::new(-0.4, -h + 1.2), 1.1, 0.0, 1.8);
    let candle = room.world(Vec2::new(0.0, -h + 1.3)) + Vec3::new(0.0, 2.4, 0.0);
    light(commands, candle, CANDLE, 8.0, 12.0);
    glow(commands, palette, candle, 0.16);
}

fn bank(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    for index in 0..5 {
        room.piece(
            commands,
            "furn-kitchenBar",
            Vec2::new(-h + 0.5 + index as f32 * 0.73, 0.2),
            0.0,
        );
    }
    room.piece(
        commands,
        "furn-kitchenBarEnd",
        Vec2::new(-h + 4.15, 0.2),
        0.0,
    );
    room.piece(commands, "furn-desk", Vec2::new(-h + 0.6, -h + 1.4), 0.0);
    room.piece(commands, "furn-desk", Vec2::new(h - 2.2, -h + 1.4), 0.0);
    room.piece(
        commands,
        "furn-chairDesk",
        Vec2::new(-h + 1.2, -h + 0.5),
        0.0,
    );
    room.piece(
        commands,
        "furn-chairDesk",
        Vec2::new(h - 1.6, -h + 0.5),
        0.0,
    );
    shelves(room, commands, Vec2::new(-h + 0.2, -h + 0.45), 0.0, 2, true);
    for index in 0..4 {
        room.put(
            commands,
            "chest",
            Vec2::new(-h + 0.5 + index as f32 * 1.0, -h + 3.3),
            0.0,
            1.8,
        );
        room.raised(
            commands,
            "coin",
            Vec2::new(-h + 1.1 + index as f32 * 1.0, -h + 1.2),
            0.75,
            0.3 * index as f32,
            1.6,
        );
    }
    room.piece(commands, "furn-rugRectangle", Vec2::new(-1.5, h - 0.5), 0.0);
    room.piece(
        commands,
        "furn-pottedPlant",
        Vec2::new(h - 0.5, h - 0.5),
        0.0,
    );
    room.piece(
        commands,
        "furn-pottedPlant",
        Vec2::new(-h + 0.5, h - 0.5),
        0.0,
    );
    room.piece(
        commands,
        "furn-lampRoundFloor",
        Vec2::new(h - 0.4, 1.2),
        0.0,
    );
    let lamp = room.world(Vec2::new(h - 0.3, 1.1)) + Vec3::new(0.0, 1.5, 0.0);
    glow(commands, palette, lamp, 0.16);
    light(commands, lamp, CANDLE, 7.0, 11.0);
}

fn chapel(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    room.put(commands, "altar-stone", Vec2::new(0.0, -h + 1.4), 0.0, 2.6);
    room.raised(commands, "cross", Vec2::new(0.0, -h + 0.5), 1.6, 0.0, 2.4);
    room.raised(
        commands,
        "detail-chalice",
        Vec2::new(0.3, -h + 1.3),
        0.9,
        0.0,
        2.0,
    );
    for row in 0..4 {
        for side in [-1.0, 1.0] {
            room.piece(
                commands,
                "furn-bench",
                Vec2::new(side * 1.4 - 0.4, -h + 3.4 + row as f32 * 1.1),
                0.0,
            );
            room.piece(
                commands,
                "furn-bench",
                Vec2::new(side * 2.6 - 0.4, -h + 3.4 + row as f32 * 1.1),
                0.0,
            );
        }
    }
    room.piece(
        commands,
        "furn-rugRectangle",
        Vec2::new(-0.78, h - 0.6),
        LEFT,
    );
    room.piece(commands, "furn-rugRectangle", Vec2::new(-0.78, 0.4), LEFT);
    for side in [-1.0, 1.0] {
        let spot = Vec2::new(side * (h - 0.8), -h + 1.0);
        room.put(commands, "candle-multiple", spot, 0.0, 2.6);
        let position = room.world(spot) + Vec3::new(0.0, 0.6, 0.0);
        glow(commands, palette, position, 0.16);
        light(commands, position, CANDLE, 8.0, 13.0);
        room.put(
            commands,
            "pillar-large",
            Vec2::new(side * (h - 0.7), 0.5),
            0.0,
            2.4,
        );
        room.put(
            commands,
            "urn",
            Vec2::new(side * (h - 0.7), h - 1.0),
            0.0,
            2.0,
        );
    }
    room.put(
        commands,
        "lantern-candle",
        Vec2::new(0.0, h - 1.4),
        0.0,
        2.0,
    );
}

fn keep(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    room.put(
        commands,
        "stairs-stone-round",
        Vec2::new(0.0, -h + 1.4),
        0.0,
        2.4,
    );
    room.raised(
        commands,
        "furn-chairCushion",
        Vec2::new(-0.2, -h + 0.9),
        0.75,
        0.0,
        2.6,
    );
    room.raised(
        commands,
        "furn-chairCushion",
        Vec2::new(0.6, -h + 0.9),
        0.75,
        0.0,
        2.2,
    );
    room.put(commands, "furn-tableCross", Vec2::new(-1.8, 1.2), 0.0, 1.7);
    for index in 0..4 {
        room.piece(
            commands,
            "furn-chair",
            Vec2::new(-1.5 + index as f32 * 0.9, 1.7),
            PI,
        );
        room.piece(
            commands,
            "furn-chair",
            Vec2::new(-1.2 + index as f32 * 0.9, -1.0),
            0.0,
        );
    }
    room.raised(commands, "food-pie", Vec2::new(-0.9, 0.5), 0.87, 0.0, 1.8);
    room.raised(commands, "food-cheese", Vec2::new(0.4, 0.4), 0.87, 0.0, 1.6);
    room.raised(commands, "food-mug", Vec2::new(-0.2, 0.9), 0.87, 0.0, 1.6);
    room.piece(
        commands,
        "furn-rugRectangle",
        Vec2::new(-1.35, h - 0.4),
        LEFT,
    );
    room.piece(
        commands,
        "furn-rugRectangle",
        Vec2::new(-1.35, -h + 3.6),
        LEFT,
    );
    for side in [-1.0, 1.0] {
        hearth(
            room,
            commands,
            palette,
            Vec2::new(side * (h - 0.9), -h + 2.6),
            2.6,
        );
        room.put(
            commands,
            "weapon-rack",
            Vec2::new(side * (h - 0.6), 0.6),
            if side < 0.0 { RIGHT } else { LEFT },
            2.2,
        );
        room.raised(
            commands,
            "banner-red",
            Vec2::new(side * 2.0, -h + 0.05),
            HOUSE_CELL * 1.2,
            PI - FRAC_PI_2,
            HOUSE_CELL,
        );
        room.put(
            commands,
            "arena-statue",
            Vec2::new(side * (h - 0.9), h - 0.9),
            PI,
            1.8,
        );
    }
    shelves(room, commands, Vec2::new(-h + 0.2, -h + 0.45), 0.0, 2, true);
    room.put(commands, "chest", Vec2::new(h - 1.6, -h + 0.6), 0.0, 2.0);
    room.put(commands, "flag-wide", Vec2::new(0.0, h - 0.7), PI, 1.6);
}

fn smithy(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    room.put(
        commands,
        "workbench-anvil",
        Vec2::new(0.4, -h + 1.4),
        0.0,
        2.4,
    );
    hearth(room, commands, palette, Vec2::new(h - 1.0, -h + 1.0), 3.0);
    room.put(
        commands,
        "workbench",
        Vec2::new(-h + 0.8, -h + 1.0),
        0.0,
        2.4,
    );
    room.put(
        commands,
        "weapon-rack",
        Vec2::new(-h + 0.4, 0.6),
        RIGHT,
        2.2,
    );
    room.put(
        commands,
        "weapon-rack",
        Vec2::new(-h + 0.4, 1.8),
        RIGHT,
        2.2,
    );
    room.put(commands, "barrel", Vec2::new(h - 0.6, 0.8), 0.0, 2.4);
    room.put(commands, "bucket", Vec2::new(h - 0.6, 1.8), 0.0, 2.4);
    room.put(commands, "resource-stone", Vec2::new(-0.8, 0.6), 0.4, 2.0);
    room.put(
        commands,
        "resource-planks",
        Vec2::new(h - 1.2, h - 0.8),
        0.4,
        2.0,
    );
    room.put(
        commands,
        "tool-pickaxe",
        Vec2::new(-h + 1.4, h - 0.6),
        1.0,
        2.4,
    );
    room.put(commands, "tool-axe", Vec2::new(-h + 2.0, h - 0.7), 2.0, 2.4);
    room.piece(commands, "furn-table", Vec2::new(-h + 0.3, 1.2), 0.0);
    room.raised(
        commands,
        "weapon-sword",
        Vec2::new(-h + 0.6, 1.0),
        0.6,
        0.4,
        2.0,
    );
    room.raised(
        commands,
        "weapon-spear",
        Vec2::new(-h + 1.2, 0.9),
        0.6,
        -0.3,
        2.0,
    );
}

fn apothecary(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    shelves(
        room,
        commands,
        Vec2::new(-h + 0.2, -h + 0.45),
        0.0,
        3,
        false,
    );
    room.piece(commands, "furn-desk", Vec2::new(-h + 0.3, h - 1.2), PI + PI);
    room.piece(commands, "furn-chairDesk", Vec2::new(-h + 0.9, -0.4), PI);
    room.raised(
        commands,
        "bottle",
        Vec2::new(-h + 0.6, h - 1.4),
        0.65,
        0.0,
        1.8,
    );
    room.raised(
        commands,
        "bottle-large",
        Vec2::new(-h + 1.2, h - 1.5),
        0.65,
        0.0,
        1.8,
    );
    room.raised(
        commands,
        "food-honey",
        Vec2::new(-h + 0.9, h - 1.9),
        0.65,
        0.0,
        1.6,
    );
    room.put(
        commands,
        "pot_large",
        Vec2::new(h - 0.6, -h + 0.6),
        0.0,
        2.0,
    );
    room.put(
        commands,
        "pot_small",
        Vec2::new(h - 0.5, -h + 1.4),
        0.0,
        2.0,
    );
    room.put(
        commands,
        "mushroom_redGroup",
        Vec2::new(h - 0.6, 0.4),
        0.0,
        1.8,
    );
    room.piece(
        commands,
        "furn-pottedPlant",
        Vec2::new(h - 0.5, h - 0.5),
        0.0,
    );
    room.piece(
        commands,
        "furn-plantSmall2",
        Vec2::new(-h + 2.4, -h + 0.5),
        0.0,
    );
    room.piece(commands, "furn-rugRound", Vec2::new(-0.6, 1.0), 0.0);
    let lamp = room.world(Vec2::new(0.0, -0.4)) + Vec3::new(0.0, 2.1, 0.0);
    glow(commands, palette, lamp, 0.16);
    light(commands, lamp, ARCANE, 7.0, 10.0);
}

fn guard_house(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    room.put(
        commands,
        "furn-bedBunk",
        Vec2::new(-h + 0.1, -h + 2.9),
        0.0,
        FURNITURE_SCALE * 0.85,
    );
    room.put(
        commands,
        "furn-bedBunk",
        Vec2::new(h - 2.5, -h + 2.9),
        0.0,
        FURNITURE_SCALE * 0.85,
    );
    room.put(
        commands,
        "weapon-rack",
        Vec2::new(-h + 0.4, h - 1.2),
        RIGHT,
        2.2,
    );
    room.put(
        commands,
        "weapon-rack",
        Vec2::new(h - 0.4, h - 1.2),
        LEFT,
        2.2,
    );
    table_and_chairs(room, commands, Vec2::new(-0.6, 0.8), false);
    room.put(commands, "chest", Vec2::new(0.0, -h + 0.6), 0.0, 2.0);
    room.put(
        commands,
        "arena-banner",
        Vec2::new(-h + 1.2, h - 0.5),
        0.0,
        2.2,
    );
    hearth(room, commands, palette, Vec2::new(h - 0.9, 0.2), 2.2);
}

fn cabin(room: &Room, commands: &mut Commands, palette: &Palette) {
    let h = room.half;
    bed(room, commands, Vec2::new(-h + 0.1, -h + 2.9), true);
    table_and_chairs(room, commands, Vec2::new(0.0, 0.6), true);
    room.put(commands, "chest", Vec2::new(h - 0.7, -h + 0.6), 0.0, 2.2);
    room.put(
        commands,
        "pirate-chest",
        Vec2::new(h - 0.7, -h + 1.6),
        0.4,
        1.4,
    );
    room.put(
        commands,
        "weapon-rack",
        Vec2::new(h - 0.4, h - 1.0),
        LEFT,
        2.2,
    );
    room.put(
        commands,
        "arena-banner",
        Vec2::new(-h + 0.6, h - 0.5),
        0.0,
        2.2,
    );
    room.raised(commands, "coin", Vec2::new(0.6, 0.3), 0.7, 0.0, 1.6);
    hearth(room, commands, palette, Vec2::new(0.9, -h + 0.8), 2.0);
}

fn warehouse(room: &Room, commands: &mut Commands, random: &mut Random) {
    let h = room.half;
    for _ in 0..7 {
        let local = Vec2::new(
            between(random, -h + 0.5, h - 0.5),
            between(random, -h + 0.5, h - 1.4),
        );
        let piece = [
            "pirate-crate",
            "pirate-barrel",
            "pirate-crate-bottles",
            "food-bag",
            "box-large",
        ][random.usize(0..5)];
        room.put(commands, piece, local, random.f32() * PI, 1.7);
    }
    room.put(
        commands,
        "furn-desk",
        Vec2::new(-h + 0.3, h - 1.2),
        0.0,
        FURNITURE_SCALE,
    );
    room.raised(
        commands,
        "food-fish",
        Vec2::new(-h + 0.6, h - 1.4),
        0.65,
        0.2,
        2.0,
    );
}

pub fn furnish(
    commands: &mut Commands,
    palette: &Palette,
    random: &mut Random,
    spot: Vec2,
    yaw: f32,
    cells: usize,
    floor: f32,
    interior: Interior,
) {
    let room = Room {
        spot,
        yaw,
        floor: floor + 0.06,
        half: cells as f32 * HOUSE_CELL / 2.0 - 0.35,
    };
    match interior {
        Interior::Home => home(&room, commands, palette, random),
        Interior::Inn => inn(&room, commands, palette),
        Interior::Auction => auction(&room, commands, palette),
        Interior::Bank => bank(&room, commands, palette),
        Interior::Chapel => chapel(&room, commands, palette),
        Interior::Keep => keep(&room, commands, palette),
        Interior::Smithy => smithy(&room, commands, palette),
        Interior::Apothecary => apothecary(&room, commands, palette),
        Interior::Guard => guard_house(&room, commands, palette),
        Interior::Cabin => cabin(&room, commands, palette),
        Interior::Warehouse => warehouse(&room, commands, random),
    }
}
