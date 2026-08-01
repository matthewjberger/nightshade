use crate::commands::{darken_the_hide, post_message, wound};
use crate::components::{Dwell, Facing, Hunt, Interest, Puppet, Stride, Waypoint};
use crate::contract::HuntState;
use crate::data::route::{WAYPOINTS, WOLF_DEN};
use crate::data::rules::*;
use crate::data::settings::Settings;
use crate::data::theme;
use crate::queries::{nearest_waypoint, next_hop};
use crate::resources::{BlackmarrowResources, Screen};
use crate::tags::Wolf;
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_config::prelude::Config;
use nightshade_gltf::Loading;
use nightshade_load::take;
use nightshade_physics::commands::physics_world_set_translation;
use nightshade_physics::contract::components::{ColliderComponent, RigidBodyComponent};
use nightshade_physics::queries::physics_world_cast_ray;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::Time;
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab_with_skins};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::prelude::{ChildOf, GlobalTransform, LocalTransform, Name};
use nightshade_senses::Noises;

const CLIP_PROWL: &str = "Walk";
const CLIP_CHARGE: &str = "Gallop";
const CLIP_REST: &str = "Idle";

pub fn raise(
    mut resources: ResMut<BlackmarrowResources>,
    mut loading: ResMut<Loading>,
    mut materials: ResMut<MaterialRegistry>,
    parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    if resources.models.wolf_spawned {
        return;
    }
    let Some(handle) = resources.models.wolf_handle else {
        return;
    };
    let Some(outcome) = take(&mut loading, handle) else {
        return;
    };
    let Ok(mut import) = outcome else {
        resources.models.wolf_spawned = true;
        return;
    };

    let clip = |name: &str| {
        import
            .animations
            .iter()
            .position(|entry| entry.name.eq_ignore_ascii_case(name))
    };
    let prowl = clip(CLIP_PROWL);
    let charge = clip(CLIP_CHARGE);
    let rest = clip(CLIP_REST);
    let clips = import.animations.clone();
    let skins = import.skins.clone();
    let Some(prefab) = import.prefabs.first().cloned() else {
        resources.models.wolf_spawned = true;
        return;
    };

    let den = WAYPOINTS[WOLF_DEN].at;
    let center = theme::WOLF_START + Vec3::new(0.0, WOLF_RADIUS, 0.0);
    let body = storage.spawn((
        Name("Wolf".to_string()),
        LocalTransform {
            translation: center,
            ..Default::default()
        },
        GlobalTransform::default(),
        RigidBodyComponent::new_kinematic().with_translation(center.x, center.y, center.z),
        ColliderComponent::new_ball(WOLF_RADIUS),
    ));
    storage.add_tag_type::<Wolf>(body);
    storage.set(body, Hunt(HuntState::Dormant));
    storage.set(body, Interest(den));
    storage.set(body, Dwell(0.0));
    storage.set(body, Stride(WOLF_PROWL));
    storage.set(body, Waypoint(WOLF_DEN));
    storage.set(body, Facing(0.0));

    let visual = {
        let mut spawn = PrefabSpawn { storage, parts };
        queue_model_load(&mut spawn, &mut import);
        spawn_prefab_with_skins(&mut spawn, &prefab, &clips, &skins, Vec3::zeros())
    };
    if let Some(transform) = storage.get_mut::<LocalTransform>(visual) {
        transform.translation = Vec3::new(0.0, -WOLF_RADIUS, 0.0);
        transform.scale = Vec3::new(WOLF_SCALE, WOLF_SCALE, WOLF_SCALE);
    }
    storage.set(visual, ChildOf(body));
    storage.set(body, Puppet(Some(visual)));
    darken_the_hide(storage, &mut materials, visual);
    if let Some(index) = rest
        && let Some(player) = storage.get_mut::<AnimationPlayer>(visual)
    {
        player.play(index);
        player.looping = true;
    }

    let state = &mut resources.models;
    state.clip_prowl = prowl;
    state.clip_charge = charge;
    state.clip_rest = rest;
    state.wolf_spawned = true;
}

pub type Pack<'world, 'item> = Query<
    'world,
    (
        &'item mut Hunt,
        &'item mut Interest,
        &'item mut Dwell,
        &'item mut Stride,
        &'item mut Waypoint,
        &'item mut Facing,
        &'item LocalTransform,
    ),
    WithTag<Wolf>,
>;

pub fn update(
    time: Res<Time>,
    noises: Res<Noises>,
    config: Res<Config<Settings>>,
    mut physics: ResMut<PhysicsWorld>,
    mut resources: ResMut<BlackmarrowResources>,
    mut pack: Pack<'_, '_>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Playing {
        return;
    }
    let Some(player_body) = resources.player.body else {
        return;
    };
    let delta = time.delta_time;
    let quarry = resources.player.at;
    let mut steps: Vec<(Entity, Vec3)> = Vec::new();
    let mut bites = 0;

    pack.for_each_mut(
        |entity, (hunt, interest, dwell, stride, waypoint, facing, transform)| {
            let here = transform.translation;
            let seen = sees(&physics, entity, here, quarry, player_body);
            let heard = loudest(&noises, here);

            match hunt.0 {
                HuntState::Feeding => {
                    dwell.0 -= delta;
                    if dwell.0 <= 0.0 {
                        hunt.0 = HuntState::Hunting;
                    }
                    return;
                }
                HuntState::Dormant => {
                    if seen {
                        hunt.0 = HuntState::Hunting;
                    } else if let Some(at) = heard {
                        hunt.0 = HuntState::Searching;
                        interest.0 = at;
                        dwell.0 = WOLF_PATIENCE;
                    }
                }
                HuntState::Searching => {
                    if seen {
                        hunt.0 = HuntState::Hunting;
                    } else {
                        if let Some(at) = heard {
                            interest.0 = at;
                            dwell.0 = WOLF_PATIENCE;
                        }
                        dwell.0 -= delta;
                        if dwell.0 <= 0.0 {
                            hunt.0 = HuntState::Dormant;
                            interest.0 = WAYPOINTS[WOLF_DEN].at;
                        }
                    }
                }
                HuntState::Hunting => {
                    if seen {
                        interest.0 = quarry;
                        dwell.0 = WOLF_PATIENCE;
                    } else {
                        dwell.0 -= delta;
                        if dwell.0 <= 0.0 {
                            hunt.0 = HuntState::Searching;
                            dwell.0 = WOLF_SETTLE;
                        }
                    }
                }
            }

            stride.0 = match hunt.0 {
                HuntState::Hunting => WOLF_CHARGE,
                HuntState::Searching => WOLF_PROWL * 1.6,
                _ => WOLF_PROWL,
            };

            let goal = if hunt.0 == HuntState::Dormant {
                WAYPOINTS[WOLF_DEN].at
            } else {
                interest.0
            };
            let target = if seen && hunt.0 == HuntState::Hunting {
                quarry
            } else {
                let wanted = nearest_waypoint(goal);
                let mine = nearest_waypoint(here);
                waypoint.0 = mine;
                let hop = next_hop(mine, wanted);
                if (WAYPOINTS[mine].at.xz() - goal.xz()).magnitude() < WOLF_ARRIVAL * 2.0 {
                    goal
                } else {
                    WAYPOINTS[hop].at
                }
            };

            let toward = Vec3::new(target.x - here.x, 0.0, target.z - here.z);
            let span = toward.magnitude();
            if span > WOLF_ARRIVAL {
                let direction = toward / span;
                facing.0 = turn_toward(facing.0, direction.x.atan2(direction.z), delta);
                let step = here + direction * stride.0 * delta;
                steps.push((entity, Vec3::new(step.x, WOLF_RADIUS, step.z)));
            }

            if (Vec3::new(quarry.x, 0.0, quarry.z) - Vec3::new(here.x, 0.0, here.z)).magnitude()
                < WOLF_REACH
                && resources.player.bite_cooldown <= 0.0
            {
                bites += 1;
                hunt.0 = HuntState::Feeding;
                dwell.0 = WOLF_BITE_INTERVAL;
            }
        },
    );

    for (entity, at) in steps {
        physics_world_set_translation(&mut physics, entity, at);
    }
    if bites > 0 {
        resources.player.bite_cooldown = WOLF_BITE_INTERVAL;
        let bite = if config.toggle(Settings::Invulnerable) {
            0.0
        } else {
            WOLF_BITE * bites as f32
        };
        wound(&mut resources.player, &mut resources.run, bite);
        post_message(&mut resources.run, "Teeth. It has you.");
    }
}

pub fn animate(
    resources: Res<BlackmarrowResources>,
    mut pack: Query<(&Hunt, &Facing, &Puppet), WithTag<Wolf>>,
    mut puppets: Query<(&mut LocalTransform, &mut AnimationPlayer)>,
) {
    let models = &resources.models;
    let mut wanted: Vec<(Entity, f32, Option<usize>)> = Vec::new();
    pack.for_each_mut(|_entity, (hunt, facing, puppet)| {
        let Some(visual) = puppet.0 else {
            return;
        };
        let clip = match hunt.0 {
            HuntState::Hunting => models.clip_charge,
            HuntState::Searching => models.clip_prowl,
            _ => models.clip_rest,
        };
        wanted.push((visual, facing.0, clip));
    });
    for (visual, facing, clip) in wanted {
        puppets.for_entity(visual, |(transform, player)| {
            transform.rotation = nalgebra_glm::quat_angle_axis(facing, &Vec3::y());
            if let Some(index) = clip
                && player.current_clip != Some(index)
            {
                player.play(index);
                player.looping = true;
            }
        });
    }
}

fn sees(physics: &PhysicsWorld, wolf: Entity, here: Vec3, quarry: Vec3, body: Entity) -> bool {
    let head = here + Vec3::new(0.0, 0.35, 0.0);
    let eye = quarry + Vec3::new(0.0, 0.4, 0.0);
    let toward = eye - head;
    let span = toward.magnitude();
    if span > WOLF_SIGHT || span < 0.001 {
        return false;
    }
    let direction = toward / span;
    match physics_world_cast_ray(physics, head, direction, span, Some(wolf)) {
        Some(hit) => hit.entity == body,
        None => true,
    }
}

fn loudest(noises: &Noises, here: Vec3) -> Option<Vec3> {
    let mut best: Option<(Vec3, f32)> = None;
    for noise in &noises.list {
        let span = (noise.at.xz() - here.xz()).magnitude();
        let carry = WOLF_HEARING * noise.loudness;
        if span > carry {
            continue;
        }
        let strength = carry - span;
        if best.is_none_or(|(_, found)| strength > found) {
            best = Some((noise.at, strength));
        }
    }
    best.map(|(at, _)| at)
}

fn turn_toward(from: f32, to: f32, delta: f32) -> f32 {
    let mut difference = to - from;
    while difference > std::f32::consts::PI {
        difference -= std::f32::consts::TAU;
    }
    while difference < -std::f32::consts::PI {
        difference += std::f32::consts::TAU;
    }
    let step = (WOLF_TURN * delta).min(difference.abs());
    from + step * difference.signum()
}
