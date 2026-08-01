use crate::commands::{mark_the_giver, stand_a_body, stand_a_person};
use crate::components::{Haunt, Offer, Persona, Post, Rest, Shop, Speed, Target};
use crate::data::people::Role;
use crate::data::settings::Settings;
use crate::queries::{perch_near, role_at_post, spot_in_the_crowd};
use crate::resources::Crew;
use crate::theme;
use blocker_blocks::queries::{footing, step_holds, walkable};
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::commands::{advance_the_stride, plate_the_body};
use blocker_figure::components::{Gait, Heading, Home, Look, Pace};
use blocker_figure::data::characters::LIVING_LOOKS;
use blocker_figure::data::rig::FIGURE_SCALE;
use blocker_figure::queries::{angle_delta, plate_properties};
use blocker_world::resources::{Bench, Generation};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::Time;
use nightshade_random::Random;
use nightshade_random::commands::between;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::{LocalTransform, Name};
use nightshade_text::prelude::TextState;

pub fn populate(
    world: Res<VoxelWorld>,
    generation: Res<Generation>,
    bench: Res<Bench>,
    config: Res<Config<Settings>>,
    mut crew: ResMut<Crew>,
    mut random: ResMut<Random>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
    present: Query<(&Look,)>,
) {
    if generation.homes.is_empty() {
        return;
    }
    let rest = config.number(Settings::Rest);
    let mut count = 0usize;
    present.for_each(|_, _| count += 1);
    for (village, doors) in generation.homes.iter() {
        if count >= theme::CREW_COUNT {
            break;
        }
        if crew.served.contains(village) {
            continue;
        }
        crew.served.insert(*village);
        let plaza = Vec2::new(village[0] as f32 + 0.5, village[1] as f32 + 0.5);
        let posts = generation.posts.get(village);
        for (index, door) in doors.iter().enumerate() {
            if count >= theme::CREW_COUNT {
                break;
            }
            let Some(ground) = footing(&world, door[0], door[2], door[1]) else {
                continue;
            };
            let home = Vec2::new(door[0] as f32 + 0.5, door[2] as f32 + 0.5);
            let place = Vec3::new(home.x, ground as f32 + 1.0, home.y);
            let (post, role) = match posts {
                Some(posts) if !posts.is_empty() => {
                    let (spot, kind) = posts[index % posts.len()];
                    let role = if index < posts.len() {
                        role_at_post(kind)
                    } else {
                        Role::Folk
                    };
                    (Vec2::new(spot[0] as f32 + 0.5, spot[2] as f32 + 0.5), role)
                }
                _ => (home, Role::Folk),
            };
            let look =
                LIVING_LOOKS[nightshade_random::commands::index(&mut random, LIVING_LOOKS.len())];
            let body = stand_a_body(&mut commands, "", place, look, FIGURE_SCALE, 1.0);
            let spot = spot_in_the_crowd(index, doors.len());
            let haunt = if index % theme::CREW_SQUARE_SHARE == 0 {
                plaza + spot
            } else {
                home + spot * theme::CREW_DOORSTEP
            };
            commands.set(body, Haunt(haunt));
            commands.set(body, Post(post + spot * theme::CREW_POST_SHARE));
            commands.set(body, Rest(between(&mut random, 0.0, rest)));
            let label = if role == Role::Giver {
                let quest = nightshade_random::commands::index(
                    &mut random,
                    blocker_quests::data::quests::WANDERING_QUESTS,
                );
                let giver = blocker_quests::data::quests::QUESTS[quest].giver;
                let giver_name = blocker_quests::data::trade::GIVERS[giver].name;
                let title = blocker_quests::data::trade::GIVERS[giver].title;
                commands.set(body, Name(giver_name.to_string()));
                commands.set(body, Offer(quest));
                mark_the_giver(&mut text_state, &mut commands, body, place);
                format!("<{title}>\n{giver_name}")
            } else if role == Role::Trader {
                let trader = nightshade_random::commands::index(
                    &mut random,
                    blocker_quests::data::trade::TRADERS.len(),
                );
                let name = blocker_quests::data::trade::TRADERS[trader];
                commands.set(body, Name(name.to_string()));
                commands.set(body, Shop(trader));
                format!("<Trader>\n{name}")
            } else {
                let persona = nightshade_random::commands::index(
                    &mut random,
                    crate::data::people::FOLK_VILLAGERS,
                );
                let folk = &crate::data::people::FOLK[persona];
                commands.set(body, Name(folk.name.to_string()));
                commands.set(body, Persona(persona));
                format!("<{}>\n{}", folk.role, folk.name)
            };
            plate_the_body(
                &mut text_state,
                &mut commands,
                body,
                &label,
                place,
                plate_properties(),
            );
            count += 1;
        }
    }
    for (village, doors) in generation.homes.iter() {
        if crew.perched.contains(village) {
            continue;
        }
        let Some(first) = doors.first() else {
            continue;
        };
        if let Some(perch) = perch_near(&world, *village, first[1]) {
            crew.perched.insert(*village);
            let spot = Vec3::new(
                perch[0] as f32 + 0.5,
                perch[1] as f32 + 1.0,
                perch[2] as f32 + 0.5,
            );
            stand_a_person(
                &mut text_state,
                &mut commands,
                spot,
                theme::PERCH_LOOK,
                crate::data::people::PERCH_PERSONA,
                0.0,
                FIGURE_SCALE,
            );
        }
    }
    for seat in bench.seats.iter() {
        if crew.gated.contains(&seat.city) {
            continue;
        }
        if stand_the_gate(&world, &mut text_state, &mut commands, seat.city) {
            crew.gated.insert(seat.city);
        }
    }
    for (village, door) in generation.guides.iter() {
        if crew.guided.contains(village) {
            continue;
        }
        if stand_the_watch(&world, &mut text_state, &mut commands, *door) {
            crew.guided.insert(*village);
        }
    }
}

fn stand_the_watch(
    world: &VoxelWorld,
    text_state: &mut TextState,
    commands: &mut Commands,
    door: [i32; 3],
) -> bool {
    let Some(ground) = footing(world, door[0], door[2], door[1]) else {
        return false;
    };
    let place = Vec3::new(
        door[0] as f32 + 0.5,
        ground as f32 + 1.0,
        door[2] as f32 + 0.5,
    );
    let giver = &blocker_quests::data::trade::GIVERS[blocker_quests::data::trade::WATCH_GIVER];
    let body = stand_a_body(
        commands,
        giver.name,
        place,
        theme::WATCH_LOOK,
        FIGURE_SCALE,
        1.0,
    );
    let spot = Vec2::new(place.x, place.z);
    commands.set(body, Haunt(spot));
    commands.set(body, Post(spot));
    commands.set(body, Rest(0.0));
    commands.set(body, Offer(blocker_quests::data::quests::OPENING));
    mark_the_giver(text_state, commands, body, place);
    plate_the_body(
        text_state,
        commands,
        body,
        &format!("<{}>\n{}", giver.title, giver.name),
        place,
        plate_properties(),
    );
    true
}

type Wanderers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Home,
        &'item Haunt,
        &'item Post,
        &'item mut Target,
        &'item Speed,
        &'item mut Rest,
        &'item mut Heading,
        &'item mut LocalTransform,
    ),
>;

pub fn wander(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    renderer_state: Res<RendererState>,
    config: Res<Config<Settings>>,
    mut random: ResMut<Random>,
    mut wanderers: Wanderers<'_, '_>,
) {
    let delta_time = time.delta_time.min(0.05);
    let hour = renderer_state.day_night.hour;
    let stride = config.number(Settings::Speed);
    let linger = config.number(Settings::Linger);
    let rest_min = config.number(Settings::RestMin);
    let rest_span = config.number(Settings::Rest);
    let day = Day {
        work: config.number(Settings::WorkHour),
        noon: config.number(Settings::NoonHour),
        afternoon: config.number(Settings::AfternoonHour),
        evening: config.number(Settings::EveningHour),
        bed: config.number(Settings::BedHour),
    };
    wanderers.for_each_mut(
        |_entity, (home, haunt, post, target, speed, rest, facing, transform)| {
            let center = where_the_day_wants(&day, hour, home.0, haunt.0, post.0);
            let ground = Vec2::new(transform.translation.x, transform.translation.z);
            let to_target = target.0 - ground;
            let distance = to_target.magnitude();
            if distance < 0.1 {
                rest.0 -= delta_time;
                if rest.0 <= 0.0 {
                    let turn = between(&mut random, 0.0, std::f32::consts::TAU);
                    let span = between(&mut random, 0.0, linger);
                    target.0 = center + Vec2::new(turn.cos(), turn.sin()) * span;
                    rest.0 = between(&mut random, rest_min, rest_min + rest_span);
                }
            } else {
                let step = (stride * speed.0 * delta_time).min(distance);
                let direction = to_target / distance;
                let next = ground + direction * step;
                let stand = transform.translation.y.floor() as i32;
                if step_holds(&world, ground, next, stand) {
                    transform.translation.x = next.x;
                    transform.translation.z = next.y;
                    let wanted = direction.x.atan2(direction.y) + theme::CREW_FACING;
                    let share = (theme::CREW_TURN * delta_time).clamp(0.0, 1.0);
                    facing.0 += angle_delta(facing.0, wanted) * share;
                } else {
                    target.0 = ground;
                    rest.0 = rest_min;
                }
            }
            transform.rotation = nalgebra_glm::quat_angle_axis(facing.0, &Vec3::y());
            if let Some(ground) = footing(
                &world,
                transform.translation.x.floor() as i32,
                transform.translation.z.floor() as i32,
                transform.translation.y.floor() as i32,
            ) {
                let wanted = ground as f32 + 1.0;
                let share = (theme::CREW_STEP * delta_time).clamp(0.0, 1.0);
                transform.translation.y += (wanted - transform.translation.y) * share;
            }
        },
    );
}

struct Day {
    work: f32,
    noon: f32,
    afternoon: f32,
    evening: f32,
    bed: f32,
}

fn where_the_day_wants(day: &Day, hour: f32, home: Vec2, plaza: Vec2, post: Vec2) -> Vec2 {
    if hour < day.work || hour >= day.bed {
        home
    } else if hour < day.noon {
        post
    } else if hour < day.afternoon {
        plaza
    } else if hour < day.evening {
        post
    } else {
        plaza
    }
}

pub fn stride(
    time: Res<Time>,
    config: Res<Config<Settings>>,
    mut rig: Query<(&Speed, &Target, &mut Gait, &mut Pace, &LocalTransform)>,
) {
    let delta_time = time.delta_time.min(0.05);
    let stride = config.number(Settings::Speed);
    rig.for_each_mut(|_entity, (speed, target, gait, pace, transform)| {
        let ground = Vec2::new(transform.translation.x, transform.translation.z);
        let moving = (target.0 - ground).magnitude() > 0.15;
        advance_the_stride(gait, pace, stride * speed.0, moving, 1.0, delta_time);
    });
}

fn stand_the_gate(
    world: &VoxelWorld,
    text_state: &mut TextState,
    commands: &mut Commands,
    city: [i32; 2],
) -> bool {
    let mut places = Vec::new();
    for (offset, persona) in theme::GATE_POSTS
        .iter()
        .zip(crate::data::people::GATE_PERSONAS)
    {
        let x = city[0] + offset[0];
        let z = city[1] + blocker_world::data::world::CITY_WALL + offset[1];
        let Some(top) = walkable(world, x, z) else {
            return false;
        };
        places.push((
            Vec3::new(x as f32 + 0.5, top as f32 + 1.0, z as f32 + 0.5),
            persona,
        ));
    }
    for (place, persona) in places {
        stand_a_person(
            text_state,
            commands,
            place,
            theme::WATCH_LOOK,
            persona,
            1.0,
            FIGURE_SCALE,
        );
    }
    true
}
