use crate::commands::models::{place_model, plant_model};
use crate::commands::random::{random_name, random_place_spot, wild_spot};
use crate::commands::spells::prop;
use crate::commands::{STONE, mob_material, pickup_material};
use crate::components::{
    Aggro, Approach, Beat, Bounty, Chill, Class, Cooldowns, Dying, Experience, Fade, Flinch, Giver,
    GiverMark, Goal, Guild, Heading, Health, Home, Hunt, Job, Kills, Leash, Level, Lift, Limb,
    Mana, MaxHealth, MaxMana, Pause, Phase, Pickup, Pivot, Plate, PlateLevel, Post, Rank, Reach,
    Rebirth, Recovery, Regrow, Respawn, Rest, Rig, Rising, Speed, Sprint, Strength, Stride, Strike,
    Swing, Target, Task, Voice, Wander, Worth,
};
use crate::data::{
    ABILITY_COUNT, BISON_MODEL, CALLINGS, CLASS_MODELS, CLASSES, DOG_MODEL, DOG_ROUTES, ELITE,
    FIELD, GIVERS, GUARD_MODEL, GUARD_NAME, GUARD_POSTS, GUARD_ROUTES, GUILDS, HUNTS, LANDMARKS,
    PICKUPS, RING_MODEL, SPAWNS, TOWN, TOWNSFOLK,
};
use crate::queries::{
    guessed_ground, heading_toward, hunt_for_level, level_threshold, person_plate, sim_max_health,
    sim_strength, spot_of,
};
use crate::resources::{Generation, Hero, Places};
use crate::tags::Figure;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera_controller::prelude::*;
use nightshade_random::prelude::*;
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

fn root(commands: &mut Commands, position: Vec3, heading: f32, name: String) -> Entity {
    commands.spawn((
        Name(name),
        LocalTransform {
            translation: position,
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
    position: Vec3,
    heading: f32,
    model: &str,
    scale: f32,
    name: String,
) -> Entity {
    let body = place_model(
        commands,
        model,
        position,
        heading,
        scale * theme::FIGURE_SCALE,
    );
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
    material: &str,
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
    position: Vec3,
    heading: f32,
    size: f32,
    material: &str,
    snout: &str,
    name: String,
) -> Entity {
    let size = size * theme::FIGURE_SCALE;
    let body = root(commands, position, heading, name);
    let mut part = |position: Vec3, scale: Vec3, material: &str| {
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

fn walker(commands: &mut Commands, body: Entity, heading: f32) {
    commands.set(body, Heading(heading));
    commands.set(body, Stride(0.0));
}

pub fn spawn_mobs(
    random: &mut Random,
    generation: &Generation,
    places: &Places,
    text_state: &mut TextState,
    commands: &mut Commands,
) {
    for (index, spawn) in SPAWNS.iter().enumerate() {
        for _ in 0..spawn.count {
            let spot = if spawn.rank >= ELITE {
                wild_spot(random, generation, |random| {
                    around(
                        random,
                        spot_of(places, spawn.place, Vec2::zeros()),
                        2.0,
                        5.0,
                    )
                })
            } else {
                wild_spot(random, generation, |random| {
                    random_place_spot(random, places, spawn.place, 0.85)
                })
            };
            let position = guessed_ground(generation, spot);
            let heading = random.f32() * std::f32::consts::TAU;
            let body = match spawn.model {
                Some(model) => spawn_figure(
                    commands,
                    position,
                    heading,
                    model,
                    spawn.model_scale,
                    spawn.name.to_string(),
                ),
                None => spawn_quadruped(
                    commands,
                    position,
                    heading,
                    spawn.scale,
                    &mob_material(index),
                    STONE,
                    spawn.name.to_string(),
                ),
            };
            let lift = if spawn.biped {
                theme::PLATE_HEIGHT * spawn.scale
            } else {
                1.5 * spawn.scale * theme::FIGURE_SCALE
            };
            let elite = spawn.rank >= ELITE;
            let plate = spawn_plate(
                text_state,
                commands,
                position + Vec3::new(0.0, lift, 0.0),
                &crate::queries::mob_plate(spawn.name, spawn.level, spawn.rank),
                if elite {
                    theme::ELITE_PLATE_COLOR
                } else {
                    theme::MONSTER_PLATE_COLOR
                },
                if elite {
                    theme::PLATE_FONT_SIZE * 1.3
                } else {
                    theme::PLATE_FONT_SIZE * 0.85
                },
            );
            walker(commands, body, heading);
            commands.set(body, Level(spawn.level));
            commands.set(body, Speed(spawn.speed));
            commands.set(body, Health(spawn.health));
            commands.set(body, MaxHealth(spawn.health));
            commands.set(body, Home(position));
            commands.set(body, Goal(position));
            commands.set(body, Wander(between(random, 0.0, 4.0)));
            commands.set(body, Target(None));
            commands.set(body, Swing(between(random, 0.0, theme::ATTACK_PERIOD)));
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

pub fn spawn_people(
    random: &mut Random,
    generation: &Generation,
    places: &Places,
    text_state: &mut TextState,
    commands: &mut Commands,
) {
    let board = spot_of(places, TOWN, GIVERS[0].offset);
    for _ in 0..theme::POPULATION {
        let level = random.u32(theme::SIM_LEVEL_MIN..theme::SIM_LEVEL_MAX);
        let class = random.usize(0..CLASSES.len());
        let guild = GUILDS[random.usize(0..GUILDS.len())].to_string();
        let name = random_name(random);
        let heading = random.f32() * std::f32::consts::TAU;
        let hunt = hunt_for_level(level);
        let camp = HUNTS[hunt].place;
        let giver = &GIVERS[HUNTS[hunt].giver];
        let giver_spot = spot_of(places, giver.place, giver.offset);

        let roll = random.f32();
        let (job, spot, goal, kills) = if roll < 0.22 {
            let spot = around(random, board, 2.0, 5.0);
            (Job::Resting, spot, guessed_ground(generation, spot), 0)
        } else if roll < 0.5 {
            let spot = around(random, board, 3.0, 9.0);
            (
                Job::ToCamp,
                spot,
                guessed_ground(generation, random_place_spot(random, places, camp, 0.8)),
                0,
            )
        } else if roll < 0.82 {
            let spot = wild_spot(random, generation, |random| {
                random_place_spot(random, places, camp, 0.9)
            });
            (
                Job::Hunting,
                spot,
                guessed_ground(generation, spot),
                random.u32(0..theme::SIM_QUEST_KILLS),
            )
        } else {
            let spot = wild_spot(random, generation, |random| {
                random_place_spot(random, places, camp, 1.0)
            });
            (
                Job::ToGiver,
                spot,
                guessed_ground(generation, around(random, giver_spot, 1.5, 3.5)),
                theme::SIM_QUEST_KILLS,
            )
        };

        let (male, female) = CLASS_MODELS[class];
        let model = if random.bool() { male } else { female };
        let position = guessed_ground(generation, spot);
        let body = spawn_figure(
            commands,
            position,
            heading,
            model,
            theme::FOLK_SCALE,
            name.clone(),
        );
        let plate = spawn_plate(
            text_state,
            commands,
            position + Vec3::new(0.0, theme::PLATE_HEIGHT, 0.0),
            &person_plate(&name, &guild, level),
            CLASSES[class].plate,
            theme::PLATE_FONT_SIZE,
        );

        let max_health = sim_max_health(level);
        walker(commands, body, heading);
        commands.set(
            body,
            Speed(between(random, theme::SIM_SPEED_MIN, theme::SIM_SPEED_MAX)),
        );
        commands.set(body, Health(max_health));
        commands.set(body, MaxHealth(max_health));
        commands.set(body, Task(job));
        commands.set(body, Rest(between(random, 1.0, theme::REST_MAX_SECONDS)));
        commands.set(body, Hunt(hunt));
        commands.set(body, Goal(goal));
        commands.set(body, Target(None));
        commands.set(body, Swing(between(random, 0.0, theme::ATTACK_PERIOD)));
        commands.set(body, Strength(sim_strength(level)));
        commands.set(body, Chill(0.0));
        commands.set(body, Kills(kills));
        commands.set(body, Level(level));
        commands.set(body, Experience(random.u32(0..level_threshold(level))));
        commands.set(body, Plate(plate));
        commands.set(body, Lift(theme::PLATE_HEIGHT));
        commands.set(body, PlateLevel(level));
        commands.set(body, Guild(guild));
        commands.set(body, Class(class));
        commands.set(body, Voice(random.usize(0..theme::VOICE_COUNT)));
    }
}

pub fn spawn_givers(
    generation: &Generation,
    places: &Places,
    text_state: &mut TextState,
    commands: &mut Commands,
) {
    let board = guessed_ground(generation, spot_of(places, TOWN, GIVERS[0].offset));
    for (index, giver) in GIVERS.iter().enumerate() {
        let spot = spot_of(places, giver.place, giver.offset);
        let position = guessed_ground(generation, spot);
        let heading = if giver.board {
            0.0
        } else {
            heading_toward(position, board)
        };
        let body = spawn_figure(
            commands,
            position,
            heading,
            giver.model,
            giver.model_scale,
            giver.name.to_string(),
        );
        let lift = if giver.board {
            2.7
        } else {
            theme::PLATE_HEIGHT + 0.5
        };
        let plate = spawn_plate(
            text_state,
            commands,
            position + Vec3::new(0.0, lift, 0.0),
            &crate::queries::giver_plate(index, 0),
            theme::GIVER_PLATE_COLOR,
            theme::PLATE_FONT_SIZE,
        );
        walker(commands, body, heading);
        commands.set(body, Health(1.0));
        commands.set(body, Giver(index));
        commands.set(body, GiverMark(0));
        commands.set(body, Plate(plate));
        commands.set(body, Lift(lift));
    }
}

pub fn spawn_pickups(
    random: &mut Random,
    generation: &Generation,
    places: &Places,
    commands: &mut Commands,
) {
    for (kind, pickup) in PICKUPS.iter().enumerate() {
        for _ in 0..pickup.count {
            let spot = wild_spot(random, generation, |random| {
                random_place_spot(random, places, pickup.place, 0.95)
            });
            let node = plant_model(
                commands,
                generation,
                pickup.model,
                spot,
                random.f32() * std::f32::consts::TAU,
                pickup.model_scale,
                0.0,
            );
            let glow = prop(
                commands,
                "Sphere",
                Vec3::new(0.0, 1.9, 0.0),
                Vec3::new(0.18, 0.18, 0.18),
                &pickup_material(kind),
            );
            commands.set(glow, ChildOf(node));
            commands.set(node, Pickup(kind));
            commands.set(node, Regrow(0.0));
            commands.set(node, Home(guessed_ground(generation, spot)));
        }
    }
}

pub fn spawn_ring(commands: &mut Commands) -> Entity {
    place_model(
        commands,
        RING_MODEL,
        Vec3::new(0.0, -theme::SINK_DEPTH, 0.0),
        0.0,
        3.0,
    )
}

pub fn spawn_floating_pool(text_state: &mut TextState, commands: &mut Commands) -> Vec<Entity> {
    (0..theme::FLOATING_COUNT)
        .map(|_| {
            let entity = spawn_plate(
                text_state,
                commands,
                Vec3::new(0.0, -theme::SINK_DEPTH, 0.0),
                "",
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                theme::PLATE_FONT_SIZE * 1.2,
            );
            commands.set(entity, Fade(0.0));
            entity
        })
        .collect()
}

pub struct HeroStart {
    pub position: Vec3,
    pub yaw: f32,
    pub level: u32,
    pub experience: u32,
    pub kills: u32,
    pub health: Option<f32>,
    pub mana: Option<f32>,
}

pub fn spawn_player(
    hero: &Hero,
    start: HeroStart,
    text_state: &mut TextState,
    commands: &mut Commands,
) -> (Entity, Entity) {
    let class = &CALLINGS[hero.class];
    let name = hero.name.clone();
    let steps = start.level.saturating_sub(1) as f32;
    let max_health = class.health + theme::HEALTH_PER_LEVEL * steps;
    let max_mana = class.mana + theme::MANA_PER_LEVEL * steps;
    let strength = class.strength + theme::STRENGTH_PER_LEVEL * steps;

    let body = spawn_figure(
        commands,
        start.position,
        start.yaw,
        class.model,
        theme::FOLK_SCALE,
        name.clone(),
    );
    let plate = spawn_plate(
        text_state,
        commands,
        start.position + Vec3::new(0.0, theme::PLATE_HEIGHT, 0.0),
        &crate::queries::player_plate(&name, start.level),
        theme::PLAYER_PLATE_COLOR,
        theme::PLATE_FONT_SIZE,
    );

    walker(commands, body, start.yaw);
    commands.set(body, Speed(class.speed));
    commands.set(
        body,
        Health(start.health.unwrap_or(max_health).min(max_health)),
    );
    commands.set(body, MaxHealth(max_health));
    commands.set(body, Approach(None));
    commands.set(body, Mana(start.mana.unwrap_or(max_mana).min(max_mana)));
    commands.set(body, MaxMana(max_mana));
    commands.set(body, Cooldowns([0.0; ABILITY_COUNT]));
    commands.set(body, Recovery(0.0));
    commands.set(body, Sprint(0.0));
    commands.set(body, Chill(0.0));
    commands.set(body, Target(None));
    commands.set(body, Swing(0.0));
    commands.set(body, Strength(strength));
    commands.set(body, Level(start.level));
    commands.set(body, Experience(start.experience));
    commands.set(body, Kills(start.kills));
    commands.set(body, Plate(plate));
    commands.set(body, Lift(theme::PLATE_HEIGHT));
    commands.set(body, PlateLevel(start.level));

    let camera = spawn_third_person_camera(
        commands,
        body,
        theme::CAMERA_DISTANCE,
        start.yaw,
        theme::CAMERA_PITCH,
        "Player Camera".to_string(),
    );
    if let Some(third_person) = commands.staged_mut::<ThirdPersonCamera>(camera) {
        third_person.collision_enabled = false;
        third_person.distance_min = 0.0;
        third_person.distance_max = theme::CAMERA_DISTANCE_MAX;
        third_person.height_offset = theme::CAMERA_HEAD;
    }
    commands.restage::<ThirdPersonCamera>(camera);

    (body, camera)
}

pub fn spawn_stage_figure(commands: &mut Commands, class: usize, position: Vec3) -> Entity {
    let data = &CALLINGS[class];
    let body = spawn_figure(
        commands,
        position,
        std::f32::consts::PI,
        data.model,
        theme::FOLK_SCALE,
        data.name.to_string(),
    );
    walker(commands, body, std::f32::consts::PI);
    commands.set(body, Health(1.0));
    body
}

pub fn spawn_stage_camera(commands: &mut Commands, figure: Entity) -> Entity {
    let camera = spawn_third_person_camera(
        commands,
        figure,
        theme::STAGE_DISTANCE,
        0.0,
        theme::STAGE_PITCH,
        "Stage Camera".to_string(),
    );
    if let Some(third_person) = commands.staged_mut::<ThirdPersonCamera>(camera) {
        third_person.collision_enabled = false;
        third_person.height_offset = theme::CAMERA_HEAD * 0.7;
    }
    commands.restage::<ThirdPersonCamera>(camera);
    camera
}

fn plain_plate(
    text_state: &mut TextState,
    commands: &mut Commands,
    body: Entity,
    position: Vec3,
    lift: f32,
    label: &str,
    color: Vec4,
) {
    let plate = spawn_plate(
        text_state,
        commands,
        position + Vec3::new(0.0, lift, 0.0),
        label,
        color,
        theme::PLATE_FONT_SIZE * 0.9,
    );
    commands.set(body, Plate(plate));
    commands.set(body, Lift(lift));
}

fn spawn_guard(
    generation: &Generation,
    text_state: &mut TextState,
    commands: &mut Commands,
    spot: Vec2,
    heading: f32,
) -> Entity {
    let position = guessed_ground(generation, spot);
    let body = spawn_figure(
        commands,
        position,
        heading,
        GUARD_MODEL,
        theme::GUARD_SCALE,
        GUARD_NAME.to_string(),
    );
    walker(commands, body, heading);
    commands.set(body, Health(1.0));
    commands.set(body, Speed(theme::GUARD_SPEED));
    plain_plate(
        text_state,
        commands,
        body,
        position,
        theme::PLATE_HEIGHT + 0.3,
        &format!("<Town Watch>\n{GUARD_NAME}"),
        theme::GUARD_PLATE_COLOR,
    );
    body
}

pub fn spawn_guards(
    generation: &Generation,
    places: &Places,
    text_state: &mut TextState,
    commands: &mut Commands,
) {
    for (offset, heading) in GUARD_POSTS {
        spawn_guard(
            generation,
            text_state,
            commands,
            spot_of(places, TOWN, offset),
            heading,
        );
    }
    for (beat, route) in GUARD_ROUTES.iter().enumerate() {
        for start in [0, route.len() / 2] {
            let spot = spot_of(places, TOWN, route[start]);
            let body = spawn_guard(generation, text_state, commands, spot, 0.0);
            commands.set(body, Beat(beat));
            commands.set(body, Post((start + 1) % route.len()));
            commands.set(body, Pause(0.5 + start as f32));
        }
    }
}

pub fn spawn_townsfolk(
    generation: &Generation,
    places: &Places,
    text_state: &mut TextState,
    commands: &mut Commands,
) {
    for person in &TOWNSFOLK {
        let position = guessed_ground(generation, spot_of(places, person.place, person.offset));
        let body = spawn_figure(
            commands,
            position,
            person.heading,
            person.model,
            person.scale,
            person.name.to_string(),
        );
        walker(commands, body, person.heading);
        commands.set(body, Health(1.0));
        plain_plate(
            text_state,
            commands,
            body,
            position,
            theme::PLATE_HEIGHT + 0.3,
            &format!("<{}>\n{}", person.title, person.name),
            theme::GIVER_PLATE_COLOR,
        );
    }
}

pub fn spawn_animals(
    random: &mut Random,
    generation: &Generation,
    places: &Places,
    commands: &mut Commands,
) {
    for (beat, route) in DOG_ROUTES.iter().enumerate() {
        let position = guessed_ground(generation, spot_of(places, TOWN, route[0]));
        let body = spawn_figure(
            commands,
            position,
            0.0,
            DOG_MODEL,
            theme::DOG_SCALE,
            "Dog".to_string(),
        );
        walker(commands, body, 0.0);
        commands.set(body, Health(1.0));
        commands.set(body, Speed(theme::DOG_SPEED));
        commands.set(body, Beat(GUARD_ROUTES.len() + beat));
        commands.set(body, Post(1));
        commands.set(body, Pause(1.0));
        commands.set(body, Lift(theme::DOG_LIFT * theme::FIGURE_SCALE));
    }
    for _ in 0..theme::BISON_COUNT {
        let spot = wild_spot(random, generation, |random| {
            random_place_spot(random, places, FIELD, 0.7)
        });
        plant_model(
            commands,
            generation,
            BISON_MODEL,
            spot,
            random.f32() * std::f32::consts::TAU,
            theme::BISON_SCALE * theme::FIGURE_SCALE,
            theme::BISON_LIFT * theme::FIGURE_SCALE,
        );
    }
}

pub fn spawn_landmarks(generation: &Generation, places: &Places, commands: &mut Commands) {
    for landmark in &LANDMARKS {
        plant_model(
            commands,
            generation,
            landmark.model,
            spot_of(places, landmark.place, landmark.offset),
            landmark.yaw,
            landmark.scale,
            0.0,
        );
    }
}
