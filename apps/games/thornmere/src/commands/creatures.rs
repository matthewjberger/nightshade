use crate::commands::models::{place_model, plant_model};
use crate::commands::random::*;
use crate::commands::scenery::prop;
use crate::components::{
    Aggro, Airborne, Approach, Beat, Bounty, Chill, Class, Cooldowns, Dying, Experience, Fade,
    Flinch, Giver, GiverMark, Goal, Guild, Heading, Health, Home, Hunt, Job, Kills, Leap, Leash,
    Level, Lift, Limb, Mana, MaxHealth, MaxMana, Merchant, Pause, Phase, Pickup, Pivot, Plate,
    PlateLevel, Post, Rank, Reach, Rebirth, Recovery, Regrow, Respawn, Rest, Rig, Rising, Speed,
    Sprint, Strength, Stride, Strike, Swing, Target, Task, Voice, Wander, Worth,
};
use crate::data::items::{MERCHANT_PLATE_COLOR, MERCHANTS};
use crate::data::*;
use crate::queries::*;
use crate::resources::Palette;
use crate::resources::ThornmereResources;
use crate::tags::Figure;
use nightshade::prelude::*;
use nightshade_camera_controller::prelude::*;
use nightshade_random::prelude::*;
use nightshade_render::data::asset_id::MaterialId;
use nightshade_scene::prelude::{ChildOf, GlobalTransform, LocalTransform, Name};
use nightshade_text::prelude::*;

pub fn spawn_plate(
    text_state: &mut TextState,
    commands: &mut Commands,
    position: Vec3,
    label: &str,
    color: Vec4,
    size: f32,
) -> Entity {
    spawn_3d_billboard_text_with_properties(
        text_state,
        commands,
        label,
        position,
        TextProperties {
            font_size: size,
            color,
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 1.5,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    )
}

fn root(commands: &mut Commands, spot: Vec2, heading: f32, name: String) -> Entity {
    commands.spawn((
        Name(name),
        LocalTransform {
            translation: ground(spot),
            rotation: nalgebra_glm::quat_angle_axis(heading, &Vec3::y()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
    ))
}

fn give_body(commands: &mut Commands, body: Entity) {
    commands.set(body, Phase(0.0));
    commands.set(body, Strike(0.0));
    commands.set(body, Flinch(0.0));
    commands.set(body, Dying(0.0));
    commands.set(body, Rising(0.0));
    commands.add_tag_type::<Figure>(body);
}

pub fn spawn_figure(
    commands: &mut Commands,
    spot: Vec2,
    heading: f32,
    model: &str,
    scale: f32,
    name: String,
) -> Entity {
    let body = plant_model(commands, model, spot, heading, scale);
    commands.set(body, Name(name));
    commands.set(body, Rig(None));
    give_body(commands, body);
    body
}

fn leg(
    commands: &mut Commands,
    body: Entity,
    size: f32,
    side: f32,
    depth: f32,
    sign: f32,
    material: MaterialId,
) {
    let pivot = Vec3::new(side, 0.5, depth) * size;
    let reach = Vec3::new(0.0, -0.25, 0.0) * size;
    let child = prop(
        commands,
        "Cube",
        pivot + reach,
        Vec3::new(0.18, 0.5, 0.18) * size,
        material,
    );
    commands.set(child, ChildOf(body));
    commands.set(child, Limb(sign));
    commands.set(child, Pivot(pivot));
    commands.set(child, Reach(reach));
}

pub fn spawn_quadruped(
    commands: &mut Commands,
    spot: Vec2,
    heading: f32,
    size: f32,
    material: MaterialId,
    snout: MaterialId,
    name: String,
) -> Entity {
    let body = root(commands, spot, heading, name);
    let mut part = |position: Vec3, scale: Vec3, material: MaterialId| {
        let child = prop(commands, "Cube", position * size, scale * size, material);
        commands.set(child, ChildOf(body));
    };
    part(
        Vec3::new(0.0, 0.55, 0.0),
        Vec3::new(0.8, 0.7, 1.3),
        material,
    );
    part(
        Vec3::new(0.0, 0.65, -0.85),
        Vec3::new(0.5, 0.45, 0.5),
        material,
    );
    part(Vec3::new(0.0, 0.5, -1.15), Vec3::new(0.22, 0.2, 0.2), snout);
    part(
        Vec3::new(0.0, 0.62, 0.85),
        Vec3::new(0.12, 0.12, 0.5),
        material,
    );
    for side in [-0.28, 0.28] {
        for depth in [-0.45, 0.45] {
            let sign = if (side < 0.0) == (depth < 0.0) {
                1.0
            } else {
                -1.0
            };
            leg(commands, body, size, side, depth, sign, material);
        }
    }
    give_body(commands, body);
    body
}

pub fn spawn_mobs(
    random: &mut Random,
    text_state: &mut TextState,
    commands: &mut Commands,
    palette: &Palette,
) {
    let mut taken: Vec<Vec2> = Vec::new();
    for (index, spawn) in SPAWNS.iter().enumerate() {
        for _ in 0..spawn.count {
            let spot = if spawn.rank >= ELITE {
                wild_spot(random, &taken, MOB_GAP * 0.5, |random| {
                    around(random, PLACES[spawn.place].center, 2.0, 6.0)
                })
            } else {
                wild_spot(random, &taken, MOB_GAP, |random| {
                    random_place_spot(random, spawn.place, 0.95)
                })
            };
            taken.push(spot);
            let heading = random.f32() * std::f32::consts::TAU;
            let body = match spawn.model {
                Some(model) => spawn_figure(
                    commands,
                    spot,
                    heading,
                    model,
                    spawn.model_scale,
                    spawn.name.to_string(),
                ),
                None => spawn_quadruped(
                    commands,
                    spot,
                    heading,
                    spawn.scale,
                    palette.mobs[index],
                    palette.stone,
                    spawn.name.to_string(),
                ),
            };
            let lift = if spawn.biped {
                PLATE_HEIGHT * spawn.scale
            } else {
                1.5 * spawn.scale
            };
            let elite = spawn.rank >= ELITE;
            let plate = spawn_plate(
                text_state,
                commands,
                ground(spot) + Vec3::new(0.0, lift, 0.0),
                &mob_plate(spawn.name, spawn.level, spawn.rank),
                if elite {
                    ELITE_PLATE_COLOR
                } else {
                    MONSTER_PLATE_COLOR
                },
                if elite {
                    PLATE_FONT_SIZE * 1.3
                } else {
                    PLATE_FONT_SIZE * 0.85
                },
            );
            let position = ground(spot);
            commands.set(body, Level(spawn.level));
            commands.set(body, Speed(spawn.speed));
            commands.set(body, Health(spawn.health));
            commands.set(body, MaxHealth(spawn.health));
            commands.set(body, Heading(heading));
            commands.set(body, Stride(0.0));
            commands.set(body, Home(position));
            commands.set(body, Goal(position));
            commands.set(body, Wander(between(random, 0.0, 4.0)));
            commands.set(body, Target(None));
            commands.set(body, Swing(between(random, 0.0, ATTACK_PERIOD)));
            commands.set(body, Strength(spawn.strength));
            commands.set(body, Aggro(spawn.aggro));
            commands.set(body, Leash(spawn.leash));
            commands.set(body, Worth(spawn.experience));
            commands.set(body, Bounty(spawn.coins));
            commands.set(body, Rebirth(spawn.respawn));
            commands.set(body, Respawn(0.0));
            commands.set(body, Rank(spawn.rank));
            commands.set(body, Chill(0.0));
            commands.set(body, Plate(plate));
            commands.set(body, Lift(lift));
        }
    }
}

pub fn spawn_people(random: &mut Random, text_state: &mut TextState, commands: &mut Commands) {
    let mut taken: Vec<Vec2> = Vec::new();
    for _ in 0..POPULATION {
        let level = random.u32(SIM_LEVEL_MIN..SIM_LEVEL_MAX);
        let class = random.usize(0..CLASSES.len());
        let guild = GUILDS[random.usize(0..GUILDS.len())].to_string();
        let name = random_name(random);
        let heading = random.f32() * std::f32::consts::TAU;
        let hunt = hunt_for_level(level);
        let camp = HUNTS[hunt].place;
        let giver_spot = GIVERS[HUNTS[hunt].giver].spot;

        let roll = random.f32();
        let (job, spot, goal, kills) = if roll < 0.22 {
            let spot = spread_spot(random, &taken, FOLK_GAP, |random| {
                around(random, BOARD_SPOT, 3.0, 13.0)
            });
            (Job::Resting, spot, ground(spot), 0)
        } else if roll < 0.5 {
            let spot = spread_spot(random, &taken, FOLK_GAP, |random| {
                around(random, SQUARE_SPOT, 2.0, 15.0)
            });
            (
                Job::ToCamp,
                spot,
                ground(random_place_spot(random, camp, 0.8)),
                0,
            )
        } else if roll < 0.82 {
            let spot = spread_spot(random, &taken, FOLK_GAP, |random| {
                random_place_spot(random, camp, 0.95)
            });
            (
                Job::Hunting,
                spot,
                ground(spot),
                random.u32(0..SIM_QUEST_KILLS),
            )
        } else {
            let spot = spread_spot(random, &taken, FOLK_GAP, |random| {
                random_place_spot(random, camp, 1.0)
            });
            (
                Job::ToGiver,
                spot,
                ground(around(random, giver_spot, 1.5, 4.5)),
                SIM_QUEST_KILLS,
            )
        };
        taken.push(spot);

        let (male, female) = CLASS_MODELS[class];
        let model = if random.bool() { male } else { female };
        let body = spawn_figure(commands, spot, heading, model, FOLK_SCALE, name.clone());
        let plate = spawn_plate(
            text_state,
            commands,
            ground(spot) + Vec3::new(0.0, PLATE_HEIGHT, 0.0),
            &person_plate(&name, &guild, level),
            CLASSES[class].plate,
            PLATE_FONT_SIZE,
        );

        let max_health = sim_max_health(level);
        commands.set(body, Speed(between(random, SIM_SPEED_MIN, SIM_SPEED_MAX)));
        commands.set(body, Health(max_health));
        commands.set(body, MaxHealth(max_health));
        commands.set(body, Heading(heading));
        commands.set(body, Stride(0.0));
        commands.set(body, Task(job));
        commands.set(body, Rest(between(random, 1.0, REST_MAX_SECONDS)));
        commands.set(body, Hunt(hunt));
        commands.set(body, Goal(goal));
        commands.set(body, Target(None));
        commands.set(body, Swing(between(random, 0.0, ATTACK_PERIOD)));
        commands.set(body, Strength(sim_strength(level)));
        commands.set(body, Chill(0.0));
        commands.set(body, Kills(kills));
        commands.set(body, Level(level));
        commands.set(body, Experience(random.u32(0..level_threshold(level))));
        commands.set(body, Plate(plate));
        commands.set(body, Lift(PLATE_HEIGHT));
        commands.set(body, PlateLevel(level));
        commands.set(body, Guild(guild));
        commands.set(body, Class(class));
        commands.set(body, Voice(random.usize(0..VOICE_COUNT)));
    }
}

pub fn spawn_givers(text_state: &mut TextState, commands: &mut Commands) {
    for (index, giver) in GIVERS.iter().enumerate() {
        let heading = heading_toward(ground(giver.spot), ground(BOARD_SPOT));
        let body = spawn_figure(
            commands,
            giver.spot,
            if giver.board { 0.0 } else { heading },
            giver.model,
            giver.model_scale,
            giver.name.to_string(),
        );
        let lift = if giver.board { 2.7 } else { PLATE_HEIGHT + 0.5 };
        let plate = spawn_plate(
            text_state,
            commands,
            ground(giver.spot) + Vec3::new(0.0, lift, 0.0),
            &giver_plate(index, 0),
            GIVER_PLATE_COLOR,
            PLATE_FONT_SIZE,
        );
        commands.set(body, Heading(heading));
        commands.set(body, Stride(0.0));
        commands.set(body, Health(1.0));
        commands.set(body, Giver(index));
        commands.set(body, GiverMark(0));
        commands.set(body, Plate(plate));
        commands.set(body, Lift(lift));
    }
}

pub fn spawn_merchants(text_state: &mut TextState, commands: &mut Commands) {
    for (index, merchant) in MERCHANTS.iter().enumerate() {
        let heading = heading_toward(ground(merchant.spot), ground(SQUARE_SPOT));
        let body = spawn_figure(
            commands,
            merchant.spot,
            heading,
            merchant.model,
            merchant.model_scale,
            merchant.name.to_string(),
        );
        let lift = PLATE_HEIGHT + 0.5;
        let plate = spawn_plate(
            text_state,
            commands,
            ground(merchant.spot) + Vec3::new(0.0, lift, 0.0),
            &format!(
                "{}
{}",
                merchant.title, merchant.name
            ),
            MERCHANT_PLATE_COLOR,
            PLATE_FONT_SIZE,
        );
        commands.set(body, Heading(heading));
        commands.set(body, Stride(0.0));
        commands.set(body, Health(1.0));
        commands.set(body, Merchant(index));
        commands.set(body, Plate(plate));
        commands.set(body, Lift(lift));
    }
}

pub fn spawn_pickups(random: &mut Random, commands: &mut Commands, palette: &Palette) {
    for (kind, pickup) in PICKUPS.iter().enumerate() {
        for _ in 0..pickup.count {
            let spot = random_place_spot(random, pickup.place, 0.95);
            let position = ground(spot);
            let node = place_model(
                commands,
                pickup.model,
                position,
                random.f32() * std::f32::consts::TAU,
                pickup.model_scale,
            );
            let glow = prop(
                commands,
                "Sphere",
                Vec3::new(0.0, 1.9, 0.0),
                Vec3::new(0.18, 0.18, 0.18),
                palette.pickups[kind],
            );
            commands.set(glow, ChildOf(node));
            commands.set(node, Pickup(kind));
            commands.set(node, Regrow(0.0));
            commands.set(node, Home(position));
        }
    }
}

pub fn spawn_ring(commands: &mut Commands) -> Entity {
    place_model(
        commands,
        RING_MODEL,
        Vec3::new(0.0, -SINK_DEPTH, 0.0),
        0.0,
        3.0,
    )
}

pub fn spawn_floating_pool(text_state: &mut TextState, commands: &mut Commands) -> Vec<Entity> {
    (0..FLOATING_COUNT)
        .map(|_| {
            let entity = spawn_plate(
                text_state,
                commands,
                Vec3::new(0.0, -SINK_DEPTH, 0.0),
                "",
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                PLATE_FONT_SIZE * 1.2,
            );
            commands.set(entity, Fade(0.0));
            entity
        })
        .collect()
}

pub fn spawn_player(
    resources: &mut ThornmereResources,
    text_state: &mut TextState,
    commands: &mut Commands,
) -> (Entity, Entity) {
    let class = &CALLINGS[resources.class];
    let name = resources.player_name.clone();
    let heading = 0.0;

    let body = spawn_figure(
        commands,
        PLAYER_START,
        heading,
        class.model,
        FOLK_SCALE,
        name.clone(),
    );
    let plate = spawn_plate(
        text_state,
        commands,
        ground(PLAYER_START) + Vec3::new(0.0, PLATE_HEIGHT, 0.0),
        &player_plate(&name, 1),
        PLAYER_PLATE_COLOR,
        PLATE_FONT_SIZE,
    );

    commands.set(body, Speed(class.speed));
    commands.set(body, Health(class.health));
    commands.set(body, MaxHealth(class.health));
    commands.set(body, Approach(None));
    commands.set(body, Leap(0.0));
    commands.set(body, Airborne(0.0));
    commands.set(body, Mana(class.mana));
    commands.set(body, MaxMana(class.mana));
    commands.set(body, Heading(heading));
    commands.set(body, Stride(0.0));
    commands.set(body, Cooldowns([0.0; ABILITY_COUNT]));
    commands.set(body, Recovery(0.0));
    commands.set(body, Sprint(0.0));
    commands.set(body, Chill(0.0));
    commands.set(body, Target(None));
    commands.set(body, Swing(0.0));
    commands.set(body, Strength(class.strength));
    commands.set(body, Level(1));
    commands.set(body, Experience(0));
    commands.set(body, Kills(0));
    commands.set(body, Plate(plate));
    commands.set(body, Lift(PLATE_HEIGHT));
    commands.set(body, PlateLevel(1));

    let camera = spawn_third_person_camera(
        commands,
        body,
        CAMERA_DISTANCE,
        heading,
        CAMERA_PITCH,
        "Player Camera".to_string(),
    );
    if let Some(third_person) = commands.staged_mut::<ThirdPersonCamera>(camera) {
        third_person.collision_enabled = false;
        third_person.distance_max = 20.0;
    }
    commands.restage::<ThirdPersonCamera>(camera);

    (body, camera)
}

pub fn spawn_stage_figure(commands: &mut Commands, class: usize) -> Entity {
    let data = &CALLINGS[class];
    let body = spawn_figure(
        commands,
        STAGE_SPOT,
        std::f32::consts::PI,
        data.model,
        FOLK_SCALE,
        data.name.to_string(),
    );
    commands.set(body, Heading(std::f32::consts::PI));
    commands.set(body, Stride(0.0));
    commands.set(body, Health(1.0));
    body
}

pub fn spawn_stage_camera(commands: &mut Commands, figure: Entity) -> Entity {
    let camera = spawn_third_person_camera(
        commands,
        figure,
        STAGE_DISTANCE,
        0.0,
        STAGE_PITCH,
        "Stage Camera".to_string(),
    );
    if let Some(third_person) = commands.staged_mut::<ThirdPersonCamera>(camera) {
        third_person.collision_enabled = false;
    }
    commands.restage::<ThirdPersonCamera>(camera);
    camera
}

fn plain_plate(
    text_state: &mut TextState,
    commands: &mut Commands,
    body: Entity,
    spot: Vec2,
    lift: f32,
    label: &str,
    color: Vec4,
) {
    let plate = spawn_plate(
        text_state,
        commands,
        ground(spot) + Vec3::new(0.0, lift, 0.0),
        label,
        color,
        PLATE_FONT_SIZE * 0.9,
    );
    commands.set(body, Plate(plate));
    commands.set(body, Lift(lift));
}

fn spawn_guard(
    text_state: &mut TextState,
    commands: &mut Commands,
    spot: Vec2,
    heading: f32,
) -> Entity {
    let body = spawn_figure(
        commands,
        spot,
        heading,
        GUARD_MODEL,
        GUARD_SCALE,
        GUARD_NAME.to_string(),
    );
    commands.set(body, Heading(heading));
    commands.set(body, Stride(0.0));
    commands.set(body, Health(1.0));
    commands.set(body, Speed(GUARD_SPEED));
    plain_plate(
        text_state,
        commands,
        body,
        spot,
        PLATE_HEIGHT + 0.3,
        &format!("<City Watch>\n{GUARD_NAME}"),
        GUARD_PLATE_COLOR,
    );
    body
}

pub fn spawn_guards(text_state: &mut TextState, commands: &mut Commands) {
    for (spot, heading) in GUARD_POSTS {
        spawn_guard(text_state, commands, spot, heading);
    }
    for (beat, route) in GUARD_ROUTES.iter().enumerate() {
        for start in [0, route.len() / 2] {
            let spot = route[start];
            let body = spawn_guard(text_state, commands, spot, 0.0);
            commands.set(body, Beat(beat));
            commands.set(body, Post((start + 1) % route.len()));
            commands.set(body, Pause(0.5 + start as f32));
        }
    }
}

pub fn spawn_townsfolk(text_state: &mut TextState, commands: &mut Commands) {
    for person in &TOWNSFOLK {
        let body = spawn_figure(
            commands,
            person.spot,
            person.heading,
            person.model,
            person.scale,
            person.name.to_string(),
        );
        commands.set(body, Heading(person.heading));
        commands.set(body, Stride(0.0));
        commands.set(body, Health(1.0));
        plain_plate(
            text_state,
            commands,
            body,
            person.spot,
            PLATE_HEIGHT + 0.3,
            &format!("<{}>\n{}", person.title, person.name),
            GIVER_PLATE_COLOR,
        );
    }
}

pub fn spawn_animals(random: &mut Random, commands: &mut Commands) {
    for (beat, route) in DOG_ROUTES.iter().enumerate() {
        let spot = route[0];
        let body = spawn_figure(commands, spot, 0.0, DOG_MODEL, DOG_SCALE, "Dog".to_string());
        commands.set(body, Heading(0.0));
        commands.set(body, Stride(0.0));
        commands.set(body, Health(1.0));
        commands.set(body, Speed(DOG_SPEED));
        commands.set(body, Beat(GUARD_ROUTES.len() + beat));
        commands.set(body, Post(1));
        commands.set(body, Pause(1.0));
        commands.set(body, Lift(DOG_LIFT));
    }
    for _ in 0..BISON_COUNT {
        let spot = random_place_spot(random, FIELD, 0.7);
        place_model(
            commands,
            BISON_MODEL,
            ground(spot) + Vec3::new(0.0, BISON_LIFT, 0.0),
            random.f32() * std::f32::consts::TAU,
            BISON_SCALE,
        );
    }
}
