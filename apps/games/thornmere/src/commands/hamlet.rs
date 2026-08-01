use crate::commands::models::{place_model, plant_model};
use crate::commands::random::random_place_spot;
use crate::commands::scenery::{house, lantern, turned};
use crate::data::{
    FIELD, HAMLET, HAMLET_BARN, HAMLET_HOUSE, HAMLET_INN, HOUSE_CELL, Interior, ROAD_WIDTH,
    SCATTER_GAP, STARTER, STARTER_TREE_COUNT, STARTER_TREE_GAP, TREE_SCALE,
};
use crate::queries::{
    far_enough, ground, in_city, in_place, near_hamlet, road_distance, under_water,
};
use crate::resources::Palette;
use nightshade::prelude::*;
use nightshade_random::prelude::*;

const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
const PI: f32 = std::f32::consts::PI;
const TAU: f32 = std::f32::consts::TAU;

pub fn spawn_starter_wood(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    let oaks = [
        "tree_oak",
        "tree_default",
        "tree_detailed",
        "tree_fat",
        "tree_oak",
        "tree_default",
        "tree-high",
        "tree",
    ];
    let mut taken: Vec<Vec2> = Vec::new();
    let mut planted = 0;
    let mut attempts = 0;
    while planted < STARTER_TREE_COUNT && attempts < STARTER_TREE_COUNT * 12 {
        attempts += 1;
        let spot = random_place_spot(random, STARTER, 1.0);
        if road_distance(spot) < ROAD_WIDTH * 2.4
            || in_city(spot, 14.0)
            || in_place(spot, FIELD, 0.55)
            || near_hamlet(spot)
            || under_water(spot)
            || !far_enough(spot, &taken, STARTER_TREE_GAP)
        {
            continue;
        }
        taken.push(spot);
        planted += 1;
        let kind = oaks[random.usize(0..oaks.len())];
        let scale = if kind.starts_with("tree-") {
            4.4
        } else {
            TREE_SCALE * 1.3
        };
        plant_model(
            commands,
            kind,
            spot,
            random.f32() * TAU,
            scale * between(random, 0.85, 1.25),
        );
    }
    for _ in 0..70 {
        let spot = random_place_spot(random, STARTER, 1.0);
        if road_distance(spot) < ROAD_WIDTH * 1.4
            || in_city(spot, 12.0)
            || under_water(spot)
            || !far_enough(spot, &taken, SCATTER_GAP)
        {
            continue;
        }
        taken.push(spot);
        let piece = [
            "plant_bushDetailed",
            "plant_bushLarge",
            "mushroom_redGroup",
            "mushroom_tanGroup",
            "log",
            "stump_round",
            "rock_smallC",
            "flower_yellowA",
            "flower_purpleB",
            "grass_large",
        ][random.usize(0..10)];
        plant_model(commands, piece, spot, random.f32() * TAU, 2.8);
    }
    for index in 0..8 {
        let along = -46.0 + index as f32 * 8.0;
        for side in [-1.0, 1.0] {
            let spot = Vec2::new(side * 5.4, along);
            if index % 3 == 0 {
                lantern(
                    commands,
                    palette,
                    spot,
                    if side < 0.0 { FRAC_PI_2 } else { -FRAC_PI_2 },
                );
            } else {
                plant_model(commands, "fence_simple", spot, FRAC_PI_2, 2.4);
            }
        }
    }
    plant_model(commands, "signpost", Vec2::new(4.0, 6.0), -0.4, 3.2);
    hamlet(commands, palette, random);
}

fn hamlet(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    house(
        commands,
        palette,
        random,
        HAMLET_INN,
        -FRAC_PI_2,
        3,
        2,
        true,
        "roof-point",
        Interior::Inn,
    );
    house(
        commands,
        palette,
        random,
        HAMLET_HOUSE,
        FRAC_PI_2,
        2,
        1,
        false,
        "roof-point",
        Interior::Home,
    );
    house(
        commands,
        palette,
        random,
        HAMLET_BARN,
        PI,
        2,
        1,
        true,
        "roof-point",
        Interior::Warehouse,
    );
    let front = HAMLET_INN + turned(Vec2::new(0.0, HOUSE_CELL * 1.5 + 0.2), -FRAC_PI_2);
    place_model(
        commands,
        "banner-green",
        ground(front + Vec2::new(0.0, 2.2)) + Vec3::new(0.0, HOUSE_CELL * 1.2, 0.0),
        -PI,
        HOUSE_CELL,
    );
    plant_model(
        commands,
        "stall-bench",
        front + Vec2::new(-1.6, -3.0),
        0.0,
        3.0,
    );
    plant_model(commands, "barrel", front + Vec2::new(-1.4, 3.4), 0.3, 3.0);
    plant_model(
        commands,
        "barrel-open",
        front + Vec2::new(-1.2, 4.8),
        1.1,
        3.0,
    );
    plant_model(
        commands,
        "fountain-round",
        HAMLET + Vec2::new(-4.0, -2.0),
        0.0,
        3.0,
    );
    plant_model(
        commands,
        "cart-high",
        HAMLET + Vec2::new(-2.0, 6.0),
        0.6,
        3.0,
    );
    plant_model(
        commands,
        "hay-bale",
        HAMLET_BARN + Vec2::new(-6.0, 1.0),
        0.4,
        2.8,
    );
    plant_model(
        commands,
        "hay-bale-bundled",
        HAMLET_BARN + Vec2::new(-6.5, 3.6),
        1.2,
        2.8,
    );
    plant_model(
        commands,
        "log_stack",
        HAMLET_BARN + Vec2::new(6.0, 2.0),
        0.2,
        3.0,
    );
    plant_model(
        commands,
        "workbench",
        HAMLET_HOUSE + Vec2::new(0.0, 6.2),
        0.6,
        3.0,
    );
    plant_model(
        commands,
        "bucket",
        HAMLET_HOUSE + Vec2::new(2.0, 6.0),
        0.0,
        2.8,
    );
    for index in 0..6 {
        let x = HAMLET_HOUSE.x - 6.0 + index as f32 * 2.2;
        plant_model(
            commands,
            "fence_simple",
            Vec2::new(x, HAMLET_HOUSE.y + 8.0),
            0.0,
            2.2,
        );
    }
    for row in 0..3 {
        for column in 0..4 {
            let spot = HAMLET_HOUSE + Vec2::new(-5.0 + column as f32 * 2.0, 9.4 + row as f32 * 2.0);
            let crop = [
                "crops_wheatStageB",
                "crop_pumpkin",
                "crop_carrot",
                "crops_cornStageC",
            ][(row + column) % 4];
            plant_model(commands, crop, spot, between(random, -0.3, 0.3), 2.4);
        }
    }
    for offset in [
        Vec2::new(-7.0, -8.0),
        Vec2::new(8.0, -9.0),
        Vec2::new(22.0, 10.0),
    ] {
        plant_model(
            commands,
            "tree_oak",
            HAMLET + offset,
            random.f32() * TAU,
            TREE_SCALE * 1.4,
        );
    }
    lantern(commands, palette, HAMLET + Vec2::new(-1.5, 2.5), 0.0);
    lantern(
        commands,
        palette,
        HAMLET_INN + Vec2::new(-7.4, -5.0),
        FRAC_PI_2,
    );
    plant_model(
        commands,
        "signpost-single",
        HAMLET + Vec2::new(-6.0, 3.0),
        0.5,
        3.0,
    );
}
