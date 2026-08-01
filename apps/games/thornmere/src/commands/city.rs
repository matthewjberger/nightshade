use crate::commands::block;
use crate::commands::effects::flame;
use crate::commands::models::{place_model, plant_model};
use crate::commands::scenery::{house, lantern, prop, tower, turned};
use crate::data::*;
use crate::queries::{ground, moat_bridge_span, terrain_height};
use crate::resources::Palette;
use crate::tags::Lamp;
use nightshade::prelude::*;
use nightshade_random::prelude::*;

const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
const PI: f32 = std::f32::consts::PI;
const TAU: f32 = std::f32::consts::TAU;
const BRIDGE_PIECE: f32 = 0.93;
const BRIDGE_DECK: f32 = 1.18;
const HEX_BASE: f32 = 1.31;
const HEX_MID: f32 = 0.46;

pub fn spawn_city(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    walls(commands);
    gate(commands);
    moat_bridge(commands);
    streets(commands);
    plaza(commands, palette, random);
    for block in &CITY_BLOCKS {
        house(
            commands,
            palette,
            random,
            block.spot,
            block.yaw,
            block.cells,
            block.floors,
            block.wood,
            block.roof,
            block.interior,
        );
    }
    trade_district(commands, palette, random);
    canal(commands);
    cathedral_square(commands, palette, random);
    keep(commands, palette);
    old_town(commands, palette, random);
    park(commands, random);
    mage_quarter(commands, palette);
    street_lamps(commands, palette);
}

fn wall_run(commands: &mut Commands, from: Vec2, to: Vec2, gap: Option<(f32, f32)>) {
    let span = to - from;
    let length = span.magnitude();
    let count = (length / WALL_SCALE).round().max(1.0);
    let step = length / count;
    let direction = span / length;
    let yaw = if direction.x.abs() > direction.y.abs() {
        0.0
    } else {
        FRAC_PI_2
    };
    for index in 0..count as usize {
        let along = step * (index as f32 + 0.5);
        if let Some((low, high)) = gap
            && along > low
            && along < high
        {
            continue;
        }
        let spot = from + direction * along;
        plant_model(commands, "castle-wall", spot, yaw, WALL_SCALE);
    }
}

fn walls(commands: &mut Commands) {
    let north_west = Vec2::new(CITY_MIN.x, CITY_MAX.y);
    let north_east = CITY_MAX;
    let south_west = CITY_MIN;
    let south_east = Vec2::new(CITY_MAX.x, CITY_MIN.y);
    let half = (CITY_MAX.x - CITY_MIN.x) / 2.0;
    wall_run(
        commands,
        north_west,
        north_east,
        Some((half - GATE_HALF, half + GATE_HALF)),
    );
    wall_run(commands, south_west, south_east, None);
    wall_run(commands, south_west, north_west, None);
    wall_run(commands, south_east, north_east, None);
    let thick = WALL_SCALE / 2.0;
    for side in [-thick, thick] {
        block(
            commands,
            north_west + Vec2::new(0.0, side),
            Vec2::new(-GATE_HALF, CITY_MAX.y + side),
        );
        block(
            commands,
            Vec2::new(GATE_HALF, CITY_MAX.y + side),
            north_east + Vec2::new(0.0, side),
        );
        block(
            commands,
            south_west + Vec2::new(0.0, side),
            south_east + Vec2::new(0.0, side),
        );
        block(
            commands,
            south_west + Vec2::new(side, 0.0),
            north_west + Vec2::new(side, 0.0),
        );
        block(
            commands,
            south_east + Vec2::new(side, 0.0),
            north_east + Vec2::new(side, 0.0),
        );
    }
    block(
        commands,
        Vec2::new(-GATE_HALF, CITY_MAX.y - thick),
        Vec2::new(-GATE_HALF, CITY_MAX.y + thick),
    );
    block(
        commands,
        Vec2::new(GATE_HALF, CITY_MAX.y - thick),
        Vec2::new(GATE_HALF, CITY_MAX.y + thick),
    );
    for corner in [north_west, north_east, south_west, south_east] {
        tower(commands, corner, 0.0, 3, 4.6);
        place_model(
            commands,
            "flag-wide",
            ground(corner) + Vec3::new(0.0, 4.0 * 4.6, 0.0),
            0.0,
            2.4,
        );
    }
    for x in [-30.0, 30.0] {
        tower(commands, Vec2::new(x, CITY_MAX.y), 0.0, 2, 4.4);
        tower(commands, Vec2::new(x, CITY_MIN.y), 0.0, 2, 4.4);
    }
    for z in [-88.0, -112.0] {
        tower(commands, Vec2::new(CITY_MIN.x, z), 0.0, 2, 4.4);
        tower(commands, Vec2::new(CITY_MAX.x, z), 0.0, 2, 4.4);
    }
}

fn gate(commands: &mut Commands) {
    for side in [-1.0, 1.0] {
        let foot = Vec2::new(side * (GATE_HALF + WALL_SCALE), CITY_MAX.y);
        tower(commands, foot, 0.0, 4, 4.5);
        place_model(
            commands,
            "flag-banner-long",
            ground(foot) + Vec3::new(0.0, 5.0 * 4.5, 0.0),
            FRAC_PI_2,
            2.6,
        );
        let leaf = Vec2::new(side * 3.6, CITY_MAX.y + 0.5);
        plant_model(commands, "castle-gate", leaf, side * 1.15, 6.6);
        plant_model(
            commands,
            "castle-flag-pennant",
            Vec2::new(side * (GATE_HALF + 0.8), CITY_MAX.y - 2.4),
            0.0,
            3.0,
        );
    }
}

fn moat_bridge(commands: &mut Commands) {
    let scale = 4.4;
    let (start, end) = moat_bridge_span();
    let length = end - start;
    let count = (length / (BRIDGE_PIECE * scale)).ceil() as usize;
    let step = length / count as f32;
    let deck = terrain_height(0.0, CITY_MAX.y - 2.0) - BRIDGE_DECK * scale;
    for index in 0..count {
        let z = start + step * (index as f32 + 0.5);
        let piece = if index == 0 || index + 1 == count {
            "castle-bridge-straight-pillar"
        } else {
            "castle-bridge-straight"
        };
        place_model(commands, piece, Vec3::new(0.0, deck, z), FRAC_PI_2, scale);
    }
    for side in [-1.0, 1.0] {
        plant_model(
            commands,
            "lightpost-double",
            Vec2::new(side * 4.4, end + 1.6),
            0.0,
            3.2,
        );
    }
}

fn streets(commands: &mut Commands) {
    for street in CITY_STREETS {
        for pair in street.windows(2) {
            let span = pair[1] - pair[0];
            let length = span.magnitude();
            let direction = span / length;
            let yaw = direction.x.atan2(direction.y);
            let mut along = 0.0;
            while along <= length {
                let spot = pair[0] + direction * along;
                let position = Vec3::new(spot.x, terrain_height(spot.x, spot.y) + 0.06, spot.y);
                place_model(commands, "road", position, yaw, ROAD_WIDTH);
                along += ROAD_TILE_STEP;
            }
        }
    }
    let mut x = PLAZA_MIN.x + 1.5;
    while x < PLAZA_MAX.x {
        let mut z = PLAZA_MIN.y + 1.5;
        while z < PLAZA_MAX.y {
            let position = Vec3::new(x, terrain_height(x, z) + 0.05, z);
            place_model(commands, "road", position, 0.0, 3.0);
            z += 3.0;
        }
        x += 3.0;
    }
}

fn plaza(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let center = (PLAZA_MIN + PLAZA_MAX) / 2.0;
    plant_model(commands, "fountain-square-detail", center, 0.0, 4.5);
    let pedestal = Vec2::new(0.0, PLAZA_MIN.y + 2.4);
    plant_model(commands, "stairs-stone-round", pedestal, PI, 3.2);
    place_model(
        commands,
        "arena-statue",
        ground(pedestal) + Vec3::new(0.0, 1.2, 0.0),
        0.0,
        3.4,
    );
    let stalls = [
        (Vec2::new(-9.0, -77.0), "stall-green"),
        (Vec2::new(9.0, -77.0), "stall-red"),
        (Vec2::new(-9.0, -89.0), "stall"),
        (Vec2::new(9.0, -89.0), "stall-green"),
    ];
    let goods = [
        "market-display-bread",
        "market-display-fruit",
        "food-cheese",
        "food-fish",
        "food-pumpkin",
        "food-loaf-round",
        "food-pie",
        "food-loaf",
    ];
    for (index, (spot, stall)) in stalls.iter().enumerate() {
        let to = center - spot;
        let yaw = to.x.atan2(to.y) + PI;
        plant_model(commands, stall, *spot, yaw, 3.4);
        let counter = ground(*spot) + Vec3::new(0.0, 1.75, 0.0);
        for slot in 0..2 {
            let item = goods[(index * 2 + slot) % goods.len()];
            let offset = turned(Vec2::new(slot as f32 * 1.0 - 0.5, 0.2), yaw);
            place_model(
                commands,
                item,
                counter + Vec3::new(offset.x, 0.0, offset.y),
                yaw + between(random, -0.4, 0.4),
                2.2,
            );
        }
        plant_model(
            commands,
            "stall-stool",
            spot + turned(Vec2::new(1.8, -1.2), yaw),
            yaw,
            2.6,
        );
        plant_model(
            commands,
            "market-shopping-basket",
            spot + turned(Vec2::new(-1.6, 1.4), yaw),
            yaw,
            2.2,
        );
    }
    plant_model(commands, "cart", Vec2::new(4.0, -91.0), 0.8, 2.4);
    plant_model(commands, "cart-high", Vec2::new(-5.0, -91.5), -0.6, 2.4);
    plant_model(commands, "food-barrel", Vec2::new(-10.5, -83.0), 0.3, 2.4);
    plant_model(commands, "food-barrel", Vec2::new(-10.0, -84.6), 1.1, 2.4);
    plant_model(commands, "planks", Vec2::new(10.5, -83.5), 0.3, 2.4);
    for index in 0..8 {
        let angle = index as f32 / 8.0 * TAU + 0.2;
        let spot = center + Vec2::new(angle.cos() * 10.5, angle.sin() * 8.0);
        lantern(commands, palette, spot, angle);
    }
}

fn banners(commands: &mut Commands, block: Vec2, yaw: f32, cells: usize, colors: [&str; 2]) {
    let front = HOUSE_CELL * cells as f32 / 2.0;
    for (index, color) in colors.iter().enumerate() {
        let side = index as f32 * 2.0 - 1.0;
        let spot = block + turned(Vec2::new(side * HOUSE_CELL * 0.9, front + 0.15), yaw);
        place_model(
            commands,
            color,
            ground(spot) + Vec3::new(0.0, HOUSE_CELL * 1.2, 0.0),
            yaw - FRAC_PI_2,
            HOUSE_CELL,
        );
    }
}

fn pillars(commands: &mut Commands, block: Vec2, yaw: f32, cells: usize) {
    let front = HOUSE_CELL * cells as f32 / 2.0 + 0.9;
    for side in [-1.0, 1.0] {
        let spot = block + turned(Vec2::new(side * HOUSE_CELL * 1.1, front), yaw);
        plant_model(commands, "pillar-stone", spot, yaw, 2.4);
    }
}

fn trade_district(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    banners(
        commands,
        AUCTION_HOUSE,
        -FRAC_PI_2,
        3,
        ["banner-green", "banner-green"],
    );
    pillars(commands, AUCTION_HOUSE, -FRAC_PI_2, 3);
    plant_model(
        commands,
        "overhang",
        AUCTION_HOUSE + turned(Vec2::new(0.0, HOUSE_CELL * 1.5 + 0.3), -FRAC_PI_2),
        -FRAC_PI_2,
        HOUSE_CELL,
    );
    for offset in [-2.0, 2.0] {
        let spot = AUCTION_HOUSE + turned(Vec2::new(offset, HOUSE_CELL * 1.5 + 2.6), -FRAC_PI_2);
        plant_model(commands, "chest", spot, -FRAC_PI_2, 2.6);
    }
    banners(commands, BANK, FRAC_PI_2, 3, ["banner-red", "banner-red"]);
    pillars(commands, BANK, FRAC_PI_2, 3);
    plant_model(
        commands,
        "arena-block",
        BANK + turned(Vec2::new(-3.2, HOUSE_CELL * 1.5 + 2.4), FRAC_PI_2),
        0.3,
        2.0,
    );
    let front = PI;
    for side in [-1.0, 1.0] {
        let spot = INN + turned(Vec2::new(side * 2.6, HOUSE_CELL * 1.9), front);
        plant_model(commands, "barrel", spot, front, 3.0);
    }
    let bench = INN + turned(Vec2::new(0.0, HOUSE_CELL * 2.3), front);
    plant_model(commands, "stall-bench", bench, front + FRAC_PI_2, 2.4);
    banners(commands, INN, front, 3, ["banner-red", "banner-green"]);
    plant_model(commands, "food-mug", bench + Vec2::new(0.6, 0.0), 0.3, 2.4);
    let herbs = APOTHECARY + turned(Vec2::new(1.4, HOUSE_CELL * 1.5 + 1.2), PI);
    plant_model(commands, "pot_large", herbs, 0.0, 2.4);
    plant_model(commands, "pot_small", herbs + Vec2::new(1.1, 0.4), 0.0, 2.4);
    plant_model(
        commands,
        "plant_bushDetailed",
        APOTHECARY + turned(Vec2::new(-1.6, HOUSE_CELL * 1.5 + 1.0), PI),
        0.0,
        2.0,
    );
    for _ in 0..6 {
        let spot = Vec2::new(
            between(random, -11.0, 11.0),
            between(random, PLAZA_MIN.y + 1.0, PLAZA_MAX.y - 1.0),
        );
        if (spot - (PLAZA_MIN + PLAZA_MAX) / 2.0).magnitude() < 5.5 {
            continue;
        }
        let piece = [
            "box",
            "box-open",
            "food-bag",
            "pirate-crate",
            "retro-detail-crate-ropes",
        ][random.usize(0..5)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.4);
    }
    campfire_light(commands, palette, Vec2::new(0.0, -80.0));
}

fn campfire_light(commands: &mut Commands, palette: &Palette, spot: Vec2) {
    plant_model(commands, "fire-basket", spot, 0.0, 2.6);
    let glow = prop(
        commands,
        "Sphere",
        ground(spot) + Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.26, 0.18, 0.26),
        palette.fire,
    );
    commands.add_tag_type::<Lamp>(glow);
    flame(commands, ground(spot) + Vec3::new(0.0, 0.5, 0.0), 0.7);
}

fn canal(commands: &mut Commands) {
    let scale = 3.0;
    let deck = terrain_height(15.0, CANAL_Z - 8.0) - BRIDGE_DECK * scale;
    for x in CANAL_BRIDGES {
        for side in [-1.0, 1.0] {
            place_model(
                commands,
                "castle-bridge-straight",
                Vec3::new(x, deck, CANAL_Z + side * BRIDGE_PIECE * scale * 0.5),
                FRAC_PI_2,
                scale,
            );
        }
    }
    let bank = CANAL_HALF + 1.5;
    let mut x = -52.0;
    while x <= 52.0 {
        let near_bridge = CANAL_BRIDGES.iter().any(|bridge| (x - bridge).abs() < 2.6);
        if !near_bridge {
            for side in [-1.0, 1.0] {
                plant_model(
                    commands,
                    "fence",
                    Vec2::new(x, CANAL_Z + side * bank),
                    0.0,
                    2.4,
                );
            }
        }
        x += 2.4;
    }
    for x in [-44.0, -16.0, 16.0, 44.0] {
        plant_model(
            commands,
            "pirate-boat-row-small",
            Vec2::new(x, CANAL_Z),
            0.2,
            0.9,
        );
    }
}

fn cathedral_square(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let bell = Vec2::new(CATHEDRAL.x + 9.0, CATHEDRAL.y - 5.0);
    tower(commands, bell, 0.0, 5, 3.2);
    place_model(
        commands,
        "cross",
        ground(CATHEDRAL) + Vec3::new(0.0, 3.0 * HOUSE_CELL + 10.8, 0.0),
        0.0,
        4.0,
    );
    pillars(commands, CATHEDRAL, 0.0, 3);
    banners(commands, CATHEDRAL, 0.0, 3, ["banner-red", "banner-red"]);
    for side in [-1.0, 1.0] {
        let spot = CATHEDRAL + Vec2::new(side * 7.0, HOUSE_CELL * 1.5 + 1.4);
        plant_model(commands, "statue_column", spot, 0.0, 3.0);
    }
    plant_model(
        commands,
        "stairs-wide-stone",
        CATHEDRAL + Vec2::new(0.0, HOUSE_CELL * 1.5 + 1.2),
        PI,
        3.2,
    );

    let stones = [
        "gravestone-round",
        "gravestone-cross",
        "gravestone-bevel",
        "gravestone-flat",
        "gravestone-wide",
        "gravestone-decorative",
        "gravestone-roof",
    ];
    for row in 0..2 {
        for column in 0..6 {
            let spot = GRAVEYARD + Vec2::new(column as f32 * 2.0 - 6.0, row as f32 * 2.6 - 1.4);
            plant_model(
                commands,
                stones[random.usize(0..stones.len())],
                spot,
                PI + between(random, -0.2, 0.2),
                1.9,
            );
        }
    }
    let fence_scale = 1.9;
    for index in 0..9 {
        let x = GRAVEYARD.x - 7.6 + index as f32 * fence_scale;
        plant_model(
            commands,
            "iron-fence",
            Vec2::new(x, GRAVEYARD.y + 3.0),
            0.0,
            fence_scale,
        );
        plant_model(
            commands,
            "iron-fence",
            Vec2::new(x, GRAVEYARD.y - 3.0),
            0.0,
            fence_scale,
        );
    }
    for index in 0..3 {
        let z = GRAVEYARD.y - 2.0 + index as f32 * fence_scale;
        plant_model(
            commands,
            "iron-fence",
            Vec2::new(GRAVEYARD.x - 8.2, z),
            FRAC_PI_2,
            fence_scale,
        );
    }
    plant_model(
        commands,
        "iron-fence-border-gate",
        Vec2::new(GRAVEYARD.x + 8.2, GRAVEYARD.y),
        FRAC_PI_2,
        fence_scale,
    );
    plant_model(
        commands,
        "crypt",
        GRAVEYARD + Vec2::new(-6.0, 0.0),
        FRAC_PI_2,
        2.4,
    );
    for spot in [
        GRAVEYARD + Vec2::new(-8.6, 3.4),
        GRAVEYARD + Vec2::new(8.6, -3.4),
    ] {
        plant_model(commands, "lightpost-single", spot, 0.0, 2.2);
        let glow = prop(
            commands,
            "Sphere",
            ground(spot) + Vec3::new(0.0, 2.6, 0.0),
            Vec3::new(0.22, 0.22, 0.22),
            palette.lantern,
        );
        commands.add_tag_type::<Lamp>(glow);
    }
    plant_model(
        commands,
        "pine",
        GRAVEYARD + Vec2::new(-9.5, -1.0),
        0.0,
        1.7,
    );
    plant_model(commands, "bench", GRAVEYARD + Vec2::new(4.0, 4.6), PI, 2.2);
    plant_model(commands, "urn", GRAVEYARD + Vec2::new(-3.0, 4.4), 0.0, 2.2);
}

fn keep(commands: &mut Commands, palette: &Palette) {
    for (x, z) in [
        (-11.5, -133.0),
        (11.5, -133.0),
        (-11.5, -115.0),
        (11.5, -115.0),
    ] {
        let foot = Vec2::new(x, z);
        tower(commands, foot, 0.0, 5, 4.0);
        place_model(
            commands,
            "flag-banner-long",
            ground(foot) + Vec3::new(0.0, 6.0 * 4.0, 0.0),
            FRAC_PI_2,
            2.4,
        );
    }
    banners(commands, KEEP, 0.0, 4, ["banner-red", "banner-red"]);
    pillars(commands, KEEP, 0.0, 4);
    let wall_z = KEEP_MAX.y - 0.6;
    block(
        commands,
        Vec2::new(KEEP_MIN.x, wall_z),
        Vec2::new(-GATE_HALF - 0.6, wall_z),
    );
    block(
        commands,
        Vec2::new(GATE_HALF + 0.6, wall_z),
        Vec2::new(KEEP_MAX.x, wall_z),
    );
    let mut x = KEEP_MIN.x + 1.0;
    while x <= KEEP_MAX.x - 1.0 {
        if x.abs() > GATE_HALF + 0.6 {
            plant_model(
                commands,
                "castle-wall-narrow",
                Vec2::new(x, wall_z),
                FRAC_PI_2,
                4.0,
            );
        }
        x += 4.0;
    }
    for side in [-1.0, 1.0] {
        plant_model(
            commands,
            "castle-gate",
            Vec2::new(side * 3.0, wall_z + 0.4),
            side * 1.1,
            5.6,
        );
        plant_model(
            commands,
            "tower-square-base",
            Vec2::new(side * (GATE_HALF + 2.4), wall_z),
            0.0,
            3.4,
        );
        place_model(
            commands,
            "castle-tower-square-top-roof-high",
            ground(Vec2::new(side * (GATE_HALF + 2.4), wall_z)) + Vec3::new(0.0, 3.4, 0.0),
            0.0,
            3.4,
        );
    }
    for x in [-1.7, 0.0, 1.7] {
        plant_model(
            commands,
            "stairs-wide-stone",
            Vec2::new(x, KEEP_MAX.y + 1.2),
            PI,
            1.7,
        );
    }
    for spot in [
        Vec2::new(-6.0, -113.0),
        Vec2::new(6.0, -113.0),
        Vec2::new(-14.0, -113.0),
        Vec2::new(14.0, -113.0),
        Vec2::new(-14.0, -127.0),
        Vec2::new(14.0, -127.0),
    ] {
        lantern(commands, palette, spot, 0.0);
    }
    plant_model(
        commands,
        "castle-siege-trebuchet",
        Vec2::new(15.0, -124.0),
        -FRAC_PI_2,
        1.8,
    );
    plant_model(
        commands,
        "castle-siege-ballista",
        Vec2::new(-15.0, -124.0),
        FRAC_PI_2,
        2.0,
    );
    plant_model(commands, "weapon-rack", Vec2::new(-4.0, -111.5), PI, 2.4);
    plant_model(commands, "weapon-rack", Vec2::new(4.0, -111.5), PI, 2.4);
    for side in [-1.0, 1.0] {
        plant_model(
            commands,
            "castle-flag-pennant",
            Vec2::new(side * 12.0, -110.0),
            0.0,
            3.0,
        );
    }
}

fn old_town(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    plant_model(
        commands,
        "workbench-anvil",
        Vec2::new(20.5, -115.0),
        0.4,
        2.6,
    );
    campfire_light(commands, palette, Vec2::new(27.5, -115.0));
    plant_model(commands, "barrel", Vec2::new(29.5, -115.0), 0.2, 2.8);
    plant_model(commands, "tool-pickaxe", Vec2::new(21.8, -114.6), 1.2, 2.6);
    plant_model(commands, "weapon-rack", Vec2::new(-27.5, -115.0), PI, 2.4);
    plant_model(
        commands,
        "resource-planks",
        Vec2::new(-20.5, -115.2),
        0.5,
        2.6,
    );

    let low = STABLE + Vec2::new(-5.0, -4.0);
    let high = STABLE + Vec2::new(5.0, 4.2);
    let segment = 2.2;
    let mut x = low.x + segment / 2.0;
    while x < high.x {
        plant_model(commands, "fence_simple", Vec2::new(x, low.y), 0.0, segment);
        plant_model(commands, "fence_simple", Vec2::new(x, high.y), 0.0, segment);
        x += segment;
    }
    let mut z = low.y + segment / 2.0;
    while z < high.y {
        if z < high.y - 2.5 || z > high.y - 0.5 {
            plant_model(
                commands,
                "fence_simple",
                Vec2::new(low.x, z),
                FRAC_PI_2,
                segment,
            );
        }
        plant_model(
            commands,
            "fence_simple",
            Vec2::new(high.x, z),
            FRAC_PI_2,
            segment,
        );
        z += segment;
    }
    plant_model(
        commands,
        "fence_gate",
        Vec2::new(low.x, high.y - 1.5),
        FRAC_PI_2,
        segment,
    );
    for (spot, yaw) in HORSE_SPOTS {
        place_model(
            commands,
            HORSE_MODEL,
            ground(spot) + Vec3::new(0.0, HORSE_LIFT, 0.0),
            yaw,
            HORSE_SCALE,
        );
    }
    plant_model(commands, "hay-bale", Vec2::new(44.2, -102.0), 0.4, 2.6);
    plant_model(
        commands,
        "hay-bale-bundled",
        Vec2::new(52.0, -106.0),
        1.4,
        2.6,
    );
    plant_model(commands, "bucket", Vec2::new(45.0, -107.5), 0.0, 2.4);
    plant_model(commands, "planks", Vec2::new(33.0, -114.6), 0.9, 2.4);
    plant_model(commands, "poles", Vec2::new(50.0, -114.8), 0.2, 2.4);
    for _ in 0..5 {
        let spot = Vec2::new(between(random, 32.0, 54.0), between(random, -115.0, -114.0));
        let piece = ["barrel-open", "box-large", "retro-barrels", "log_stack"][random.usize(0..4)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.4);
    }
}

fn park(commands: &mut Commands, random: &mut Random) {
    plant_model(commands, "fountain-round", PARK, 0.0, 3.0);
    for (offset, kind) in [
        (Vec2::new(-3.6, -3.4), "tree-high"),
        (Vec2::new(3.8, -3.0), "tree"),
        (Vec2::new(-3.4, 3.6), "tree-high-round"),
        (Vec2::new(3.6, 3.4), "tree-high"),
    ] {
        plant_model(commands, kind, PARK + offset, random.f32() * TAU, 4.0);
    }
    for index in 0..10 {
        let angle = index as f32 / 10.0 * TAU;
        if (angle - PI * 1.5).abs() < 0.4 {
            continue;
        }
        let spot = PARK + Vec2::new(angle.cos() * 5.6, angle.sin() * 5.2);
        plant_model(commands, "hedge", spot, -angle + FRAC_PI_2, 2.2);
    }
    plant_model(
        commands,
        "bench",
        PARK + Vec2::new(-2.0, 0.0),
        FRAC_PI_2,
        2.2,
    );
    plant_model(
        commands,
        "bench",
        PARK + Vec2::new(2.0, 0.0),
        -FRAC_PI_2,
        2.2,
    );
    let flowers = [
        "flower_purpleA",
        "flower_redB",
        "flower_yellowC",
        "flower_purpleC",
        "flower_redA",
    ];
    for _ in 0..12 {
        let spot = PARK + Vec2::new(between(random, -4.6, 4.6), between(random, -4.4, 4.4));
        if (spot - PARK).magnitude() < 2.4 {
            continue;
        }
        plant_model(
            commands,
            flowers[random.usize(0..flowers.len())],
            spot,
            random.f32() * TAU,
            2.4,
        );
    }
}

fn mage_quarter(commands: &mut Commands, palette: &Palette) {
    let scale = 5.0;
    let foot = ground(MAGE_TOWER);
    plant_model(
        commands,
        "castle-tower-hexagon-base",
        MAGE_TOWER,
        0.0,
        scale,
    );
    let mut height = HEX_BASE * scale;
    for _ in 0..3 {
        place_model(
            commands,
            "castle-tower-hexagon-mid",
            foot + Vec3::new(0.0, height, 0.0),
            0.0,
            scale,
        );
        height += HEX_MID * scale;
    }
    place_model(
        commands,
        "castle-tower-hexagon-roof",
        foot + Vec3::new(0.0, height, 0.0),
        0.0,
        scale,
    );
    place_model(
        commands,
        "flag",
        foot + Vec3::new(0.0, height + 0.83 * scale, 0.0),
        0.0,
        2.0,
    );
    for index in 0..6 {
        let angle = index as f32 / 6.0 * TAU;
        if index == 1 {
            continue;
        }
        let spot = MAGE_TOWER + Vec2::new(angle.cos() * 6.2, angle.sin() * 6.2);
        plant_model(commands, "hedge-large", spot, -angle + FRAC_PI_2, 2.4);
    }
    for (offset, piece) in [
        (Vec2::new(-4.0, 2.4), "statue_obelisk"),
        (Vec2::new(4.2, -2.2), "statue_ring"),
    ] {
        plant_model(commands, piece, MAGE_TOWER + offset, 0.0, 2.2);
    }
    for offset in [
        Vec2::new(3.2, 3.2),
        Vec2::new(-3.4, -3.0),
        Vec2::new(0.0, -4.4),
    ] {
        let spot = MAGE_TOWER + offset;
        plant_model(commands, "detail-crystal", spot, 0.0, 2.6);
        let glow = prop(
            commands,
            "Sphere",
            ground(spot) + Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.28, 0.28, 0.28),
            palette.violet,
        );
        commands.add_tag_type::<Lamp>(glow);
    }
    lantern(commands, palette, MAGE_TOWER + Vec2::new(5.2, 4.6), 0.0);
}

fn street_lamps(commands: &mut Commands, palette: &Palette) {
    let mut z = -68.0;
    while z > -106.0 {
        if (z - CANAL_Z).abs() > 4.5 && !(PLAZA_MIN.y..PLAZA_MAX.y).contains(&z) {
            lantern(commands, palette, Vec2::new(-2.8, z), FRAC_PI_2);
            lantern(commands, palette, Vec2::new(2.8, z), -FRAC_PI_2);
        }
        z -= 9.0;
    }
    for x in [-46.0, -28.5, 28.5, 46.0] {
        lantern(commands, palette, Vec2::new(x, -72.6), 0.0);
    }
    for x in [-34.5, -12.6, 12.6, 34.5] {
        lantern(commands, palette, Vec2::new(x, -125.4), PI);
    }
    for z in [-80.0, -100.0, -118.0] {
        lantern(commands, palette, Vec2::new(-32.6, z), FRAC_PI_2);
        lantern(commands, palette, Vec2::new(32.6, z), -FRAC_PI_2);
    }
}
