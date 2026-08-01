use crate::components::{
    BodyRadius, Bulk, Draft, Grip, Gust, Motion, Pull, Reach, Relic, Speaker, SpinRate,
};
use crate::data::{BodyView, Collider, REACH_FADE, RelicSpot, SpeakerSpot, Updraft};
use nightshade::ecs::ChildOf;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;

pub type Bodies<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item BodyRadius,
        &'item Pull,
        &'item Reach,
        &'item SpinRate,
        &'item Motion,
        &'item LocalTransform,
        &'item Name,
        &'item Grip,
    ),
>;

pub type Drafts<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Draft,
        &'item Gust,
        &'item LocalTransform,
        &'item ChildOf,
    ),
>;

pub type Features<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item Bulk, &'item LocalTransform, &'item ChildOf),
>;

pub type Speakers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item Speaker, &'item LocalTransform, &'item ChildOf),
>;

pub type Relics<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item Relic, &'item LocalTransform, &'item ChildOf),
>;

pub fn world_point(view: &BodyView, local: Vec3) -> Vec3 {
    view.center + nalgebra_glm::quat_rotate_vec3(&view.rotation, &(local * view.radius))
}

pub fn collect_speakers(speakers: Speakers<'_, '_>, views: &[BodyView]) -> Vec<SpeakerSpot> {
    let mut spots = Vec::new();
    speakers.for_each(|entity, (speaker, transform, parent)| {
        let Some(view) = views.iter().find(|view| view.entity == parent.0) else {
            return;
        };
        spots.push(SpeakerSpot {
            entity,
            index: speaker.0,
            center: world_point(view, transform.translation),
        });
    });
    spots
}

pub fn collect_relics(relics: Relics<'_, '_>, views: &[BodyView]) -> Vec<RelicSpot> {
    let mut spots = Vec::new();
    relics.for_each(|entity, (relic, transform, parent)| {
        let Some(view) = views.iter().find(|view| view.entity == parent.0) else {
            return;
        };
        spots.push(RelicSpot {
            entity,
            index: relic.0,
            body: view.entity,
            center: world_point(view, transform.translation),
        });
    });
    spots
}

pub fn collect_bodies(bodies: Bodies<'_, '_>) -> Vec<BodyView> {
    let mut views = Vec::new();
    bodies.for_each(
        |entity, (radius, pull, reach, spin, motion, transform, name, grip)| {
            views.push(BodyView {
                entity,
                name: name.0.clone(),
                center: transform.translation,
                rotation: transform.rotation,
                radius: radius.0,
                pull: pull.0,
                reach: reach.0,
                spin: spin.0,
                motion: motion.0,
                grip: grip.0,
            });
        },
    );
    views
}

pub fn collect_colliders(features: Features<'_, '_>, views: &[BodyView]) -> Vec<Collider> {
    let mut colliders = Vec::new();
    features.for_each(|_entity, (bulk, transform, parent)| {
        let Some(view) = views.iter().find(|view| view.entity == parent.0) else {
            return;
        };
        colliders.push(Collider {
            body: view.entity,
            center: world_point(view, transform.translation),
            radius: bulk.0,
        });
    });
    colliders
}

pub fn collect_drafts(drafts: Drafts<'_, '_>, views: &[BodyView]) -> Vec<Updraft> {
    let mut updrafts = Vec::new();
    drafts.for_each(|_entity, (draft, gust, transform, parent)| {
        let Some(view) = views.iter().find(|view| view.entity == parent.0) else {
            return;
        };
        let center = world_point(view, transform.translation);
        updrafts.push(Updraft {
            body: view.entity,
            center,
            radius: draft.0,
            lift: (center - view.center).normalize() * gust.0,
        });
    });
    updrafts
}

pub fn draft_at(position: Vec3, body: Entity, updrafts: &[Updraft]) -> Vec3 {
    updrafts
        .iter()
        .filter(|updraft| updraft.body == body)
        .filter(|updraft| (position - updraft.center).magnitude() < updraft.radius)
        .map(|updraft| updraft.lift)
        .sum()
}

pub fn altitude(position: Vec3, body: &BodyView) -> f32 {
    (position - body.center).magnitude() - body.radius
}

pub fn surface_gravity(body: &BodyView) -> f32 {
    body.pull / body.radius
}

pub fn gravity_at(position: Vec3, bodies: &[BodyView]) -> Vec3 {
    let mut total = Vec3::zeros();
    for body in bodies {
        let toward = body.center - position;
        let distance = toward.magnitude();
        if distance <= f32::EPSILON || distance >= body.reach {
            continue;
        }
        let strength = if distance < body.radius {
            body.pull * distance / (body.radius * body.radius)
        } else {
            body.pull / distance
        };
        let fade = if body.reach.is_finite() {
            ((body.reach - distance) / (body.reach * REACH_FADE)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        total += toward / distance * strength * fade;
    }
    total
}

pub fn nearest_body(position: Vec3, bodies: &[BodyView]) -> Option<usize> {
    bodies
        .iter()
        .enumerate()
        .min_by(|(_, first), (_, second)| {
            altitude(position, first).total_cmp(&altitude(position, second))
        })
        .map(|(index, _)| index)
}

pub fn orbit_point(radius: f32, phase: f32) -> Vec3 {
    Vec3::new(radius * phase.cos(), 0.0, radius * phase.sin())
}

pub fn orbit_velocity(speed: f32, phase: f32) -> Vec3 {
    Vec3::new(-speed * phase.sin(), 0.0, speed * phase.cos())
}

pub fn up_to(direction: Vec3) -> Quat {
    nalgebra_glm::quat_normalize(&nalgebra_glm::quat_rotation(&Vec3::y(), &direction))
}

pub fn level_on(outward: Vec3, forward: Vec3) -> Quat {
    let flat = forward - outward * forward.dot(&outward);
    let base = up_to(outward);
    if flat.magnitude() < 1.0e-3 {
        return base;
    }
    let flat = flat.normalize();
    let base_forward = nalgebra_glm::quat_rotate_vec3(&base, &Vec3::new(0.0, 0.0, -1.0));
    let turn = nalgebra_glm::quat_rotation(&base_forward, &flat);
    nalgebra_glm::quat_normalize(&(turn * base))
}

pub fn shade(color: [f32; 4], factor: f32) -> [f32; 4] {
    [
        (color[0] * factor).min(1.0),
        (color[1] * factor).min(1.0),
        (color[2] * factor).min(1.0),
        color[3],
    ]
}
