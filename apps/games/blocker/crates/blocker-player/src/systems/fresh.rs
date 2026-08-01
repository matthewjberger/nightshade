use crate::components::{
    Air, Boat, Fall, Food, Grounded, Held, Motion, Pitch, Riding, Spawn, Submerged, Velocity,
    WatchSpeed, Yaw, Zoom, ZoomWanted,
};
use crate::resources::{Fireworks, Hand, Resume};
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::resources::Hero;
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use nightshade_health::Health;

type Vitals<'world, 'item> = Query<
    'world,
    (
        &'item mut Health,
        &'item mut Food,
        &'item mut Air,
        &'item mut Fall,
        &'item mut Spawn,
        &'item mut Velocity,
        &'item mut Motion,
        &'item mut Grounded,
    ),
>;

type Bearing<'world, 'item> = Query<
    'world,
    (
        &'item mut Yaw,
        &'item mut Pitch,
        &'item mut Eye,
        &'item mut Gaze,
        &'item mut Zoom,
        &'item mut ZoomWanted,
        &'item mut Submerged,
        &'item mut WatchSpeed,
    ),
>;

pub fn stand_anew(
    fresh: EventReader<NewWorld>,
    hero: Res<Hero>,
    mut hand: ResMut<Hand>,
    mut fireworks: ResMut<Fireworks>,
    mut resume: ResMut<Resume>,
    mut commands: Commands,
    dropped: Query<(&Held,)>,
    boats: Query<(&Boat,)>,
    mut vitals: Vitals<'_, '_>,
    mut bearing: Bearing<'_, '_>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    resume.0 = None;
    let mut gone: Vec<Entity> = Vec::new();
    dropped.for_each(|entity, _| gone.push(entity));
    boats.for_each(|entity, _| gone.push(entity));
    gone.extend(hand.entity.take());
    *hand = Hand::default();
    for shell in fireworks.shells.drain(..) {
        commands.despawn(shell.entity);
    }
    if !gone.is_empty() {
        commands.queue(move |storage, resources| {
            despawn_entities_with_cache_cleanup(storage, resources, &gone);
        });
    }
    let Some(body) = hero.0 else {
        return;
    };
    vitals.for_entity(
        body,
        |(health, food, air, fall, spawn, velocity, motion, grounded)| {
            health.0 = crate::data::body::FULL_HEALTH;
            food.0 = crate::data::body::FULL_FOOD;
            *air = Air::default();
            *fall = Fall::default();
            *spawn = Spawn::default();
            *velocity = Velocity::default();
            *motion = Motion::Walk;
            *grounded = Grounded::default();
        },
    );
    bearing.for_entity(
        body,
        |(yaw, pitch, eye, gaze, zoom, zoom_wanted, submerged, watch_speed)| {
            *yaw = Yaw::default();
            *pitch = Pitch::default();
            *eye = Eye::default();
            *gaze = Gaze::default();
            *zoom = Zoom::default();
            *zoom_wanted = ZoomWanted::default();
            *submerged = Submerged::default();
            *watch_speed = WatchSpeed::default();
        },
    );
    commands.remove::<(Riding,)>(body);
}
