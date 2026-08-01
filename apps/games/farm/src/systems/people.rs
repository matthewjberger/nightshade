use crate::commands::life::set_hidden;

use crate::components::{
    Arrive, Home, House, Inside, Npc, Post, Reach, Rest, Shadow, Stuck, Target,
};
use crate::queries::clock::is_waking_hour;
use crate::resources::Weather;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::*;

pub type Plans<'world, 'item> = Query<
    'world,
    (
        &'item Npc,
        &'item House,
        &'item mut Post,
        &'item Arrive,
        &'item mut Inside,
        &'item mut Stuck,
        &'item Shadow,
    ),
>;

pub type Steps<'world, 'item> = Query<
    'world,
    (
        &'item mut Home,
        &'item mut Target,
        &'item mut Reach,
        &'item mut Rest,
        &'item mut LocalTransform,
    ),
>;

nightshade::ecs::query_bundle! {
    pub struct Walkers<'world, 'item> {
        pub plans: Plans<'world, 'item>,
        pub steps: Steps<'world, 'item>,
    }
}

#[derive(Default)]
struct March {
    warp: Option<Vec2>,
    home: Option<Vec2>,
    target: Option<Vec2>,
    reach: Option<f32>,
    rest: Option<f32>,
}

pub fn keep_the_schedule(
    time: Res<Time>,
    renderer_state: Res<RendererState>,
    weather: Res<Weather>,
    mut commands: Commands,
    mut people: Walkers<'_, '_>,
) {
    let hour = renderer_state.day_night.hour;
    let indoor_weather = weather.raining;
    let awake = is_waking_hour(hour) && !indoor_weather;
    let delta_time = time.delta_time;

    let mut walks: Vec<(Entity, Vec2, Vec2, Vec2, f32)> = Vec::new();
    people
        .steps
        .for_each_mut(|entity, (home, target, reach, _, transform)| {
            let ground = Vec2::new(transform.translation.x, transform.translation.z);
            walks.push((entity, ground, home.0, target.0, reach.0));
        });

    let mut orders: Vec<(Entity, March)> = Vec::new();
    people.plans.for_each_mut(
        |entity, (npc, house, post, arrive, inside, stuck, shadow)| {
            let Some(house) = house.0 else {
                return;
            };
            let Some(&(_, ground, walk_home, walk_target, walk_reach)) =
                walks.iter().find(|(known, ..)| *known == entity)
            else {
                return;
            };
            let mut march = March::default();
            if awake {
                if inside.0 {
                    inside.0 = false;
                    stuck.0 = 0.0;
                    march.warp = Some(house);
                    set_hidden(&mut commands, entity, shadow.0, false);
                }
                let at = npc.person.post_at(hour);
                if post.0 != at {
                    post.0 = at;
                }
                let mut reach_now = walk_reach;
                if walk_home != post.0 {
                    march.home = Some(post.0);
                    march.target = Some(post.0);
                    march.reach = Some(arrive.0);
                    march.rest = Some(0.0);
                    reach_now = arrive.0;
                }
                let aimed = march.target.unwrap_or(walk_target);
                let far = (ground - post.0).magnitude() > reach_now + 0.5;
                if far && (aimed - ground).magnitude() < 0.1 {
                    stuck.0 += delta_time;
                    march.target = Some(post.0);
                    if stuck.0 > 4.0 {
                        march.warp = Some(post.0);
                        stuck.0 = 0.0;
                    }
                }
                orders.push((entity, march));
                return;
            }
            if inside.0 {
                return;
            }
            if walk_home != house {
                march.home = Some(house);
                march.target = Some(house);
                march.reach = Some(0.0);
                march.rest = Some(0.0);
            }
            let aimed = march.target.unwrap_or(walk_target);
            let arrived = (ground - house).magnitude() < 0.6;
            let stalled = (aimed - ground).magnitude() < 0.1;
            if stalled && !arrived {
                stuck.0 += delta_time;
                march.target = Some(house);
            }
            if arrived || stuck.0 > 6.0 {
                inside.0 = true;
                stuck.0 = 0.0;
                set_hidden(&mut commands, entity, shadow.0, true);
            }
            orders.push((entity, march));
        },
    );

    people
        .steps
        .for_each_mut(|entity, (home, target, reach, rest, transform)| {
            let Some((_, march)) = orders.iter().find(|(known, _)| *known == entity) else {
                return;
            };
            if let Some(at) = march.warp {
                transform.translation.x = at.x;
                transform.translation.z = at.y;
            }
            if let Some(spot) = march.home {
                home.0 = spot;
            }
            if let Some(spot) = march.target {
                target.0 = spot;
            }
            if let Some(span) = march.reach {
                reach.0 = span;
            }
            if let Some(pause) = march.rest {
                rest.0 = pause;
            }
        });
}
