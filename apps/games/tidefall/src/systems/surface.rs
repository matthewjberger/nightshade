use crate::commands::{draw_beside, draw_between, draw_clear_direction, draw_direction, draw_near};
use crate::components::{Bulk, Draft, Gust, Relic, Speaker};
use crate::data::{
    BEACON_HEIGHT, BEACON_WIDTH, BOULDER_SHADE, BOULDER_SIZE, BOULDERS_PER_AREA, BodyPlan,
    CANOPY_COLOR, CANOPY_HEIGHT, CANOPY_WIDTH, CRATER_ANGLE, CRATER_RIM, CRATER_STONE,
    CRATERS_PER_AREA, DOME_SHADE, DOME_SIZE, DOMES_PER_AREA, DRAFT_LIFT, DRAFT_RADIUS,
    FROZEN_COLOR, FROZEN_PER_AREA, HILL_FLATTEN, HILL_SHADE, HILL_SIZE, HILLS_PER_AREA,
    HUT_KEEP_OUT, HUT_ROOF, HUT_ROOF_HEIGHT, HUT_SIZE, HUT_SPREAD, HUT_WALL, HUTS,
    ICE_SPIRE_FACTOR, Kind, LAKE_COLOR, LAKE_SIZE, LAKES_PER_AREA, LAVA_COLOR, LAVA_GLOW,
    LAVA_GLOW_STRENGTH, LAVA_SIZE, MARIA_PER_AREA, MARIA_SHADE, MARIA_SIZE, MARKER_SPEECH,
    MOUNTAIN_HEIGHT, MOUNTAINS_PER_AREA, PAD_COLOR, PAD_HEIGHT, PAD_RADIUS, PILLAR_HEIGHT,
    PILLARS_PER_AREA, POOLS_PER_AREA, RELIC_SIZE, RELICS_PER_WORLD, RING_ISLAND_SIZE, RING_ISLANDS,
    RING_SPREAD, SHARD_COLOR, SHARD_GLOW, SHARD_GLOW_STRENGTH, SHELL_HEIGHT, SHELL_ROCKS_PER_AREA,
    SHELL_SIZE, SPIRE_HEIGHT, SPIRE_SHADE, SPIRE_WIDTH, SPIRES_PER_AREA, TREES_PER_AREA,
    TRUNK_COLOR, TRUNK_HEIGHT, TRUNK_WIDTH, VENT_GLOW, VENT_GLOW_STRENGTH, VENT_HEIGHT, VENT_SHADE,
    VENT_WIDTH, VENTS_PER_AREA, VILLAGER_COATS, VILLAGER_HEAD, VILLAGER_HEIGHT, VILLAGER_SKIN,
    VILLAGER_STANDOFF, VILLAGER_WIDTH,
};
use crate::queries::{shade, up_to};
use nightshade::ecs::ChildOf;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::material::Material;

pub fn scatter(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    glow: MaterialRef,
    body: Entity,
    plan: &BodyPlan,
    world: usize,
    pad: Option<Vec3>,
) {
    let stone = paint(
        registry,
        plan,
        "stone",
        shade(plan.color, BOULDER_SHADE),
        0.95,
        0.0,
    );
    let crystal = paint(
        registry,
        plan,
        "crystal",
        shade(plan.color, SPIRE_SHADE),
        0.4,
        0.1,
    );
    let ground = paint(
        registry,
        plan,
        "ground",
        shade(plan.color, HILL_SHADE),
        plan.roughness,
        plan.metallic,
    );
    let shard = glowing(
        registry,
        plan,
        "shard",
        SHARD_COLOR,
        SHARD_GLOW,
        SHARD_GLOW_STRENGTH,
    );
    let area = plan.radius * plan.radius;
    let first = world * RELICS_PER_WORLD;

    let beacon = draw_clear_direction(roll, pad);
    let mut peaks: Vec<(Vec3, f32)> = Vec::new();
    let mut spires: Vec<Vec3> = Vec::new();

    if plan.kind != Kind::Giant {
        for _ in 0..count(area, HILLS_PER_AREA) {
            let size = draw_between(roll, HILL_SIZE.0, HILL_SIZE.1);
            let direction = draw_clear_direction(roll, pad);
            plant(
                commands,
                body,
                plan,
                "Sphere",
                direction,
                0.0,
                Vec3::new(size, size * HILL_FLATTEN, size),
                ground,
                Some(size * HILL_FLATTEN),
            );
        }
        for _ in 0..count(area, MOUNTAINS_PER_AREA) {
            let height = draw_between(roll, MOUNTAIN_HEIGHT.0, MOUNTAIN_HEIGHT.1);
            let width = height * draw_between(roll, 0.9, 1.6);
            let direction = draw_clear_direction(roll, pad);
            plant(
                commands,
                body,
                plan,
                "Cone",
                direction,
                height * 0.5 - 0.5,
                Vec3::new(width, height, width),
                stone,
                Some(height * 0.3),
            );
            peaks.push((direction, height));
        }
        for _ in 0..count(area, BOULDERS_PER_AREA) {
            let size = draw_between(roll, BOULDER_SIZE.0, BOULDER_SIZE.1);
            let direction = draw_clear_direction(roll, pad);
            boulder(commands, body, plan, direction, size * 0.55, size, stone);
        }
        let spire_factor = if plan.kind == Kind::Ice {
            ICE_SPIRE_FACTOR
        } else {
            1.0
        };
        for _ in 0..count(area, SPIRES_PER_AREA * spire_factor) + 3 {
            let height = draw_between(roll, SPIRE_HEIGHT.0, SPIRE_HEIGHT.1);
            let width = draw_between(roll, SPIRE_WIDTH.0, SPIRE_WIDTH.1);
            let direction = draw_clear_direction(roll, pad);
            plant(
                commands,
                body,
                plan,
                "Cone",
                direction,
                height * 0.5 - 0.3,
                Vec3::new(width, height, width),
                crystal,
                Some(height * 0.3),
            );
            spires.push(direction);
        }
    }

    match plan.kind {
        Kind::Scorched => {
            let vents = vent(commands, registry, roll, body, plan, stone);
            pools(commands, registry, roll, body, plan);
            for (index, direction) in vents.iter().take(RELICS_PER_WORLD).enumerate() {
                let spot = draw_near(roll, *direction, 0.03);
                relic(commands, body, plan, spot, 0.0, first + index, shard);
            }
        }
        Kind::Home => {
            grow(commands, registry, roll, body, plan, pad);
            lakes(commands, registry, roll, body, plan, pad);
            if let Some(pad) = pad {
                let huts = settle(commands, registry, roll, body, plan, pad);
                dwell(commands, registry, roll, body, plan, &huts);
                let by_stone = draw_near(roll, beacon, 0.012);
                relic(commands, body, plan, by_stone, 0.0, first, shard);
                let hamlet = huts.first().copied().unwrap_or(pad);
                let by_hut = draw_near(roll, hamlet, 0.02);
                relic(commands, body, plan, by_hut, 0.0, first + 1, shard);
                let tallest = peaks
                    .iter()
                    .max_by(|(_, first), (_, second)| first.total_cmp(second))
                    .map(|(direction, _)| *direction)
                    .unwrap_or(beacon);
                let by_peak = draw_near(roll, tallest, 0.03);
                relic(commands, body, plan, by_peak, 0.0, first + 2, shard);
            }
        }
        Kind::Moon => {
            let bowls = crater(commands, roll, body, plan, stone);
            maria(commands, registry, roll, body, plan);
            for (index, center) in bowls.iter().take(RELICS_PER_WORLD).enumerate() {
                relic(commands, body, plan, *center, 0.0, first + index, shard);
            }
        }
        Kind::Shell => {
            let rocks = shell(commands, roll, body, plan, stone);
            pillars(commands, roll, body, plan, stone);
            for (index, direction) in rocks.iter().take(RELICS_PER_WORLD).enumerate() {
                relic(commands, body, plan, *direction, 0.0, first + index, shard);
            }
        }
        Kind::Giant => {
            domes(commands, registry, roll, body, plan);
            let perches = islands(commands, roll, body, plan, stone);
            for (index, (direction, top)) in perches.iter().take(RELICS_PER_WORLD).enumerate() {
                relic(commands, body, plan, *direction, *top, first + index, shard);
            }
        }
        Kind::Ice => {
            frozen(commands, registry, roll, body, plan);
            for (index, direction) in spires.iter().take(RELICS_PER_WORLD).enumerate() {
                let spot = draw_near(roll, *direction, 0.015);
                relic(commands, body, plan, spot, 0.0, first + index, shard);
            }
        }
    }

    if let Some(pad) = pad {
        let surface = paint(registry, plan, "pad", PAD_COLOR, 0.7, 0.3);
        plant(
            commands,
            body,
            plan,
            "Cylinder",
            pad,
            0.0,
            Vec3::new(PAD_RADIUS * 2.0, PAD_HEIGHT, PAD_RADIUS * 2.0),
            surface,
            None,
        );
    }

    let post = plant(
        commands,
        body,
        plan,
        "Cylinder",
        beacon,
        BEACON_HEIGHT * 0.5 - 0.3,
        Vec3::new(BEACON_WIDTH, BEACON_HEIGHT, BEACON_WIDTH),
        stone,
        Some(BEACON_HEIGHT * 0.25),
    );
    commands.set(post, Speaker(MARKER_SPEECH + world));
    plant(
        commands,
        body,
        plan,
        "Sphere",
        beacon,
        BEACON_HEIGHT + 0.5,
        Vec3::repeat(BEACON_WIDTH),
        glow,
        Some(BEACON_WIDTH * 0.9),
    );
}

fn count(area: f32, per_area: f32) -> usize {
    (area * per_area) as usize
}

fn paint(
    registry: &mut MaterialRegistry,
    plan: &BodyPlan,
    part: &str,
    color: [f32; 4],
    roughness: f32,
    metallic: f32,
) -> MaterialRef {
    MaterialRef::new(insert_material(
        registry,
        format!("tidefall_{}_{part}", plan.name.to_lowercase()),
        Material {
            base_color: color,
            roughness,
            metallic,
            ..Default::default()
        },
    ))
}

fn glowing(
    registry: &mut MaterialRegistry,
    plan: &BodyPlan,
    part: &str,
    color: [f32; 4],
    emissive: [f32; 3],
    strength: f32,
) -> MaterialRef {
    MaterialRef::new(insert_material(
        registry,
        format!("tidefall_{}_{part}", plan.name.to_lowercase()),
        Material {
            base_color: color,
            emissive_factor: emissive,
            emissive_strength: strength,
            unlit: true,
            ..Default::default()
        },
    ))
}

fn boulder(
    commands: &mut Commands,
    body: Entity,
    plan: &BodyPlan,
    direction: Vec3,
    lift: f32,
    size: f32,
    material: MaterialRef,
) {
    plant(
        commands,
        body,
        plan,
        "Sphere",
        direction,
        lift,
        Vec3::new(size, size * 0.75, size),
        material,
        Some(size * 0.85),
    );
}

fn disc(
    commands: &mut Commands,
    body: Entity,
    plan: &BodyPlan,
    direction: Vec3,
    size: f32,
    material: MaterialRef,
) {
    plant(
        commands,
        body,
        plan,
        "Cylinder",
        direction,
        0.0,
        Vec3::new(size, 0.3, size),
        material,
        None,
    );
}

fn relic(
    commands: &mut Commands,
    body: Entity,
    plan: &BodyPlan,
    direction: Vec3,
    lift: f32,
    index: usize,
    material: MaterialRef,
) {
    let entity = plant(
        commands,
        body,
        plan,
        "Cone",
        direction,
        lift + RELIC_SIZE,
        Vec3::new(RELIC_SIZE, RELIC_SIZE * 2.0, RELIC_SIZE),
        material,
        Some(RELIC_SIZE * 0.8),
    );
    commands.set(entity, Relic(index));
}

fn grow(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    pad: Option<Vec3>,
) {
    let trunk = paint(registry, plan, "trunk", TRUNK_COLOR, 0.95, 0.0);
    let canopy = paint(registry, plan, "canopy", CANOPY_COLOR, 0.85, 0.0);
    for _ in 0..count(plan.radius * plan.radius, TREES_PER_AREA) {
        let direction = draw_clear_direction(roll, pad);
        let trunk_height = draw_between(roll, TRUNK_HEIGHT.0, TRUNK_HEIGHT.1);
        let canopy_height = draw_between(roll, CANOPY_HEIGHT.0, CANOPY_HEIGHT.1);
        let canopy_width = draw_between(roll, CANOPY_WIDTH.0, CANOPY_WIDTH.1);
        plant(
            commands,
            body,
            plan,
            "Cylinder",
            direction,
            trunk_height * 0.5 - 0.2,
            Vec3::new(TRUNK_WIDTH, trunk_height, TRUNK_WIDTH),
            trunk,
            Some(TRUNK_WIDTH),
        );
        plant(
            commands,
            body,
            plan,
            "Cone",
            direction,
            trunk_height + canopy_height * 0.5 - 0.4,
            Vec3::new(canopy_width, canopy_height, canopy_width),
            canopy,
            None,
        );
    }
}

fn lakes(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    pad: Option<Vec3>,
) {
    let water = paint(registry, plan, "water", LAKE_COLOR, 0.08, 0.0);
    for _ in 0..count(plan.radius * plan.radius, LAKES_PER_AREA) {
        let size = draw_between(roll, LAKE_SIZE.0, LAKE_SIZE.1);
        let direction = draw_clear_direction(roll, pad);
        disc(commands, body, plan, direction, size, water);
    }
}

fn settle(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    pad: Vec3,
) -> Vec<Vec3> {
    let wall = paint(registry, plan, "wall", HUT_WALL, 0.9, 0.0);
    let roof = paint(registry, plan, "roof", HUT_ROOF, 0.8, 0.0);
    let mut huts = Vec::with_capacity(HUTS);
    for _ in 0..HUTS {
        let mut direction = draw_near(roll, pad, HUT_SPREAD);
        while direction.dot(&pad) > HUT_KEEP_OUT {
            direction = draw_near(roll, pad, HUT_SPREAD);
        }
        plant(
            commands,
            body,
            plan,
            "Cube",
            direction,
            HUT_SIZE * 0.4,
            Vec3::new(HUT_SIZE, HUT_SIZE * 0.8, HUT_SIZE),
            wall,
            Some(HUT_SIZE * 0.7),
        );
        plant(
            commands,
            body,
            plan,
            "Cone",
            direction,
            HUT_SIZE * 0.8 + HUT_ROOF_HEIGHT * 0.5 - 0.2,
            Vec3::new(HUT_SIZE * 1.3, HUT_ROOF_HEIGHT, HUT_SIZE * 1.3),
            roof,
            None,
        );
        huts.push(direction);
    }
    huts
}

fn dwell(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    huts: &[Vec3],
) {
    let skin = paint(registry, plan, "skin", VILLAGER_SKIN, 0.8, 0.0);
    let coats: Vec<MaterialRef> = VILLAGER_COATS
        .iter()
        .enumerate()
        .map(|(index, color)| paint(registry, plan, &format!("coat{index}"), *color, 0.9, 0.0))
        .collect();
    for index in 0..MARKER_SPEECH {
        let Some(hut) = huts.get(index % huts.len().max(1)) else {
            return;
        };
        let direction = draw_beside(roll, *hut, HUT_SIZE * VILLAGER_STANDOFF / plan.radius);
        let figure = plant(
            commands,
            body,
            plan,
            "Cylinder",
            direction,
            VILLAGER_HEIGHT * 0.5,
            Vec3::new(VILLAGER_WIDTH, VILLAGER_HEIGHT, VILLAGER_WIDTH),
            coats[index % coats.len()],
            Some(VILLAGER_WIDTH * 0.7),
        );
        commands.set(figure, Speaker(index));
        plant(
            commands,
            body,
            plan,
            "Sphere",
            direction,
            VILLAGER_HEIGHT + VILLAGER_HEAD * 0.9,
            Vec3::repeat(VILLAGER_HEAD),
            skin,
            None,
        );
    }
}

fn vent(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    stone: MaterialRef,
) -> Vec<Vec3> {
    let mouth = paint(
        registry,
        plan,
        "mouth",
        shade(plan.color, VENT_SHADE),
        0.95,
        0.0,
    );
    let ember = glowing(
        registry,
        plan,
        "ember",
        [1.0, 0.6, 0.3, 1.0],
        VENT_GLOW,
        VENT_GLOW_STRENGTH,
    );
    let mut vents = Vec::new();
    for _ in 0..count(plan.radius * plan.radius, VENTS_PER_AREA) + 3 {
        let direction = draw_direction(roll);
        plant(
            commands,
            body,
            plan,
            "Cone",
            direction,
            VENT_HEIGHT * 0.5 - 0.3,
            Vec3::new(VENT_WIDTH, VENT_HEIGHT, VENT_WIDTH),
            mouth,
            Some(VENT_WIDTH * 0.35),
        );
        let entity = plant(
            commands,
            body,
            plan,
            "Sphere",
            direction,
            VENT_HEIGHT * 0.9,
            Vec3::repeat(VENT_WIDTH * 0.3),
            ember,
            None,
        );
        commands.set(entity, Draft(DRAFT_RADIUS));
        commands.set(entity, Gust(DRAFT_LIFT));
        for _ in 0..3 {
            let near = draw_near(roll, direction, 0.06);
            let size = draw_between(roll, 1.0, 2.5);
            boulder(commands, body, plan, near, size * 0.5, size, stone);
        }
        vents.push(direction);
    }
    vents
}

fn pools(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
) {
    let lava = glowing(
        registry,
        plan,
        "lava",
        LAVA_COLOR,
        LAVA_GLOW,
        LAVA_GLOW_STRENGTH,
    );
    for _ in 0..count(plan.radius * plan.radius, POOLS_PER_AREA) {
        let size = draw_between(roll, LAVA_SIZE.0, LAVA_SIZE.1);
        let direction = draw_direction(roll);
        disc(commands, body, plan, direction, size, lava);
    }
}

fn crater(
    commands: &mut Commands,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    stone: MaterialRef,
) -> Vec<Vec3> {
    let mut bowls = Vec::new();
    for _ in 0..count(plan.radius * plan.radius, CRATERS_PER_AREA) + 3 {
        let center = draw_direction(roll);
        let side = center.cross(&draw_direction(roll)).normalize();
        let other = center.cross(&side);
        for index in 0..CRATER_RIM {
            let angle = index as f32 / CRATER_RIM as f32 * std::f32::consts::TAU;
            let direction =
                (center + (side * angle.cos() + other * angle.sin()) * CRATER_ANGLE).normalize();
            let size = draw_between(roll, CRATER_STONE.0, CRATER_STONE.1);
            boulder(commands, body, plan, direction, size * 0.5, size, stone);
        }
        bowls.push(center);
    }
    bowls
}

fn maria(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
) {
    let dark = paint(
        registry,
        plan,
        "maria",
        shade(plan.color, MARIA_SHADE),
        0.95,
        0.0,
    );
    for _ in 0..count(plan.radius * plan.radius, MARIA_PER_AREA) + 2 {
        let size = draw_between(roll, MARIA_SIZE.0, MARIA_SIZE.1);
        disc(commands, body, plan, draw_direction(roll), size, dark);
    }
}

fn shell(
    commands: &mut Commands,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    stone: MaterialRef,
) -> Vec<Vec3> {
    let mut rocks = Vec::new();
    for _ in 0..count(plan.radius * plan.radius, SHELL_ROCKS_PER_AREA) + 3 {
        let direction = draw_direction(roll);
        let height = draw_between(roll, SHELL_HEIGHT.0, SHELL_HEIGHT.1);
        let size = draw_between(roll, SHELL_SIZE.0, SHELL_SIZE.1);
        boulder(commands, body, plan, direction, height, size, stone);
        rocks.push(direction);
    }
    rocks
}

fn pillars(
    commands: &mut Commands,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    stone: MaterialRef,
) {
    for _ in 0..count(plan.radius * plan.radius, PILLARS_PER_AREA) {
        let height = draw_between(roll, PILLAR_HEIGHT.0, PILLAR_HEIGHT.1);
        let width = height * draw_between(roll, 0.12, 0.25);
        plant(
            commands,
            body,
            plan,
            "Cylinder",
            draw_direction(roll),
            height * 0.5 - 0.5,
            Vec3::new(width, height, width),
            stone,
            Some(width * 0.6),
        );
    }
}

fn domes(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
) {
    let cloud = paint(
        registry,
        plan,
        "cloud",
        shade(plan.color, DOME_SHADE),
        0.7,
        0.0,
    );
    for _ in 0..count(plan.radius * plan.radius, DOMES_PER_AREA) {
        let size = draw_between(roll, DOME_SIZE.0, DOME_SIZE.1);
        plant(
            commands,
            body,
            plan,
            "Sphere",
            draw_direction(roll),
            0.0,
            Vec3::new(size, size * HILL_FLATTEN, size),
            cloud,
            None,
        );
    }
}

fn islands(
    commands: &mut Commands,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
    stone: MaterialRef,
) -> Vec<(Vec3, f32)> {
    let spread = plan.radius * RING_SPREAD;
    let mut perches = Vec::with_capacity(RING_ISLANDS);
    for index in 0..RING_ISLANDS {
        let angle = (index as f32 + roll.f32() * 0.6) / RING_ISLANDS as f32 * std::f32::consts::TAU;
        let direction = Vec3::new(angle.cos(), 0.0, angle.sin());
        let size = draw_between(roll, RING_ISLAND_SIZE.0, RING_ISLAND_SIZE.1);
        let lift = spread - plan.radius;
        boulder(commands, body, plan, direction, lift, size, stone);
        perches.push((direction, lift + size * 0.75));
    }
    perches
}

fn frozen(
    commands: &mut Commands,
    registry: &mut MaterialRegistry,
    roll: &mut fastrand::Rng,
    body: Entity,
    plan: &BodyPlan,
) {
    let ice = paint(registry, plan, "frozen", FROZEN_COLOR, 0.05, 0.2);
    for _ in 0..count(plan.radius * plan.radius, FROZEN_PER_AREA) + 2 {
        let size = draw_between(roll, LAKE_SIZE.0, LAKE_SIZE.1);
        disc(commands, body, plan, draw_direction(roll), size, ice);
    }
}

fn plant(
    commands: &mut Commands,
    body: Entity,
    plan: &BodyPlan,
    mesh: &str,
    direction: Vec3,
    lift: f32,
    size: Vec3,
    material: MaterialRef,
    bulk: Option<f32>,
) -> Entity {
    let entity = spawn_mesh_at(
        commands,
        mesh,
        direction * (1.0 + lift / plan.radius),
        size / plan.radius,
    );
    if let Some(transform) = commands.staged_mut::<LocalTransform>(entity) {
        transform.rotation = up_to(direction);
    }
    commands.restage::<LocalTransform>(entity);
    commands.set(entity, ChildOf(body));
    commands.set(entity, material);
    if let Some(bulk) = bulk {
        commands.set(entity, Bulk(bulk));
    }
    entity
}
