use crate::data::districts::{
    ALLEY_PROPS, CAFE_CHAIRS, CAFE_TABLES, CARS, COMMERCIAL_BUILDINGS, CURB_PROPS, District, FENCE,
    HOUSES, INDUSTRIAL_BUILDINGS, INDUSTRIAL_PROPS, LAMP_POST, PLANTER, ROAD_CROSSING,
    ROAD_CROSSROAD, ROAD_STRAIGHT, SCAFFOLDING, SIDEWALK_PROPS, SITE_PROPS, TOWERS, TREES, TRUCKS,
    YARD_PROPS,
};
use crate::data::look::{NEON_COLORS, NEON_DEPTH, NEON_HEIGHT};
use crate::data::motion::{DOOR_OUT, PROP_GAP, PROP_TRIES};
use crate::data::rooms::{HOME_ROOMS, Room, SHOP_ROOMS, WORKS_ROOMS};
use crate::data::scale::{
    CAR_SCALE, CELL, CELL_TILES, KIT_SCALE, PAVEMENT, ROAD_LIFT, TILE, TOWER_SCALE,
};
use crate::queries::collide::lots_overlap;
use crate::queries::footprints::{extent, kit_scale, lot_of, origin_for, post_lot};
use crate::queries::grid::{district_of, tile_center, yaw_of};
use crate::resources::{CellJob, CellLayout, Door, Lamp, Lot, Neon, Pavement, Placement, Plan};
use nightshade::prelude::*;
use nightshade_kit::prelude::model;
use nightshade_random::prelude::*;
use nightshade_worker::prelude::Job;

const EDGE_INSET: f32 = 12.0;
const LAMP_COLUMNS: [i32; 2] = [2, 5];
const SIDEWALK_LINE: f32 = 0.4;
const LAMP_ARM: f32 = 0.2;
const LAMP_HEAD: f32 = 0.65;
const PARKING_ROWS: [f32; 2] = [0.32, 0.68];
const PARKING_STEP: f32 = 3.4;
const YARD_GAP: [f32; 2] = [2.5, 6.0];
const SHOP_GAP: [f32; 2] = [0.0, 1.5];
const PROP_COUNT: [usize; 2] = [4, 9];
const SIDEWALK_GAP: [f32; 2] = [4.0, 12.0];
const FACE_GAP: f32 = 0.12;
const CAFE_CHANCE: f32 = 0.3;
const CAFE_ROOM: f32 = 0.15;
const CURB_LINE: f32 = 3.6;
const CURB_STEP: f32 = 2.6;

impl Job for CellJob {
    type Key = [i32; 2];
    type Done = CellLayout;
    type Tools = ();

    fn key(&self) -> Self::Key {
        self.key
    }

    fn run(self, _tools: &mut Self::Tools) -> Self::Done {
        lay_out(self.key, self.seed, self.plan)
    }
}

struct Sheet {
    key: [i32; 2],
    plan: Plan,
    district: District,
    placements: Vec<Placement>,
    lots: Vec<Lot>,
    props: Vec<Option<Lot>>,
    lamps: Vec<Lamp>,
    neons: Vec<Neon>,
    pavements: Vec<Pavement>,
    doors: Vec<Door>,
}

fn lay_out(key: [i32; 2], seed: u64, plan: Plan) -> CellLayout {
    let salt = (key[0] as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (key[1] as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let mut random = Random::seeded(seed ^ salt);
    let district = district_of(key);
    let mut sheet = Sheet {
        key,
        plan,
        district,
        placements: Vec::new(),
        lots: Vec::new(),
        props: Vec::new(),
        lamps: Vec::new(),
        neons: Vec::new(),
        pavements: Vec::new(),
        doors: Vec::new(),
    };
    pave(&mut sheet);
    light(&mut sheet, district);
    match district {
        District::Downtown => downtown(&mut sheet, &mut random),
        District::Commercial => midtown(&mut sheet, &mut random),
        District::Industrial => works(&mut sheet, &mut random),
        District::Suburban => rows(&mut sheet, &mut random),
    }
    sidewalks(&mut sheet, &mut random);
    if chance(&mut random, sheet.plan.site_chance) {
        site(&mut sheet, &mut random);
    }
    CellLayout {
        key,
        district,
        placements: sheet.placements,
        lots: sheet.lots,
        props: sheet.props,
        lamps: sheet.lamps,
        neons: sheet.neons,
        pavements: sheet.pavements,
        doors: sheet.doors,
    }
}

fn pave(sheet: &mut Sheet) {
    let lift = Vec3::new(0.0, ROAD_LIFT, 0.0);
    for column in 0..CELL_TILES {
        for row in 0..CELL_TILES {
            let (name, yaw) = if column == 0 && row == 0 {
                (ROAD_CROSSROAD, 0.0)
            } else if row == 0 {
                let crossing = column == 1 || column == CELL_TILES - 1;
                (
                    if crossing {
                        ROAD_CROSSING
                    } else {
                        ROAD_STRAIGHT
                    },
                    0.0,
                )
            } else if column == 0 {
                let crossing = row == 1 || row == CELL_TILES - 1;
                (
                    if crossing {
                        ROAD_CROSSING
                    } else {
                        ROAD_STRAIGHT
                    },
                    std::f32::consts::FRAC_PI_2,
                )
            } else {
                continue;
            };
            sheet.placements.push(Placement {
                model: model(name),
                position: tile_center(sheet.key, column, row) + lift,
                yaw,
                scale: KIT_SCALE,
                prop: None,
            });
        }
    }
    let base = Vec3::new(
        sheet.key[0] as f32 * CELL,
        ROAD_LIFT * 0.5,
        sheet.key[1] as f32 * CELL,
    );
    let thickness = -ROAD_LIFT;
    let long = CELL - TILE;
    let short = CELL - TILE - PAVEMENT * 2.0;
    let beside = Vec3::new(PAVEMENT, thickness, long);
    let across = Vec3::new(short, thickness, PAVEMENT);
    sheet.pavements.push(Pavement {
        position: base + Vec3::new(-PAVEMENT * 0.5, 0.0, TILE + long * 0.5),
        size: beside,
    });
    sheet.pavements.push(Pavement {
        position: base + Vec3::new(TILE + PAVEMENT * 0.5, 0.0, TILE + long * 0.5),
        size: beside,
    });
    sheet.pavements.push(Pavement {
        position: base + Vec3::new(TILE + PAVEMENT + short * 0.5, 0.0, -PAVEMENT * 0.5),
        size: across,
    });
    sheet.pavements.push(Pavement {
        position: base + Vec3::new(TILE + PAVEMENT + short * 0.5, 0.0, TILE + PAVEMENT * 0.5),
        size: across,
    });
}

fn light(sheet: &mut Sheet, district: District) {
    let warm = matches!(district, District::Industrial | District::Suburban);
    let across = SIDEWALK_LINE * TILE;
    for &column in &LAMP_COLUMNS {
        let tile = tile_center(sheet.key, column, 0);
        lamp(sheet, tile + Vec3::new(0.0, 0.0, across), 0.0, warm, true);
        lamp(
            sheet,
            tile + Vec3::new(0.0, 0.0, -across),
            std::f32::consts::PI,
            warm,
            false,
        );
        let tile = tile_center(sheet.key, 0, column);
        lamp(
            sheet,
            tile + Vec3::new(across, 0.0, 0.0),
            std::f32::consts::FRAC_PI_2,
            warm,
            true,
        );
        lamp(
            sheet,
            tile + Vec3::new(-across, 0.0, 0.0),
            -std::f32::consts::FRAC_PI_2,
            warm,
            false,
        );
    }
}

fn lamp(sheet: &mut Sheet, position: Vec3, yaw: f32, warm: bool, lit: bool) {
    let index = sheet.props.len();
    sheet.props.push(Some(post_lot(position)));
    sheet.placements.push(Placement {
        model: model(LAMP_POST),
        position,
        yaw,
        scale: KIT_SCALE,
        prop: Some(index),
    });
    if lit {
        let arm = nalgebra_glm::quat_rotate_vec3(
            &nalgebra_glm::quat_angle_axis(yaw, &Vec3::y()),
            &Vec3::new(0.0, 0.0, -1.0),
        );
        sheet.lamps.push(Lamp {
            position: position + arm * (LAMP_ARM * TILE) + Vec3::new(0.0, LAMP_HEAD * TILE, 0.0),
            warm,
            prop: index,
        });
    }
}

fn block_origin(key: [i32; 2]) -> Vec3 {
    Vec3::new(
        key[0] as f32 * CELL + TILE + PAVEMENT,
        0.0,
        key[1] as f32 * CELL + TILE + PAVEMENT,
    )
}

fn block_span() -> f32 {
    CELL - TILE - PAVEMENT * 2.0
}

fn put(sheet: &mut Sheet, name: &str, position: Vec3, yaw: f32, scale: f32, solid: bool) {
    let placement = Placement {
        model: model(name),
        position,
        yaw,
        scale,
        prop: None,
    };
    if solid {
        sheet.lots.push(lot_of(&placement, name));
    }
    sheet.placements.push(placement);
}

fn prop(sheet: &mut Sheet, name: &str, position: Vec3, yaw: f32) -> bool {
    let placement = Placement {
        model: model(name),
        position,
        yaw,
        scale: kit_scale(name),
        prop: Some(sheet.props.len()),
    };
    let lot = lot_of(&placement, name);
    if sheet
        .lots
        .iter()
        .any(|other| lots_overlap(&lot, other, 0.0))
        || sheet
            .props
            .iter()
            .flatten()
            .any(|other| lots_overlap(&lot, other, PROP_GAP))
    {
        return false;
    }
    sheet.props.push(Some(lot));
    sheet.placements.push(placement);
    true
}

fn prop_at(sheet: &mut Sheet, name: &str, center: Vec3, yaw: f32) -> bool {
    let position = origin_for(name, kit_scale(name), Vec2::new(center.x, center.z), yaw);
    prop(sheet, name, position, yaw)
}

fn neon_on(
    sheet: &mut Sheet,
    random: &mut Random,
    center: Vec3,
    facing: Vec3,
    width: f32,
    depth: f32,
    height: f32,
) {
    if !chance(random, sheet.plan.neon_chance) {
        return;
    }
    let color = index(random, NEON_COLORS.len());
    let lift = between(random, 0.55, 0.9) * height;
    let along = width * between(random, 0.4, 0.8);
    let size = if facing.z.abs() > 0.5 {
        Vec3::new(along, NEON_HEIGHT, NEON_DEPTH)
    } else {
        Vec3::new(NEON_DEPTH, NEON_HEIGHT, along)
    };
    sheet.neons.push(Neon {
        position: center + facing * (depth * 0.5 + NEON_DEPTH * 0.5) + Vec3::new(0.0, lift, 0.0),
        size,
        color,
    });
}

fn fill_edge(
    sheet: &mut Sheet,
    random: &mut Random,
    models: &[&str],
    scale: f32,
    start: Vec3,
    direction: Vec3,
    length: f32,
    facing: Vec3,
    gap: [f32; 2],
    neon: bool,
    rooms: &[Room],
) {
    let mut cursor = 0.0;
    loop {
        let Some(name) = pick(random, models) else {
            return;
        };
        let size = extent(name, scale);
        if cursor + size.x > length {
            return;
        }
        let edge_point = start + direction * (cursor + size.x * 0.5);
        let center = edge_point - facing * (size.z * 0.5);
        put(sheet, name, center, yaw_of(facing), scale, true);
        if neon {
            neon_on(sheet, random, center, facing, size.x, size.z, size.y);
        }
        if let Some(room) = pick(random, rooms) {
            sheet.doors.push(Door {
                position: edge_point + facing * DOOR_OUT,
                facing,
                room: *room,
                size: Vec2::new(size.x, size.z),
            });
        }
        cursor += size.x + between(random, gap[0], gap[1]);
    }
}

fn ring(
    sheet: &mut Sheet,
    random: &mut Random,
    models: &[&str],
    scale: f32,
    gap: [f32; 2],
    neon: bool,
    rooms: &[Room],
) {
    let origin = block_origin(sheet.key);
    let span = block_span();
    let west = Vec3::new(-1.0, 0.0, 0.0);
    let east = Vec3::new(1.0, 0.0, 0.0);
    let south = Vec3::new(0.0, 0.0, -1.0);
    let north = Vec3::new(0.0, 0.0, 1.0);
    fill_edge(
        sheet, random, models, scale, origin, north, span, west, gap, neon, rooms,
    );
    fill_edge(
        sheet,
        random,
        models,
        scale,
        origin + Vec3::new(span, 0.0, 0.0),
        north,
        span,
        east,
        gap,
        neon,
        rooms,
    );
    fill_edge(
        sheet,
        random,
        models,
        scale,
        origin + Vec3::new(EDGE_INSET, 0.0, 0.0),
        east,
        span - EDGE_INSET * 2.0,
        south,
        gap,
        neon,
        rooms,
    );
    fill_edge(
        sheet,
        random,
        models,
        scale,
        origin + Vec3::new(EDGE_INSET, 0.0, span),
        east,
        span - EDGE_INSET * 2.0,
        north,
        gap,
        neon,
        rooms,
    );
}

fn scatter(sheet: &mut Sheet, random: &mut Random, models: &[&str], wanted: usize, inset: f32) {
    let origin = block_origin(sheet.key);
    let span = block_span();
    for _ in 0..wanted {
        let Some(name) = pick(random, models) else {
            return;
        };
        for _ in 0..PROP_TRIES {
            let spot = origin
                + Vec3::new(
                    between(random, inset, span - inset),
                    0.0,
                    between(random, inset, span - inset),
                );
            let yaw = whole(random, 0, 3) as f32 * std::f32::consts::FRAC_PI_2;
            if prop_at(sheet, name, spot, yaw) {
                break;
            }
        }
    }
}

fn wanted(sheet: &Sheet, random: &mut Random, low: usize, high: usize) -> usize {
    (count(random, low, high) as f32 * sheet.plan.props).round() as usize
}

fn cafe(sheet: &mut Sheet, random: &mut Random, at: Vec3, along: Vec3, inward: Vec3) -> f32 {
    let (Some(table), Some(chair)) = (pick(random, &CAFE_TABLES), pick(random, &CAFE_CHAIRS))
    else {
        return 0.0;
    };
    let table_size = extent(table, kit_scale(table));
    let chair_size = extent(chair, kit_scale(chair));
    let reach = chair_size.z + CAFE_ROOM;
    let center = at
        + along * (reach + table_size.x * 0.5)
        + inward * (chair_size.x.max(table_size.z) * 0.5 + FACE_GAP);
    prop_at(sheet, table, center, yaw_of(inward));
    let seat = table_size.x * 0.5 + chair_size.z * 0.5 + CAFE_ROOM;
    prop_at(sheet, chair, center - along * seat, yaw_of(along));
    prop_at(sheet, chair, center + along * seat, yaw_of(-along));
    reach * 2.0 + table_size.x
}

fn line(
    sheet: &mut Sheet,
    random: &mut Random,
    models: &[&str],
    start: Vec3,
    along: Vec3,
    length: f32,
    inward: Vec3,
    gap: [f32; 2],
    cafes: bool,
) {
    let mut cursor = between(random, 0.0, gap[1]);
    while cursor < length {
        if cafes && chance(random, CAFE_CHANCE) {
            let used = cafe(sheet, random, start + along * cursor, along, inward);
            cursor += used + between(random, gap[0], gap[1]);
            continue;
        }
        let Some(name) = pick(random, models) else {
            return;
        };
        let size = extent(name, kit_scale(name));
        let end = cursor + size.x;
        if end > length {
            return;
        }
        let middle = cursor + size.x * 0.5;
        let center = start + along * middle + inward * (size.z * 0.5 + FACE_GAP);
        prop_at(sheet, name, center, yaw_of(inward));
        cursor = end + between(random, gap[0], gap[1]);
    }
}

fn sidewalks(sheet: &mut Sheet, random: &mut Random) {
    let density = sheet.plan.props;
    if density <= 0.0 {
        return;
    }
    let origin = block_origin(sheet.key);
    let span = block_span();
    let gap = [SIDEWALK_GAP[0] / density, SIDEWALK_GAP[1] / density];
    let (models, cafes): (&[&str], bool) = match sheet.district {
        District::Downtown | District::Commercial => (&SIDEWALK_PROPS, true),
        District::Industrial => (&ALLEY_PROPS, false),
        District::Suburban => (&SIDEWALK_PROPS, false),
    };
    let length = span - TILE * 2.0;
    let street = TILE + PAVEMENT * 2.0;
    let north = Vec3::new(0.0, 0.0, 1.0);
    let east = Vec3::new(1.0, 0.0, 0.0);
    line(
        sheet,
        random,
        models,
        origin + Vec3::new(0.0, 0.0, TILE),
        north,
        length,
        -east,
        gap,
        cafes,
    );
    line(
        sheet,
        random,
        models,
        origin + Vec3::new(-street, 0.0, TILE),
        north,
        length,
        east,
        gap,
        cafes,
    );
    line(
        sheet,
        random,
        models,
        origin + Vec3::new(TILE, 0.0, 0.0),
        east,
        length,
        -north,
        gap,
        cafes,
    );
    line(
        sheet,
        random,
        models,
        origin + Vec3::new(TILE, 0.0, -street),
        east,
        length,
        north,
        gap,
        cafes,
    );
}

fn site(sheet: &mut Sheet, random: &mut Random) {
    let origin = block_origin(sheet.key);
    let span = block_span();
    let curb = tile_center(sheet.key, 0, 0).z + CURB_LINE;
    let mut along = origin.x + TILE + between(random, 0.0, CURB_STEP);
    while along < origin.x + span - TILE {
        if let Some(name) = pick(random, &CURB_PROPS) {
            prop_at(
                sheet,
                name,
                Vec3::new(along, 0.0, curb),
                std::f32::consts::FRAC_PI_2,
            );
        }
        along += CURB_STEP;
    }
    let towers = wanted(sheet, random, 1, 3);
    for _ in 0..towers {
        let Some(name) = pick(random, &SCAFFOLDING) else {
            break;
        };
        for _ in 0..PROP_TRIES {
            let spot = origin
                + Vec3::new(
                    between(random, EDGE_INSET, span - EDGE_INSET),
                    0.0,
                    between(random, EDGE_INSET, span - EDGE_INSET),
                );
            let yaw = whole(random, 0, 3) as f32 * std::f32::consts::FRAC_PI_2;
            if prop_at(sheet, name, spot, yaw) {
                break;
            }
        }
    }
    let props = wanted(sheet, random, 6, 12);
    scatter(sheet, random, &SITE_PROPS, props, EDGE_INSET);
}

fn downtown(sheet: &mut Sheet, random: &mut Random) {
    let origin = block_origin(sheet.key);
    let span = block_span();
    let center = origin + Vec3::new(span * 0.5, 0.0, span * 0.5);
    let Some(tower) = pick(random, &TOWERS) else {
        return;
    };
    let quarter = whole(random, 0, 3) as f32 * std::f32::consts::FRAC_PI_2;
    put(sheet, tower, center, quarter, TOWER_SCALE, true);
    let size = extent(tower, TOWER_SCALE);
    let south = Vec3::new(0.0, 0.0, -1.0);
    sheet.doors.push(Door {
        position: center + south * (size.z * 0.5 + DOOR_OUT),
        facing: south,
        room: Room::Lobby,
        size: Vec2::new(size.x, size.z),
    });
    for facing in [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, -1.0),
    ] {
        neon_on(sheet, random, center, facing, size.x, size.z, size.y);
        neon_on(sheet, random, center, facing, size.x, size.z, size.y);
    }
    ring(
        sheet,
        random,
        &COMMERCIAL_BUILDINGS,
        KIT_SCALE,
        SHOP_GAP,
        true,
        &SHOP_ROOMS,
    );
    let alley = wanted(sheet, random, 2, 4);
    scatter(sheet, random, &ALLEY_PROPS, alley, EDGE_INSET * 0.5);
}

fn midtown(sheet: &mut Sheet, random: &mut Random) {
    let mut models: Vec<&str> = COMMERCIAL_BUILDINGS.to_vec();
    if chance(random, sheet.plan.tower_chance) {
        models.extend(TOWERS);
    }
    ring(
        sheet,
        random,
        &models,
        KIT_SCALE,
        SHOP_GAP,
        true,
        &SHOP_ROOMS,
    );
    parking(sheet, random, &CARS);
    let origin = block_origin(sheet.key);
    let span = block_span();
    for step in 0..4 {
        let along = origin.x + 12.0 + step as f32 * (span - 24.0) / 3.0;
        put(
            sheet,
            PLANTER,
            Vec3::new(along, 0.0, origin.z + span * 0.5),
            0.0,
            KIT_SCALE,
            false,
        );
        if let Some(tree) = pick(random, &TREES) {
            put(
                sheet,
                tree,
                Vec3::new(along, 0.0, origin.z + span * 0.5),
                between(random, 0.0, std::f32::consts::TAU),
                KIT_SCALE,
                false,
            );
        }
    }
    let alley = wanted(sheet, random, 3, 6);
    scatter(sheet, random, &ALLEY_PROPS, alley, EDGE_INSET * 0.5);
}

fn works(sheet: &mut Sheet, random: &mut Random) {
    ring(
        sheet,
        random,
        &INDUSTRIAL_BUILDINGS,
        KIT_SCALE,
        YARD_GAP,
        false,
        &WORKS_ROOMS,
    );
    let origin = block_origin(sheet.key);
    let span = block_span();
    let props = count(random, PROP_COUNT[0], PROP_COUNT[1]);
    for _ in 0..props {
        let Some(prop) = pick(random, &INDUSTRIAL_PROPS) else {
            continue;
        };
        let spot = origin
            + Vec3::new(
                between(random, 12.0, span - 12.0),
                0.0,
                between(random, 12.0, span - 12.0),
            );
        put(
            sheet,
            prop,
            spot,
            whole(random, 0, 3) as f32 * std::f32::consts::FRAC_PI_2,
            KIT_SCALE,
            true,
        );
    }
    parking(sheet, random, &TRUCKS);
    let yard = wanted(sheet, random, 5, 10);
    scatter(sheet, random, &SITE_PROPS, yard, EDGE_INSET * 0.75);
    let alley = wanted(sheet, random, 2, 4);
    scatter(sheet, random, &ALLEY_PROPS, alley, EDGE_INSET * 0.75);
}

fn rows(sheet: &mut Sheet, random: &mut Random) {
    ring(
        sheet,
        random,
        &HOUSES,
        KIT_SCALE,
        YARD_GAP,
        false,
        &HOME_ROOMS,
    );
    let origin = block_origin(sheet.key);
    let span = block_span();
    let trees = count(random, sheet.plan.trees / 2, sheet.plan.trees);
    for _ in 0..trees {
        let Some(tree) = pick(random, &TREES) else {
            continue;
        };
        let spot = origin
            + Vec3::new(
                between(random, 10.0, span - 10.0),
                0.0,
                between(random, 10.0, span - 10.0),
            );
        put(
            sheet,
            tree,
            spot,
            between(random, 0.0, std::f32::consts::TAU),
            KIT_SCALE,
            false,
        );
    }
    let fence = extent(FENCE, KIT_SCALE).x;
    let mut along = EDGE_INSET;
    while along + fence <= span - EDGE_INSET {
        put(
            sheet,
            FENCE,
            origin + Vec3::new(along + fence * 0.5, 0.0, span * 0.5),
            0.0,
            KIT_SCALE,
            false,
        );
        along += fence;
    }
    let yard = wanted(sheet, random, 3, 8);
    scatter(sheet, random, &YARD_PROPS, yard, EDGE_INSET * 0.75);
}

fn parking(sheet: &mut Sheet, random: &mut Random, models: &[&str]) {
    let origin = block_origin(sheet.key);
    let span = block_span();
    for row in PARKING_ROWS.map(|row| row * span) {
        let mut along = EDGE_INSET;
        let mut flip = false;
        while along < span - EDGE_INSET {
            if chance(random, sheet.plan.parking)
                && let Some(car) = pick(random, models)
            {
                let yaw = if flip {
                    std::f32::consts::FRAC_PI_2
                } else {
                    -std::f32::consts::FRAC_PI_2
                };
                put(
                    sheet,
                    car,
                    origin + Vec3::new(along, 0.0, row),
                    yaw,
                    CAR_SCALE,
                    true,
                );
            }
            along += PARKING_STEP;
            flip = !flip;
        }
    }
}
