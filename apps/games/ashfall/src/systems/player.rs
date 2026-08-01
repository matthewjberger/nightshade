use crate::commands::{post_message, reveal_around, stow, unblock_tile};
use crate::components::{Alive, Bolted, Health, Inscription, ItemKind, Lock, NpcIndex, Tile};
use crate::data::ask::Ask;
use crate::data::rules::*;
use crate::data::theme::{self, PERSON_REACH};
use crate::queries::{carrying, examine_text, is_room, item_name, person, tile_of};
use crate::resources::{AshfallResources, Impact, RunState, Screen, Senses, Weapon, WorldPhase};
use crate::systems::husks;
use crate::tags::{Eyes, Flashlight, Player};
use nightshade::prelude::*;
use nightshade_flashlight::commands::switch;
use nightshade_flashlight::{Battery, Lit};
use nightshade_input::Actions;
use nightshade_physics::contract::components::CharacterControllerComponent;
use nightshade_physics::queries::physics_world_cast_ray;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_senses::commands::{hide, make_a_noise};
use nightshade_senses::{Exposure, Noises, SensesSettings};
use nightshade_ui_flash::Flash;
use nightshade_ui_flash::commands::flash;

type NearDoor = (Entity, Option<ItemKind>, (i32, i32), bool, f32);

pub fn update(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    physics: Res<PhysicsWorld>,
    mut resources: ResMut<AshfallResources>,
    mut senses: ResMut<Senses>,
    mut noises: ResMut<Noises>,
    mut screen: ResMut<Flash>,
    senses_settings: Res<SensesSettings>,
    mut commands: Commands,
    mut bodies: Query<
        (
            &LocalTransform,
            &mut CharacterControllerComponent,
            &mut Exposure,
        ),
        WithTag<Player>,
    >,
    mut eyes: Query<(&GlobalTransform, &LocalTransform), WithTag<Eyes>>,
    mut world: WorldQueries<'_, '_>,
    mut torches: Query<(&mut Battery, &mut Lit), WithTag<Flashlight>>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Playing {
        return;
    }
    let delta = time.delta_time;
    resources.run.clock += delta;
    resources.player.attack_cooldown = (resources.player.attack_cooldown - delta).max(0.0);

    let Some(body) = resources.player.body else {
        return;
    };
    let mut stance = None;
    bodies.for_entity(body, |(transform, controller, exposure)| {
        controller.max_speed = if controller.is_crouching {
            PLAYER_SPEED * CROUCH_SPEED_SCALE
        } else {
            PLAYER_SPEED
        };
        stance = Some((
            transform.translation,
            controller.is_crouching,
            controller.is_sprinting,
            controller.velocity.xz().magnitude(),
        ));
        hide(
            exposure,
            &senses_settings,
            controller.is_crouching,
            controller.is_sprinting,
        );
    });
    let Some((position, crouching, sprinting, speed)) = stance else {
        return;
    };
    resources.player.crouching = crouching;
    resources.player.moving = speed > 0.4;
    let feet = position - Vec3::new(0.0, 1.0, 0.0);
    resources.player.at = position;
    resources.player.upstairs = feet.y > UPPER_FLOOR_LEVEL;
    let standing_on = tile_of(&feet);
    let inside = is_room(standing_on.0, standing_on.1);
    resources.run.here = standing_on;
    let upstairs = resources.player.upstairs;
    reveal_around(&mut resources.run, standing_on, upstairs);

    let mut view = None;
    if let Some(eye_entity) = resources.player.eyes {
        eyes.for_entity(eye_entity, |(global, local)| {
            view = Some((global.translation(), global.forward_vector()));
            resources.player.head = local.rotation;
        });
    }
    let (eye, forward) = view.unwrap_or((
        position + Vec3::new(0.0, EYE_HEIGHT - 1.0, 0.0),
        Vec3::new(0.0, 0.0, -1.0),
    ));

    step_noise(resources, &mut noises, feet, delta, sprinting, crouching);

    if actions.just_pressed(Ask::Torch) {
        let mut flat = false;
        torches.for_each_mut(|_entity, (battery, lit)| {
            if battery.0 <= 0.0 {
                flat = true;
                return;
            }
            switch(lit, battery);
        });
        if flat {
            post_message(
                &mut resources.run,
                "The cell is flat. The switch does nothing.",
            );
        }
    }
    if actions.just_pressed(Ask::PickPipe) && resources.player.weapon != Weapon::Pipe {
        resources.player.weapon = Weapon::Pipe;
        post_message(
            &mut resources.run,
            "A length of steel pipe. It has been used before.",
        );
    }
    if actions.just_pressed(Ask::PickHandgun) && resources.player.weapon != Weapon::Pistol {
        resources.player.weapon = Weapon::Pistol;
        post_message(&mut resources.run, "The handgun. Count the shots.");
    }
    if actions.just_pressed(Ask::OpenPockets) {
        resources.pending_screen = Some(Screen::Inventory);
    }
    if actions.just_pressed(Ask::OpenMap) {
        if carrying(&resources.satchel, ItemKind::TownMap) {
            resources.pending_screen = Some(Screen::TownMap);
        } else {
            post_message(
                &mut resources.run,
                "You have nothing to read the streets off.",
            );
        }
    }
    if actions.just_pressed(Ask::Strike)
        && resources.player.attack_cooldown <= 0.0
        && resources.player.holding.is_none()
    {
        match resources.player.weapon {
            Weapon::Pipe => {
                resources.player.attack_cooldown = PIPE_COOLDOWN;
                swing_pipe(
                    &mut resources.run,
                    &mut senses,
                    &mut noises,
                    &mut world.husks,
                    &mut world.people,
                    feet,
                    eye,
                    forward,
                );
            }
            Weapon::Pistol => {
                resources.player.attack_cooldown = PISTOL_COOLDOWN;
                if resources.player.ammo == 0 {
                    post_message(&mut resources.run, "Click. Empty.");
                } else {
                    resources.player.ammo -= 1;
                    flash(&mut screen, theme::MUZZLE_FLASH, MUZZLE_FADE);
                    fire_pistol(
                        &mut resources.run,
                        &mut senses,
                        &mut noises,
                        &physics,
                        &mut world.husks,
                        &mut world.people,
                        body,
                        eye,
                        forward,
                    );
                }
            }
        }
    }

    let mut nearest_pickup: Option<(Entity, ItemKind, f32)> = None;
    world.pickups.for_each_mut(|entity, (kind, transform)| {
        let distance = (transform.translation - feet).magnitude();
        if distance <= PICKUP_REACH && nearest_pickup.is_none_or(|(_, _, best)| distance < best) {
            nearest_pickup = Some((entity, *kind, distance));
        }
    });
    let mut nearest_door: Option<NearDoor> = None;
    world
        .doors
        .for_each_mut(|entity, (lock, tile, bolted, transform)| {
            let distance = (transform.translation - feet).magnitude();
            if distance <= DOOR_REACH
                && nearest_door.is_none_or(|(_, _, _, _, best)| distance < best)
            {
                nearest_door = Some((entity, lock.0, tile.0, bolted.0, distance));
            }
        });
    let mut nearest_person: Option<(usize, f32)> = None;
    world
        .people
        .for_each_mut(|_entity, (index, _, alive, transform)| {
            if !alive.0 {
                return;
            }
            let distance = (transform.translation - position).magnitude();
            if distance <= PERSON_REACH && nearest_person.is_none_or(|(_, best)| distance < best) {
                nearest_person = Some((index.0, distance));
            }
        });
    let mut nearest_look: Option<(usize, f32)> = None;
    world
        .inscriptions
        .for_each_mut(|_entity, (look, transform)| {
            let distance = (transform.translation - feet).magnitude();
            if distance <= EXAMINE_REACH && nearest_look.is_none_or(|(_, best)| distance < best) {
                nearest_look = Some((look.index, distance));
            }
        });

    resources.run.prompt = if resources.player.holding.is_some() {
        "Throw it".to_string()
    } else {
        match (nearest_pickup, nearest_person, nearest_door, nearest_look) {
            (Some((_, kind, _)), ..) => format!("Take the {}", item_name(kind)),
            (None, Some((index, _)), ..) => format!("Speak to {}", person(index).name),
            (None, None, Some((_, _, _, true, _)), _) if !inside => {
                "The door is bolted from the other side".to_string()
            }
            (None, None, Some((_, _, _, true, _)), _) => "Draw the bolts back".to_string(),
            (None, None, Some((_, Some(_), _, _, _)), _) => "Try the door".to_string(),
            (None, None, Some((_, None, _, _, _)), _) => "Open the door".to_string(),
            (None, None, None, Some(_)) => "Look closer".to_string(),
            (None, None, None, None) => String::new(),
        }
    };

    if actions.just_pressed(Ask::Take) {
        if let Some((entity, kind, _)) = nearest_pickup {
            if take_item(resources, kind) {
                commands.despawn_recursive(entity);
            }
        } else if let Some((index, _)) = nearest_person {
            resources.run.talking = Some(index);
            resources.run.said = 0;
            resources.pending_screen = Some(Screen::Talking);
        } else if let Some((entity, lock, tile, bolted, _)) = nearest_door {
            if bolted && !inside {
                post_message(
                    &mut resources.run,
                    "Bolted, and the bolts are on the other side of it.",
                );
            } else {
                open_door(resources, &mut commands, entity, lock, tile, upstairs);
            }
        } else if let Some((index, _)) = nearest_look {
            post_message(&mut resources.run, examine_text(index));
        }
    }

    if resources.run.phase == WorldPhase::Otherworld && standing_on == resources.run.exit_tile {
        resources.pending_screen = Some(Screen::Escaped);
    }
    if resources.player.health <= 0.0 {
        resources.pending_screen = Some(Screen::Dead);
    }
}

pub type Pickups<'world, 'item> = Query<'world, (&'item ItemKind, &'item LocalTransform)>;
pub type Doors<'world, 'item> = Query<
    'world,
    (
        &'item Lock,
        &'item Tile,
        &'item Bolted,
        &'item LocalTransform,
    ),
>;
pub type Inscriptions<'world, 'item> = Query<'world, (&'item Inscription, &'item LocalTransform)>;
pub type People<'world, 'item> = Query<
    'world,
    (
        &'item NpcIndex,
        &'item mut Health,
        &'item mut Alive,
        &'item LocalTransform,
    ),
>;

nightshade::ecs::query_bundle! {
    pub struct WorldQueries<'world, 'item> {
        pub pickups: Pickups<'world, 'item>,
        pub doors: Doors<'world, 'item>,
        pub husks: husks::Wounded<'world, 'item>,
        pub inscriptions: Inscriptions<'world, 'item>,
        pub people: People<'world, 'item>,
    }
}

fn step_noise(
    resources: &mut AshfallResources,
    noises: &mut Noises,
    feet: Vec3,
    delta: f32,
    sprinting: bool,
    crouching: bool,
) {
    if !resources.player.moving || crouching {
        resources.player.footfall = 0.0;
        return;
    }
    resources.player.footfall += delta;
    let interval = if sprinting {
        FOOTFALL_INTERVAL * 0.6
    } else {
        FOOTFALL_INTERVAL
    };
    if resources.player.footfall < interval {
        return;
    }
    resources.player.footfall = 0.0;
    let loudness = if sprinting {
        NOISE_FOOTFALL
    } else {
        NOISE_FOOTFALL * 0.45
    };
    make_a_noise(noises, feet, loudness);
}

fn open_door(
    resources: &mut AshfallResources,
    commands: &mut Commands,
    entity: Entity,
    lock: Option<ItemKind>,
    tile: (i32, i32),
    upstairs: bool,
) {
    let has_key = match lock {
        None => true,
        Some(wanted) => carrying(&resources.satchel, wanted),
    };
    if has_key {
        unblock_tile(&mut resources.run, tile, upstairs);
        commands.despawn_recursive(entity);
        post_message(
            &mut resources.run,
            if lock.is_some() {
                "The lock turns. The door swings in on its own."
            } else {
                "The door gives with a wet creak."
            },
        );
    } else {
        let name = lock.map(item_name).unwrap_or_default();
        post_message(
            &mut resources.run,
            format!("Locked. A {name} shape is cut into the plate."),
        );
    }
}

fn take_item(resources: &mut AshfallResources, kind: ItemKind) -> bool {
    if let ItemKind::Note(index) = kind {
        if let Some(found) = resources.run.found_notes.get_mut(index) {
            *found = true;
        }
        resources.run.reading = Some(index);
        resources.pending_screen = Some(Screen::Reading);
        return true;
    }
    if !stow(&mut resources.satchel, kind) {
        post_message(
            &mut resources.run,
            "Your hands and your pockets are full. Something has to stay here.",
        );
        return false;
    }
    if kind == ItemKind::BoneKey {
        resources.run.phase = WorldPhase::Otherworld;
    }
    post_message(
        &mut resources.run,
        format!("You take the {}.", item_name(kind)),
    );
    true
}

fn swing_pipe(
    run: &mut RunState,
    senses: &mut Senses,
    noises: &mut Noises,
    husks: &mut husks::Wounded<'_, '_>,
    people: &mut People<'_, '_>,
    feet: Vec3,
    eye: Vec3,
    forward: Vec3,
) {
    let flat_forward = nalgebra_glm::normalize(&Vec3::new(forward.x, 0.0, forward.z));
    make_a_noise(noises, feet, NOISE_SWING);
    senses.impacts.push(Impact {
        at: eye + forward * PIPE_REACH,
        force: SWING_IMPACT,
    });

    let mut hits = 0;
    let mut dropped = 0;
    let mut finished = 0;
    husks.for_each_mut(|_entity, (state, health, timer, moving, transform)| {
        let to_husk = Vec3::new(
            transform.translation.x - feet.x,
            0.0,
            transform.translation.z - feet.z,
        );
        let distance = to_husk.magnitude();
        if distance > PIPE_REACH || distance < 0.01 {
            return;
        }
        if nalgebra_glm::dot(&(to_husk / distance), &flat_forward) < 0.45 {
            return;
        }
        let was_down = husks::is_down(state);
        let killed = husks::wound(state, health, timer, moving, PIPE_DAMAGE);
        if killed && was_down {
            finished += 1;
        } else if husks::is_down(state) && !was_down {
            dropped += 1;
        } else {
            hits += 1;
        }
    });
    let mut people_hit = 0;
    let mut people_down = 0;
    people.for_each_mut(|_entity, (_, health, alive, transform)| {
        if !alive.0 {
            return;
        }
        let toward = Vec3::new(
            transform.translation.x - feet.x,
            0.0,
            transform.translation.z - feet.z,
        );
        let distance = toward.magnitude();
        if distance > PIPE_REACH || distance < 0.01 {
            return;
        }
        if nalgebra_glm::dot(&(toward / distance), &flat_forward) < 0.45 {
            return;
        }
        health.0 -= PIPE_DAMAGE;
        people_hit += 1;
        if health.0 <= 0.0 {
            alive.0 = false;
            people_down += 1;
            make_a_noise(noises, transform.translation, PERSON_DEATH_NOISE);
            senses.stains.push(transform.translation);
        }
    });
    if people_down > 0 {
        run.message = "The last swing lands and there is no one left to shout.".to_string();
        run.message_until = run.clock + MESSAGE_SECONDS;
        return;
    }
    if people_hit > 0 {
        run.message = "The pipe lands on somebody who is still a person.".to_string();
        run.message_until = run.clock + MESSAGE_SECONDS;
        return;
    }
    run.kills += finished;
    if finished > 0 {
        run.message = "You bring the pipe down until it stops moving for good.".to_string();
    } else if dropped > 0 {
        run.message = "It folds over and lies still. It is still breathing.".to_string();
    } else if hits > 0 {
        run.message = "The pipe connects. Something cracks.".to_string();
    } else {
        return;
    }
    run.message_until = run.clock + MESSAGE_SECONDS;
}

fn fire_pistol(
    run: &mut RunState,
    senses: &mut Senses,
    noises: &mut Noises,
    physics: &PhysicsWorld,
    husks: &mut husks::Wounded<'_, '_>,
    people: &mut People<'_, '_>,
    body: Entity,
    eye: Vec3,
    forward: Vec3,
) {
    let forward = nalgebra_glm::normalize(&forward);
    make_a_noise(noises, eye, NOISE_SHOT);
    let wall = physics_world_cast_ray(physics, eye, forward, PISTOL_RANGE, Some(body));
    senses.impacts.push(Impact {
        at: wall
            .as_ref()
            .map(|hit| hit.point)
            .unwrap_or(eye + forward * PISTOL_RANGE),
        force: SHOT_IMPACT,
    });

    let mut best: Option<(Entity, f32)> = None;
    husks.for_each_mut(|entity, (state, _, _, _, transform)| {
        if husks::is_down(state) {
            return;
        }
        let chest = transform.translation + Vec3::new(0.0, 0.2, 0.0);
        let to_husk = chest - eye;
        let along = nalgebra_glm::dot(&to_husk, &forward);
        if along <= 0.0 || along > PISTOL_RANGE {
            return;
        }
        let lateral = (to_husk - forward * along).magnitude();
        if lateral > PISTOL_SPREAD {
            return;
        }
        if wall.as_ref().is_some_and(|hit| hit.distance < along - 0.6) {
            return;
        }
        if best.is_none_or(|(_, nearest)| along < nearest) {
            best = Some((entity, along));
        }
    });
    let mut hit_person: Option<(Entity, f32)> = None;
    people.for_each_mut(|entity, (_, _, alive, transform)| {
        if !alive.0 {
            return;
        }
        let chest = transform.translation + Vec3::new(0.0, 0.2, 0.0);
        let to_person = chest - eye;
        let along = nalgebra_glm::dot(&to_person, &forward);
        if along <= 0.0 || along > PISTOL_RANGE {
            return;
        }
        if (to_person - forward * along).magnitude() > PISTOL_SPREAD {
            return;
        }
        if wall.as_ref().is_some_and(|hit| hit.distance < along - 0.6) {
            return;
        }
        if hit_person.is_none_or(|(_, nearest)| along < nearest) {
            hit_person = Some((entity, along));
        }
    });
    if let Some((entity, along)) = hit_person
        && best.is_none_or(|(_, husk_along)| along < husk_along)
    {
        let mut dead = None;
        people.for_entity(entity, |(_, health, alive, transform)| {
            health.0 -= PISTOL_DAMAGE;
            if health.0 <= 0.0 && alive.0 {
                alive.0 = false;
                dead = Some(transform.translation);
            }
        });
        if let Some(at) = dead {
            make_a_noise(noises, at, PERSON_DEATH_NOISE);
            senses.stains.push(at);
            run.message = "The shot goes home. They fold up where they stood.".to_string();
        } else {
            run.message = "The shot goes into somebody who was still talking.".to_string();
        }
        run.message_until = run.clock + MESSAGE_SECONDS;
        return;
    }
    let Some((target, _)) = best else {
        run.message = "The shot goes into the fog.".to_string();
        run.message_until = run.clock + MESSAGE_SECONDS;
        return;
    };
    let mut down = false;
    husks.for_entity(target, |(state, health, timer, moving, _)| {
        husks::wound(state, health, timer, moving, PISTOL_DAMAGE);
        down = husks::is_down(state);
    });
    if down {
        run.message = "It goes down. It does not stay down for long.".to_string();
    } else {
        run.message = "It staggers and keeps coming.".to_string();
    }
    run.message_until = run.clock + MESSAGE_SECONDS;
}
