use crate::data::world::biomes::Biome;
use crate::resources::Bench;
use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::items::{Item, ItemStack};
use blocker_blocks::queries::{block_at, ground_height};
use blocker_blocks::resources::VoxelWorld;
use nalgebra_glm::IVec3;
use noise::NoiseFn;

pub fn hash(seed: u32, x: i32, z: i32) -> u32 {
    let mut value =
        seed ^ (x as u32).wrapping_mul(0x9E37_79B1) ^ (z as u32).wrapping_mul(0x85EB_CA77);
    value ^= value >> 15;
    value = value.wrapping_mul(0x2C1B_3C6D);
    value ^= value >> 12;
    value = value.wrapping_mul(0x297A_2D39);
    value ^= value >> 15;
    value
}

pub fn cell_anchor(seed: u32, salt: u32, cell: i32, gx: i32, gz: i32, margin: i32) -> (i32, i32) {
    let room = (cell - 2 * margin).max(1) as u32;
    let value = hash(seed ^ salt, gx, gz);
    let offset_x = margin + (value % room) as i32;
    let offset_z = margin + ((value / room.max(1)) % room) as i32;
    (gx * cell + offset_x, gz * cell + offset_z)
}

pub fn surface_height(bench: &Bench, x: i32, z: i32) -> i32 {
    let continent = bench.elevation.get([x as f64 / 900.0, z as f64 / 900.0]);
    let hills = bench.hills.get([x as f64 / 150.0, z as f64 / 150.0]);
    let mountain = (continent - 0.32).max(0.0);
    let height = crate::data::world::SEA_LEVEL as f64
        + continent * 46.0
        + hills * 9.0
        + mountain * mountain * 300.0;
    (height.round() as i32).clamp(
        nightshade_voxels::data::FLOOR_Y + 6,
        nightshade_voxels::data::WORLD_Y - 30,
    )
}

pub fn biome_for(bench: &Bench, x: i32, z: i32, height: i32) -> Biome {
    if height <= crate::data::world::SEA_LEVEL - 2 {
        return Biome::Ocean;
    }
    if height <= crate::data::world::SEA_LEVEL + 1 {
        return Biome::Beach;
    }
    if height >= 150 {
        return Biome::SnowyPeaks;
    }
    if height >= 124 {
        return Biome::Mountain;
    }
    let temperature = bench.temperature.get([x as f64 / 340.0, z as f64 / 340.0]);
    let moisture = bench.moisture.get([x as f64 / 250.0, z as f64 / 250.0]);
    if temperature < -0.35 {
        if moisture < 0.0 {
            Biome::Tundra
        } else {
            Biome::SnowyPlains
        }
    } else if temperature < -0.12 {
        if moisture > 0.12 {
            Biome::Taiga
        } else {
            Biome::SnowyPlains
        }
    } else if temperature < 0.12 {
        if moisture > 0.28 {
            Biome::Swamp
        } else if moisture > 0.08 {
            Biome::Forest
        } else if moisture > -0.12 {
            Biome::Meadow
        } else {
            Biome::Plains
        }
    } else if temperature < 0.34 {
        if moisture > 0.24 {
            Biome::Jungle
        } else if moisture > 0.02 {
            Biome::BirchForest
        } else if moisture > -0.2 {
            Biome::Savanna
        } else {
            Biome::Plains
        }
    } else if moisture < -0.08 {
        Biome::Desert
    } else if moisture < 0.18 {
        Biome::Badlands
    } else {
        Biome::Jungle
    }
}

pub fn category_on(mask: u32, index: usize) -> bool {
    mask & (1u32 << index) != 0
}

pub fn rotate_local(lx: i32, lz: i32, span: i32, rot: u8) -> (i32, i32) {
    match rot % 4 {
        1 => (span - 1 - lz, lx),
        2 => (span - 1 - lx, span - 1 - lz),
        3 => (lz, span - 1 - lx),
        _ => (lx, lz),
    }
}

pub fn rotate_block(kind: BlockKind, rot: u8) -> BlockKind {
    let step = (rot % 4) as usize;
    if step == 0 {
        return kind;
    }
    let doors = [
        BlockKind::DoorNorth,
        BlockKind::DoorEast,
        BlockKind::DoorSouth,
        BlockKind::DoorWest,
    ];
    let torches = [
        BlockKind::TorchNorth,
        BlockKind::TorchEast,
        BlockKind::TorchSouth,
        BlockKind::TorchWest,
    ];
    if let Some(index) = doors.iter().position(|door| *door == kind) {
        return doors[(index + step) % 4];
    }
    if let Some(index) = torches.iter().position(|torch| *torch == kind) {
        return torches[(index + step) % 4];
    }
    kind
}

pub fn village_name(seed: u32, x: i32, z: i32) -> String {
    let value = hash(seed ^ 0x5713_2A9F, x, z);
    let heads = crate::data::names::NAME_HEADS;
    let tails = crate::data::names::NAME_TAILS;
    let head = heads[(value % heads.len() as u32) as usize];
    let tail = tails[((value / 17) % tails.len() as u32) as usize];
    format!("{head}{tail}")
}

pub fn ground_is_buildable(bench: &Bench, x: i32, z: i32, reach: i32) -> bool {
    let mut lowest = surface_height(bench, x, z);
    let mut highest = lowest;
    if lowest < crate::data::world::SEA_LEVEL + theme::SETTLEMENT_LIFT {
        return false;
    }
    for ring in [reach / 2, reach] {
        for probe in 0..theme::SEAT_PROBES {
            let turn = probe as f32 / theme::SEAT_PROBES as f32 * std::f32::consts::TAU;
            let sample_x = x + (turn.cos() * ring as f32).round() as i32;
            let sample_z = z + (turn.sin() * ring as f32).round() as i32;
            let height = surface_height(bench, sample_x, sample_z);
            if height < crate::data::world::SEA_LEVEL + theme::SETTLEMENT_LIFT {
                return false;
            }
            lowest = lowest.min(height);
            highest = highest.max(height);
        }
    }
    highest - lowest <= theme::SETTLEMENT_RELIEF
}

fn seat_is_open(bench: &Bench, x: i32, z: i32, reach: i32, avoid: &[([i32; 2], i32)]) -> bool {
    ground_is_buildable(bench, x, z, reach)
        && !fixed_ground_is_claimed(bench, bench.seed, x, z, reach)
        && avoid
            .iter()
            .all(|(center, clear)| (center[0] - x).abs().max((center[1] - z).abs()) >= *clear)
}

pub fn dry_seat(bench: &Bench, at: [i32; 2], reach: i32, avoid: &[([i32; 2], i32)]) -> [i32; 2] {
    if seat_is_open(bench, at[0], at[1], reach, avoid) {
        return at;
    }
    for ring in 1..=theme::SEAT_RINGS {
        let span = ring * theme::SEAT_STEP;
        for probe in 0..(theme::SEAT_PROBES * ring.min(3)) {
            let turn =
                probe as f32 / (theme::SEAT_PROBES * ring.min(3)) as f32 * std::f32::consts::TAU;
            let x = at[0] + (turn.cos() * span as f32).round() as i32;
            let z = at[1] + (turn.sin() * span as f32).round() as i32;
            if seat_is_open(bench, x, z, reach, avoid) {
                return [x, z];
            }
        }
    }
    at
}

pub fn dry_spot(world: &VoxelWorld, start_x: i32, start_z: i32) -> (i32, i32) {
    for radius in 0i32..theme::LANDING_SEARCH {
        for offset_z in -radius..=radius {
            for offset_x in -radius..=radius {
                if offset_x.abs() != radius && offset_z.abs() != radius {
                    continue;
                }
                let x = start_x + offset_x;
                let z = start_z + offset_z;
                if let Some(top) = ground_height(world, x, z)
                    && top > crate::data::world::SEA_LEVEL + 1
                    && block_at(world, IVec3::new(x, top + 1, z)) == BlockKind::Air
                {
                    return (x, z);
                }
            }
        }
    }
    (start_x, start_z)
}

pub fn barrow_floor(bench: &Bench, at: [i32; 2]) -> i32 {
    surface_height(bench, at[0], at[1]).max(crate::data::world::SEA_LEVEL + 2)
}

pub fn hash3(seed: u32, x: i32, y: i32, z: i32) -> u32 {
    let mut value = seed
        ^ (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0xC2B2_AE3D)
        ^ (z as u32).wrapping_mul(0x85EB_CA77);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^= value >> 16;
    value
}

pub fn vein_at(seed: u32, x: i32, y: i32, z: i32) -> Option<&'static crate::data::ores::Vein> {
    let roll = hash3(seed, x, y, z) % crate::data::ores::VEIN_ROLL;
    let mut floor = 0;
    for vein in crate::data::ores::VEINS.iter() {
        if y < vein.low || y > vein.high {
            continue;
        }
        if roll >= floor && roll < floor + vein.chance {
            return Some(vein);
        }
        floor += vein.chance;
    }
    None
}

pub fn settle_the_seats(
    bench: &Bench,
) -> [crate::resources::Seat; crate::data::world::places::CAPITALS.len()] {
    let mut seats = [crate::resources::Seat::default(); crate::data::world::places::CAPITALS.len()];
    for (index, capital) in crate::data::world::places::CAPITALS.iter().enumerate() {
        let city = dry_seat(bench, capital.at, theme::CITY_SEAT_REACH, &[]);
        let barrow = dry_seat(
            bench,
            [
                city[0] + crate::data::world::places::BARROW_OFFSET[0],
                city[1] + crate::data::world::places::BARROW_OFFSET[1],
            ],
            theme::DEN_SEAT_REACH,
            &[(
                city,
                crate::data::world::places::den_keeps_clear(theme::DEN_CLAIM),
            )],
        );
        let camp = dry_seat(
            bench,
            [
                city[0] + crate::data::world::places::CAMP_OFFSET[0],
                city[1] + crate::data::world::places::CAMP_OFFSET[1],
            ],
            theme::DEN_SEAT_REACH,
            &[
                (
                    city,
                    crate::data::world::places::den_keeps_clear(theme::CAMP_REACH),
                ),
                (barrow, theme::DEN_CLAIM + theme::CAMP_REACH + 1),
            ],
        );
        seats[index] = crate::resources::Seat { city, barrow, camp };
    }
    seats
}

pub fn seat_at(
    bench: &Bench,
    x: i32,
    z: i32,
) -> Option<&'static crate::data::world::places::Capital> {
    bench
        .seats
        .iter()
        .position(|seat| seat.city[0] == x && seat.city[1] == z)
        .and_then(|index| crate::data::world::places::CAPITALS.get(index))
}

fn town_at(
    bench: &Bench,
    seed: u32,
    cell: i32,
    child: usize,
    margin: i32,
    grid_x: i32,
    grid_z: i32,
) -> Option<(i32, i32)> {
    let (x, z) = cell_anchor(seed, child as u32, cell, grid_x, grid_z, margin);
    if !ground_is_buildable(bench, x, z, theme::SETTLEMENT_FOOTPRINT) {
        return None;
    }
    let height = surface_height(bench, x, z);
    let weight = biome_for(bench, x, z, height).info().village_weight;
    let roll = hash(seed ^ (child as u32).wrapping_add(0x51ED), grid_x, grid_z) % 100;
    (roll < weight).then_some((x, z))
}

fn placed_piece_claims(at: [i32; 2], schematic: usize, x: i32, z: i32, want: i32) -> bool {
    let claim =
        crate::data::world::schematics::SCHEMATICS[schematic].span / 2 + theme::BUILDING_APRON;
    (at[0] - x).abs() <= claim + want && (at[1] - z).abs() <= claim + want
}

pub fn ground_is_claimed(bench: &Bench, seed: u32, x: i32, z: i32, want: i32) -> bool {
    if fixed_ground_is_claimed(bench, seed, x, z, want) {
        return true;
    }
    for seat in bench.seats.iter() {
        for (at, claim) in [
            (seat.city, theme::CITY_SEAT_REACH),
            (seat.barrow, theme::DEN_CLAIM),
            (seat.camp, theme::DEN_CLAIM),
        ] {
            if claim > want
                && (at[0] - x).abs() <= claim + want
                && (at[1] - z).abs() <= claim + want
            {
                return true;
            }
        }
    }
    false
}

fn fixed_ground_is_claimed(bench: &Bench, seed: u32, x: i32, z: i32, want: i32) -> bool {
    for poi in crate::data::world::places::POIS.iter() {
        if placed_piece_claims(poi.at, poi.schematic, x, z, want) {
            return true;
        }
    }
    for landmark in crate::data::world::places::LANDMARKS.iter() {
        let crate::data::world::nodes::NodeKind::Stamp { schematic } =
            &crate::data::world::nodes::NODES[landmark.node].kind
        else {
            continue;
        };
        if placed_piece_claims(landmark.at, *schematic, x, z, want) {
            return true;
        }
    }
    for (cell, child, margin, claim) in [
        (
            theme::TOWN_CELL,
            crate::data::world::nodes::SETTLEMENT_PICK,
            theme::TOWN_REACH,
            theme::TOWN_REACH,
        ),
        (
            theme::HAMLET_CELL,
            crate::data::world::nodes::SMALL_PICK,
            6,
            theme::HAMLET_REACH,
        ),
    ] {
        if claim <= want {
            continue;
        }
        let grid_x = x.div_euclid(cell);
        let grid_z = z.div_euclid(cell);
        for offset_z in -1..=1 {
            for offset_x in -1..=1 {
                let Some((town_x, town_z)) = town_at(
                    bench,
                    seed,
                    cell,
                    child,
                    margin,
                    grid_x + offset_x,
                    grid_z + offset_z,
                ) else {
                    continue;
                };
                if (town_x - x).abs() <= claim + want && (town_z - z).abs() <= claim + want {
                    return true;
                }
            }
        }
    }
    false
}

pub fn gate_notice() -> String {
    let names: Vec<&str> = crate::data::world::places::LOST
        .iter()
        .map(|lost| lost.name)
        .collect();
    format!(
        "{} {}. {}",
        crate::data::world::places::GATE_NOTICE_HEAD,
        names.join(", "),
        crate::data::world::places::GATE_NOTICE_TAIL
    )
}

pub fn city_gate(bench: &Bench, index: usize) -> [i32; 2] {
    bench
        .seats
        .get(index)
        .map(|seat| [seat.city[0], seat.city[1] + theme::GATE_WALK])
        .unwrap_or([0, 0])
}

pub fn terrace_reach(layout: &crate::data::world::nodes::Layout) -> i32 {
    let mut reach = layout.wall;
    for slot in layout.slots {
        reach = reach.max(slot.dx.abs()).max(slot.dz.abs());
    }
    reach + crate::theme::CITY_TERRACE_MARGIN
}

pub fn ruin_loot(x: i32, z: i32) -> Vec<Option<ItemStack>> {
    let mut slots = vec![None; crate::data::world::CHEST_SLOTS];
    let ruin = crate::data::world::places::POIS.iter().any(|poi| {
        poi.schematic == crate::data::world::schematics::KEEP_SCHEMATIC && poi.at == [x, z]
    });
    if !ruin {
        return slots;
    }
    let loot = [
        (Item::Block(BlockKind::IronOre), 5),
        (Item::Block(BlockKind::GoldOre), 3),
        (Item::Block(BlockKind::DiamondOre), 1),
        (Item::Block(BlockKind::Glowstone), 2),
        (Item::Block(BlockKind::Obsidian), 2),
    ];
    for (index, (item, count)) in loot.iter().enumerate() {
        if let Some(slot) = slots.get_mut(index) {
            *slot = Some(ItemStack {
                item: *item,
                count: *count,
            });
        }
    }
    slots
}

pub fn fresh_seed() -> u32 {
    fastrand::u32(..)
}
