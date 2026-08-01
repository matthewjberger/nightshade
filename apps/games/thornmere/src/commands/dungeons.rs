use crate::commands::block;
use crate::commands::effects::flame;
use crate::commands::models::{place_model, plant_model};
use crate::commands::scenery::{house, prop};
use crate::data::*;
use crate::queries::{ground, terrain_height};
use crate::resources::Palette;
use crate::tags::Lamp;
use nightshade::prelude::*;
use nightshade_random::prelude::*;

const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
const PI: f32 = std::f32::consts::PI;
const TAU: f32 = std::f32::consts::TAU;
const CELL: f32 = 4.0;

fn brazier(commands: &mut Commands, palette: &Palette, spot: Vec2, scale: f32) {
    plant_model(commands, "fire-basket", spot, 0.0, scale);
    let glow = prop(
        commands,
        "Sphere",
        ground(spot) + Vec3::new(0.0, 0.2 * scale, 0.0),
        Vec3::new(0.3, 0.2, 0.3),
        palette.fire,
    );
    commands.add_tag_type::<Lamp>(glow);
    flame(
        commands,
        ground(spot) + Vec3::new(0.0, 0.2 * scale, 0.0),
        0.8,
    );
}

fn candle(commands: &mut Commands, palette: &Palette, spot: Vec2) {
    plant_model(commands, "candle-multiple", spot, 0.0, 2.4);
    let glow = prop(
        commands,
        "Sphere",
        ground(spot) + Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.16, 0.16, 0.16),
        palette.lantern,
    );
    commands.add_tag_type::<Lamp>(glow);
}

fn pit_floor(commands: &mut Commands, pit: &Pit, tile: &str) {
    let mut x = pit.min.x + CELL / 2.0;
    while x < pit.max.x {
        let mut z = pit.min.y + CELL / 2.0;
        while z < pit.max.y {
            let position = Vec3::new(x, terrain_height(x, z) + 0.04, z);
            place_model(commands, tile, position, 0.0, CELL);
            z += CELL;
        }
        x += CELL;
    }
}

fn pit_rim(
    commands: &mut Commands,
    pit: &Pit,
    piece: &str,
    scale: f32,
    inset: f32,
    skip: impl Fn(Vec2) -> bool,
) {
    let low = pit.min + Vec2::new(inset, inset);
    let high = pit.max - Vec2::new(inset, inset);
    let mut x = low.x + scale / 2.0;
    while x < high.x {
        for (z, yaw) in [(low.y, 0.0), (high.y, PI)] {
            let spot = Vec2::new(x, z);
            if !skip(spot) {
                plant_model(commands, piece, spot, yaw, scale);
            }
        }
        x += scale;
    }
    let mut z = low.y + scale / 2.0;
    while z < high.y {
        for (x, yaw) in [(low.x, FRAC_PI_2), (high.x, -FRAC_PI_2)] {
            let spot = Vec2::new(x, z);
            if !skip(spot) {
                plant_model(commands, piece, spot, yaw, scale);
            }
        }
        z += scale;
    }
}

pub fn spawn_crypt(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let pit = &PITS[0];
    let center = (pit.min + pit.max) / 2.0;
    pit_floor(commands, pit, "dungeon-floor-detail");
    let door = Vec2::new(pit.max.x - 1.5, 84.0);
    pit_rim(commands, pit, "dungeon-wall", CELL, 1.5, |spot| {
        (spot - door).magnitude() < 3.4
    });
    let low = pit.min + Vec2::new(1.5, 1.5);
    let high = pit.max - Vec2::new(1.5, 1.5);
    block(commands, low, Vec2::new(high.x, low.y));
    block(commands, Vec2::new(low.x, high.y), high);
    block(commands, low, Vec2::new(low.x, high.y));
    block(
        commands,
        Vec2::new(high.x, low.y),
        Vec2::new(high.x, door.y - 3.4),
    );
    block(commands, Vec2::new(high.x, door.y + 3.4), high);
    for corner in [
        Vec2::new(pit.min.x + 1.5, pit.min.y + 1.5),
        Vec2::new(pit.max.x - 1.5, pit.min.y + 1.5),
        Vec2::new(pit.min.x + 1.5, pit.max.y - 1.5),
        Vec2::new(pit.max.x - 1.5, pit.max.y - 1.5),
    ] {
        plant_model(commands, "column", corner, 0.0, 4.0);
    }
    plant_model(
        commands,
        "dungeon-stairs",
        door + Vec2::new(1.6, 0.0),
        FRAC_PI_2,
        CELL,
    );
    plant_model(
        commands,
        "dungeon-stairs",
        door + Vec2::new(4.4, 0.0),
        FRAC_PI_2,
        CELL,
    );
    for side in [-1.0, 1.0] {
        plant_model(
            commands,
            "gate",
            door + Vec2::new(0.4, side * 1.4),
            FRAC_PI_2 + side * 0.9,
            4.0,
        );
        plant_model(
            commands,
            "pillar-obelisk",
            door + Vec2::new(6.4, side * 3.2),
            0.0,
            2.8,
        );
        brazier(commands, palette, door + Vec2::new(-2.4, side * 3.0), 2.8);
    }

    let throne = Vec2::new(pit.min.x + 6.0, center.y);
    plant_model(commands, "crypt-large", throne, FRAC_PI_2, 3.2);
    place_model(
        commands,
        "crypt-large-roof",
        ground(throne) + Vec3::new(0.0, 3.2, 0.0),
        FRAC_PI_2,
        3.2,
    );
    plant_model(commands, "altar-stone", center, 0.0, 3.0);
    plant_model(
        commands,
        "detail-chalice",
        center + Vec2::new(0.6, 0.0),
        0.0,
        2.4,
    );
    for (offset, yaw) in [
        (Vec2::new(-4.0, -6.0), 0.3),
        (Vec2::new(4.0, -6.0), -0.4),
        (Vec2::new(-4.0, 6.0), 2.9),
        (Vec2::new(4.0, 6.0), 3.4),
    ] {
        plant_model(commands, "coffin", center + offset, yaw, 2.6);
    }
    for offset in [
        Vec2::new(-8.0, -8.5),
        Vec2::new(-8.0, 8.5),
        Vec2::new(8.0, -8.5),
    ] {
        plant_model(
            commands,
            "coffin-old",
            center + offset,
            random.f32() * TAU,
            2.6,
        );
    }
    for index in 0..4 {
        let angle = index as f32 / 4.0 * TAU + 0.6;
        let spot = center + Vec2::new(angle.cos() * 5.6, angle.sin() * 5.6);
        plant_model(commands, "pillar-large", spot, angle, 2.8);
        candle(
            commands,
            palette,
            spot + Vec2::new(angle.cos(), angle.sin()) * 1.4,
        );
    }
    for corner in [
        Vec2::new(pit.min.x + 4.0, pit.min.y + 4.0),
        Vec2::new(pit.max.x - 4.0, pit.min.y + 4.0),
        Vec2::new(pit.min.x + 4.0, pit.max.y - 4.0),
        Vec2::new(pit.max.x - 4.0, pit.max.y - 4.0),
    ] {
        brazier(commands, palette, corner, 3.0);
    }
    for index in 0..6 {
        let x = pit.min.x + 4.0 + index as f32 * 3.2;
        place_model(
            commands,
            "banner",
            ground(Vec2::new(x, pit.max.y - 1.5)) + Vec3::new(0.0, 1.6, 0.0),
            PI,
            3.0,
        );
    }
    for _ in 0..14 {
        let spot = center + Vec2::new(between(random, -9.0, 9.0), between(random, -9.0, 9.0));
        if (spot - center).magnitude() < 3.0 || (spot - throne).magnitude() < 4.5 {
            continue;
        }
        let piece = [
            "debris",
            "debris-wood",
            "urn",
            "rocks",
            "bench-damaged",
            "cross-wood",
        ][random.usize(0..6)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.2);
    }
    plant_model(
        commands,
        "chest",
        throne + Vec2::new(0.0, 4.6),
        FRAC_PI_2,
        2.6,
    );
    plant_model(
        commands,
        "chest",
        throne + Vec2::new(0.0, -4.6),
        -FRAC_PI_2,
        2.6,
    );
    for index in 0..8 {
        let angle = index as f32 / 8.0 * TAU;
        let spot = PLACES[BARROW].center + Vec2::new(angle.cos(), angle.sin()) * 16.0;
        let piece = if index % 2 == 0 {
            "pillar-obelisk"
        } else {
            "pillar-square"
        };
        plant_model(commands, piece, spot, angle, between(random, 2.4, 3.2));
    }
}

pub fn spawn_mine(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let pit = &PITS[1];
    let center = (pit.min + pit.max) / 2.0;
    let entrance = Vec2::new(pit.min.x + 4.0, pit.min.y);
    pit_rim(commands, pit, "retro-wall-fortified", 4.0, -2.0, |spot| {
        (spot - Vec2::new(entrance.x, pit.min.y - 2.0)).magnitude() < 4.0
    });
    let low = pit.min - Vec2::new(2.0, 2.0);
    let high = pit.max + Vec2::new(2.0, 2.0);
    block(commands, Vec2::new(low.x, high.y), high);
    block(commands, low, Vec2::new(low.x, high.y));
    block(commands, Vec2::new(high.x, low.y), high);
    block(commands, low, Vec2::new(entrance.x - 4.0, low.y));
    block(
        commands,
        Vec2::new(entrance.x + 4.0, low.y),
        Vec2::new(high.x, low.y),
    );
    for side in [-1.0, 1.0] {
        plant_model(
            commands,
            "retro-wall-fortified-gate",
            Vec2::new(entrance.x + side * 2.0, pit.min.y - 2.0),
            0.0,
            4.0,
        );
    }
    let headframe = Vec2::new(center.x, pit.min.y - 4.0);
    let foot = ground(headframe);
    plant_model(commands, "retro-tower-base", headframe, 0.0, 3.4);
    for level in 1..4 {
        place_model(
            commands,
            "retro-tower",
            foot + Vec3::new(0.0, level as f32 * 3.4, 0.0),
            0.0,
            3.4,
        );
    }
    place_model(
        commands,
        "retro-tower-top",
        foot + Vec3::new(0.0, 4.0 * 3.4, 0.0),
        0.0,
        3.4,
    );
    place_model(
        commands,
        "retro-pulley-crate",
        foot + Vec3::new(0.0, 4.0 * 3.4 + 0.2, 2.6),
        0.0,
        3.0,
    );
    plant_model(
        commands,
        "castle-flag-pennant",
        headframe + Vec2::new(2.4, 0.0),
        0.0,
        3.2,
    );

    let mut z = entrance.y + 1.0;
    let mut rail_index = 0;
    while z < center.y + 4.0 {
        let spot = Vec2::new(entrance.x, z);
        plant_model(
            commands,
            "holiday-trainset-rail-straight",
            spot,
            FRAC_PI_2,
            4.0,
        );
        if rail_index == 3 || rail_index == 7 {
            plant_model(commands, "holiday-train-wagon-flat", spot, FRAC_PI_2, 4.0);
            place_model(
                commands,
                "resource-stone",
                ground(spot) + Vec3::new(0.0, 0.7, 0.0),
                0.3,
                1.6,
            );
        }
        z += 2.0;
        rail_index += 1;
    }
    let mut x = entrance.x;
    while x < center.x + 6.0 {
        plant_model(
            commands,
            "holiday-trainset-rail-straight",
            Vec2::new(x, center.y + 4.0),
            0.0,
            4.0,
        );
        x += 2.0;
    }
    plant_model(
        commands,
        "cart",
        Vec2::new(center.x + 7.0, center.y + 4.0),
        FRAC_PI_2,
        2.4,
    );

    for offset in [
        Vec2::new(-8.0, 6.0),
        Vec2::new(8.0, -7.0),
        Vec2::new(-9.0, -6.0),
        Vec2::new(7.0, 8.0),
    ] {
        let spot = center + offset;
        plant_model(commands, "wood-structure", spot, random.f32() * TAU, 3.0);
        plant_model(
            commands,
            "wood-support",
            spot + Vec2::new(2.2, 0.0),
            random.f32() * TAU,
            3.0,
        );
    }
    for offset in [Vec2::new(-4.0, 9.5), Vec2::new(9.5, 2.0)] {
        plant_model(commands, "structure", center + offset, 0.0, 4.0);
    }
    for (offset, yaw) in [
        (Vec2::new(-11.0, 0.0), FRAC_PI_2),
        (Vec2::new(11.0, 4.0), -FRAC_PI_2),
        (Vec2::new(2.0, 11.0), PI),
    ] {
        plant_model(commands, "retro-ladder", center + offset, yaw, 4.0);
    }
    for _ in 0..8 {
        let spot = center + Vec2::new(between(random, -10.0, 10.0), between(random, -10.0, 10.0));
        let piece = ["detail-crystal-large", "detail-crystal"][random.usize(0..2)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.6);
        let glow = prop(
            commands,
            "Sphere",
            ground(spot) + Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.24, 0.24, 0.24),
            palette.ice,
        );
        commands.add_tag_type::<Lamp>(glow);
    }
    for _ in 0..10 {
        let spot = center + Vec2::new(between(random, -10.5, 10.5), between(random, -10.5, 10.5));
        let piece = [
            "detail-rocks-large",
            "detail-rocks",
            "rocks",
            "stones",
            "retro-barrels",
            "resource-stone",
            "tool-pickaxe",
            "tool-shovel",
        ][random.usize(0..8)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.4);
    }
    for offset in [
        Vec2::new(-9.0, -9.0),
        Vec2::new(9.0, -9.0),
        Vec2::new(-9.0, 9.0),
        Vec2::new(9.0, 9.0),
        Vec2::new(0.0, -2.0),
    ] {
        brazier(commands, palette, center + offset, 2.6);
    }
    plant_model(commands, "tent", center + Vec2::new(-6.0, -2.0), 1.2, 3.8);
    plant_model(
        commands,
        "tent-canvas",
        center + Vec2::new(6.0, -3.0),
        -1.4,
        3.8,
    );
    plant_model(
        commands,
        "workbench",
        center + Vec2::new(-2.0, 7.0),
        0.4,
        2.6,
    );
    let deep = PLACES[DEEP].center;
    plant_model(commands, "spawn-round", deep, 0.0, 6.0);
    for offset in [
        Vec2::new(-3.0, 2.0),
        Vec2::new(3.0, 2.2),
        Vec2::new(0.0, 3.6),
    ] {
        plant_model(commands, "chest", deep + offset, PI, 2.6);
    }
    place_model(
        commands,
        "banner",
        ground(deep + Vec2::new(0.0, 5.0)) + Vec3::new(0.0, 1.6, 0.0),
        PI,
        3.2,
    );
    for _ in 0..12 {
        let spot = PLACES[MINE].center
            + Vec2::new(between(random, -22.0, 22.0), between(random, -22.0, 22.0));
        if spot.x > pit.min.x - 4.0
            && spot.x < pit.max.x + 4.0
            && spot.y > pit.min.y - 6.0
            && spot.y < pit.max.y + 4.0
        {
            continue;
        }
        let piece = [
            "rock_largeC",
            "rock_tallB",
            "stone_largeB",
            "tree_pineDefaultA",
        ][random.usize(0..4)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.6);
    }
}

pub fn spawn_fort(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let fort = &PLACES[FORT];
    let half = 14.0;
    let low = fort.center - Vec2::new(half, half);
    let high = fort.center + Vec2::new(half, half);
    let scale = 4.0;
    let gate = Vec2::new(low.x, fort.center.y);
    block(commands, low, Vec2::new(high.x, low.y));
    block(commands, Vec2::new(low.x, high.y), high);
    block(commands, Vec2::new(high.x, low.y), high);
    block(commands, low, Vec2::new(low.x, gate.y - scale));
    block(
        commands,
        Vec2::new(low.x, gate.y + scale),
        Vec2::new(low.x, high.y),
    );
    let mut x = low.x + scale / 2.0;
    while x < high.x {
        plant_model(
            commands,
            "retro-wall-fortified",
            Vec2::new(x, low.y),
            0.0,
            scale,
        );
        plant_model(
            commands,
            "retro-wall-fortified",
            Vec2::new(x, high.y),
            PI,
            scale,
        );
        x += scale;
    }
    let mut z = low.y + scale / 2.0;
    while z < high.y {
        plant_model(
            commands,
            "retro-wall-fortified",
            Vec2::new(high.x, z),
            -FRAC_PI_2,
            scale,
        );
        if (z - gate.y).abs() > scale {
            plant_model(
                commands,
                "retro-wall-fortified",
                Vec2::new(low.x, z),
                FRAC_PI_2,
                scale,
            );
        } else {
            plant_model(
                commands,
                "retro-wall-fortified-gate",
                Vec2::new(low.x, z),
                FRAC_PI_2,
                scale,
            );
        }
        z += scale;
    }
    for corner in [
        low,
        high,
        Vec2::new(low.x, high.y),
        Vec2::new(high.x, low.y),
    ] {
        let foot = ground(corner);
        plant_model(commands, "retro-tower-base", corner, 0.0, scale);
        place_model(
            commands,
            "retro-tower",
            foot + Vec3::new(0.0, scale, 0.0),
            0.0,
            scale,
        );
        place_model(
            commands,
            "retro-tower-top",
            foot + Vec3::new(0.0, 2.0 * scale, 0.0),
            0.0,
            scale,
        );
        place_model(
            commands,
            "castle-flag-pennant",
            foot + Vec3::new(0.0, 2.3 * scale, 0.0),
            0.0,
            2.6,
        );
    }

    let cabin = fort.center + Vec2::new(4.0, -2.0);
    house(
        commands,
        palette,
        random,
        cabin,
        -FRAC_PI_2,
        2,
        1,
        true,
        "roof-point",
        Interior::Cabin,
    );
    place_model(
        commands,
        "banner",
        ground(cabin + Vec2::new(-2.6, 0.0)) + Vec3::new(0.0, 1.8, 0.0),
        -FRAC_PI_2,
        3.0,
    );
    brazier(commands, palette, fort.center + Vec2::new(-4.0, 0.0), 3.2);
    plant_model(
        commands,
        "campfire-stand",
        fort.center + Vec2::new(-4.0, 0.0),
        0.4,
        3.2,
    );
    for index in 0..4 {
        let angle = index as f32 / 4.0 * TAU + 0.8;
        let spot = fort.center + Vec2::new(angle.cos(), angle.sin()) * 8.0;
        let tent = if index % 2 == 0 {
            "tent"
        } else {
            "tent-canvas"
        };
        plant_model(commands, tent, spot, angle + PI, 3.8);
        plant_model(
            commands,
            "bedroll",
            fort.center + Vec2::new(angle.cos(), angle.sin()) * 5.0,
            angle,
            3.0,
        );
    }
    for index in 0..4 {
        let spot = fort.center + Vec2::new(-9.0 + index as f32 * 1.6, 8.5);
        let piece = ["box", "retro-detail-crate", "retro-barrels", "box-open"][index];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.8);
    }
    plant_model(
        commands,
        "chest",
        fort.center + Vec2::new(8.0, 8.0),
        0.3,
        3.0,
    );
    plant_model(
        commands,
        "pirate-chest",
        fort.center + Vec2::new(6.0, 9.0),
        1.1,
        1.6,
    );
    plant_model(
        commands,
        "siege-catapult",
        fort.center + Vec2::new(-7.0, -7.0),
        0.9,
        2.2,
    );
    plant_model(
        commands,
        "castle-siege-ballista",
        fort.center + Vec2::new(7.0, -8.0),
        -1.8,
        2.0,
    );
    plant_model(
        commands,
        "castle-siege-trebuchet",
        fort.center + Vec2::new(18.0, 4.0),
        FRAC_PI_2,
        1.8,
    );
    plant_model(
        commands,
        "weapon-rack",
        fort.center + Vec2::new(0.0, 9.5),
        PI,
        2.4,
    );
    plant_model(
        commands,
        "weapon-rack",
        fort.center + Vec2::new(-2.6, 9.5),
        PI,
        2.4,
    );
    plant_model(
        commands,
        "wood-support",
        fort.center + Vec2::new(9.0, -3.0),
        0.0,
        3.0,
    );
    plant_model(
        commands,
        "arena-banner",
        fort.center + Vec2::new(9.0, -3.0),
        0.0,
        3.0,
    );
    plant_model(
        commands,
        "signpost",
        gate + Vec2::new(-3.0, -4.0),
        FRAC_PI_2,
        3.2,
    );
    for side in [-1.0, 1.0] {
        brazier(commands, palette, gate + Vec2::new(-2.0, side * 4.6), 2.6);
    }
    for _ in 0..6 {
        let spot =
            fort.center + Vec2::new(between(random, -10.0, 10.0), between(random, -10.0, 10.0));
        if (spot - cabin).magnitude() < 4.0 || (spot - fort.center).magnitude() < 3.0 {
            continue;
        }
        let piece = [
            "barrel",
            "bucket",
            "bottle-large",
            "tree-log-small",
            "hay-bale",
        ][random.usize(0..5)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.6);
    }
}

pub fn spawn_harbor(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let shore = PLACES[HARBOR].center;
    let dock_scale = 1.6;
    let deck = WATER_LEVEL - 1.3;
    let mut x = shore.x - 5.0;
    let mut index = 0;
    while x > shore.x - 19.0 {
        let piece = if index == 0 {
            "pirate-structure-platform"
        } else {
            "pirate-structure-platform-dock"
        };
        place_model(
            commands,
            piece,
            Vec3::new(x, deck, shore.y + 2.0),
            0.0,
            dock_scale,
        );
        x -= 2.5 * dock_scale;
        index += 1;
    }
    for z in [shore.y + 6.0, shore.y + 10.0] {
        place_model(
            commands,
            "pirate-structure-platform-dock",
            Vec3::new(shore.x - 17.0, deck, z),
            FRAC_PI_2,
            dock_scale,
        );
    }
    place_model(
        commands,
        "pirate-structure-roof",
        Vec3::new(shore.x - 9.0, deck + 1.31 * dock_scale, shore.y + 2.0),
        0.0,
        dock_scale,
    );
    place_model(
        commands,
        "pirate-ship-small",
        Vec3::new(shore.x - 24.0, WATER_LEVEL - 0.5, shore.y + 8.0),
        0.5,
        0.75,
    );
    place_model(
        commands,
        "pirate-boat-row-small",
        Vec3::new(shore.x - 8.0, WATER_LEVEL, shore.y - 3.5),
        0.7,
        0.9,
    );
    place_model(
        commands,
        "craft-boat-fishing-small",
        Vec3::new(shore.x - 14.0, WATER_LEVEL - 0.2, shore.y - 5.0),
        2.4,
        1.0,
    );
    place_model(
        commands,
        "craft-boat-sail-a",
        Vec3::new(shore.x - 20.0, WATER_LEVEL - 0.2, shore.y + 16.0),
        1.1,
        1.1,
    );
    for offset in [
        Vec2::new(-28.0, -6.0),
        Vec2::new(-32.0, 12.0),
        Vec2::new(-24.0, 20.0),
    ] {
        place_model(
            commands,
            "craft-buoy",
            Vec3::new(shore.x + offset.x, WATER_LEVEL + 0.3, shore.y + offset.y),
            0.0,
            1.2,
        );
    }
    let crates = [
        (Vec2::new(-1.0, 4.5), "pirate-crate"),
        (Vec2::new(0.6, 5.2), "pirate-crate-bottles"),
        (Vec2::new(-2.4, 6.0), "pirate-barrel"),
        (Vec2::new(2.0, -1.5), "pirate-barrel"),
        (Vec2::new(3.6, -0.5), "pirate-crate"),
        (Vec2::new(-3.0, -2.6), "pirate-chest"),
    ];
    for (offset, piece) in crates {
        plant_model(commands, piece, shore + offset, random.f32() * TAU, 1.6);
    }
    for offset in [Vec2::new(-1.0, 4.5), Vec2::new(3.6, -0.5)] {
        place_model(
            commands,
            "food-fish",
            ground(shore + offset) + Vec3::new(0.0, 1.25, 0.0),
            random.f32() * TAU,
            2.2,
        );
    }
    plant_model(
        commands,
        "pirate-cannon",
        shore + Vec2::new(-3.0, 8.5),
        -FRAC_PI_2,
        1.4,
    );
    plant_model(
        commands,
        "pirate-tower-watch",
        shore + Vec2::new(4.0, -6.0),
        0.0,
        1.3,
    );
    plant_model(
        commands,
        "pirate-flag-high",
        shore + Vec2::new(-4.5, -0.5),
        0.0,
        1.6,
    );
    house(
        commands,
        palette,
        random,
        shore + Vec2::new(7.0, 5.0),
        -FRAC_PI_2,
        2,
        1,
        true,
        "roof-point",
        Interior::Warehouse,
    );
    plant_model(
        commands,
        "signpost",
        shore + Vec2::new(4.0, 1.0),
        FRAC_PI_2,
        3.0,
    );
    for offset in [Vec2::new(-5.0, -3.0), Vec2::new(-5.0, 7.0)] {
        plant_model(commands, "lightpost-single", shore + offset, 0.0, 2.2);
        let glow = prop(
            commands,
            "Sphere",
            ground(shore + offset) + Vec3::new(0.0, 2.6, 0.0),
            Vec3::new(0.22, 0.22, 0.22),
            palette.lantern,
        );
        commands.add_tag_type::<Lamp>(glow);
    }
    plant_model(
        commands,
        "pirate-tool-paddle",
        shore + Vec2::new(-6.0, -4.5),
        0.4,
        1.6,
    );
    plant_model(
        commands,
        "campfire-fishing-stand",
        shore + Vec2::new(1.0, -5.5),
        0.0,
        3.0,
    );
    brazier(commands, palette, shore + Vec2::new(1.0, -5.5), 0.1);
}
