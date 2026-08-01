pub mod audio;
pub mod chat;
pub mod city;
pub mod creatures;
pub mod dungeons;
pub mod effects;
pub mod hamlet;
pub mod interiors;
pub mod items;
pub mod models;
pub mod random;
pub mod save;
pub mod scenery;
pub mod sky;
pub mod spells;

use crate::components::{Experience, Heading, Level, Stride};
use crate::contract::{Channel, ChatLine};
use crate::data::{ARRIVE_REACH, CHAT_KEEP, GATE_SPOT, SQUARE_SPOT};
use crate::queries::{footing, ground, in_city, level_threshold};
use crate::resources::{
    Banner, Banners, ChatLog, Flash, Flashes, Hit, HitSource, Hits, KillEvent, Scheduled, Slain,
    Walls,
};
use nightshade::prelude::{Commands, Entity, Vec2, Vec3, Vec4, nalgebra_glm};
use nightshade_scene::prelude::LocalTransform;

pub fn block(commands: &mut Commands, from: Vec2, to: Vec2) {
    commands.queue(move |_, resources| {
        resources.res_mut::<Walls>().segments.push((from, to));
    });
}

pub fn place_on_ground(transform: &mut LocalTransform, heading: &Heading, stride: &Stride) {
    let bob = (stride.0 * 6.0).sin().abs() * 0.05;
    transform.translation.y = footing(transform.translation.x, transform.translation.z) + bob;
    transform.rotation = nalgebra_glm::quat_angle_axis(heading.0, &Vec3::y());
}

pub fn walk_toward(
    transform: &mut LocalTransform,
    heading: &mut Heading,
    stride: &mut Stride,
    goal: Vec3,
    speed: f32,
    delta_time: f32,
) -> bool {
    let to_goal = Vec3::new(
        goal.x - transform.translation.x,
        0.0,
        goal.z - transform.translation.z,
    );
    let distance = to_goal.magnitude();
    if distance <= ARRIVE_REACH {
        return true;
    }
    let step = (speed * delta_time).min(distance);
    let direction = to_goal / distance;
    transform.translation += direction * step;
    stride.0 += step;
    heading.0 = (-direction.x).atan2(-direction.z);
    place_on_ground(transform, heading, stride);
    false
}

fn next_leg(here: Vec2, goal: Vec2) -> Vec2 {
    let inside = in_city(here, 0.0);
    if inside == in_city(goal, 0.0) {
        return goal;
    }
    let gate = GATE_SPOT;
    if (here - gate).magnitude() < ARRIVE_REACH * 2.0 {
        return if inside { goal } else { SQUARE_SPOT };
    }
    if inside && (here - SQUARE_SPOT).magnitude() > ARRIVE_REACH * 2.0 && here.y < SQUARE_SPOT.y {
        return SQUARE_SPOT;
    }
    gate
}

pub fn travel(
    transform: &mut LocalTransform,
    heading: &mut Heading,
    stride: &mut Stride,
    goal: Vec3,
    speed: f32,
    delta_time: f32,
) -> bool {
    let here = Vec2::new(transform.translation.x, transform.translation.z);
    let leg = next_leg(here, Vec2::new(goal.x, goal.z));
    if leg == Vec2::new(goal.x, goal.z) {
        return walk_toward(transform, heading, stride, goal, speed, delta_time);
    }
    walk_toward(transform, heading, stride, ground(leg), speed, delta_time);
    false
}

pub fn teleport(transform: &mut LocalTransform, heading: &Heading, stride: &Stride, spot: Vec2) {
    transform.translation = Vec3::new(spot.x, 0.0, spot.y);
    place_on_ground(transform, heading, stride);
}

pub fn push_chat(chat: &mut ChatLog, channel: Channel, line: String) {
    chat.lines.push_back((channel, line));
    while chat.lines.len() > CHAT_KEEP {
        chat.lines.pop_front();
    }
}

pub fn schedule_line(chat: &mut ChatLog, delay: f32, line: ChatLine) {
    chat.scheduled.push(Scheduled { delay, line });
}

pub fn record_hit(
    hits: &mut Hits,
    attacker: Entity,
    target: Entity,
    damage: f32,
    crit: bool,
    chill: bool,
    source: HitSource,
) {
    hits.pending.push(Hit {
        attacker,
        target,
        damage,
        crit,
        chill,
        source,
    });
}

pub fn record_kill(slain: &mut Slain, event: KillEvent) {
    slain.events.push(event);
}

pub fn flash(flashes: &mut Flashes, position: Vec3, text: String, color: Vec4, pop: f32) {
    flashes.pending.push(Flash {
        position,
        text,
        color,
        pop,
    });
}

pub fn banner(banners: &mut Banners, title: String, subtitle: String, color: Vec4) {
    banners.pending.push_back(Banner {
        title,
        subtitle,
        color,
    });
}

pub fn grant_experience(level: &mut Level, experience: &mut Experience, amount: u32) -> bool {
    experience.0 += amount;
    let mut leveled = false;
    loop {
        let threshold = level_threshold(level.0);
        if experience.0 < threshold {
            break;
        }
        experience.0 -= threshold;
        level.0 += 1;
        leveled = true;
    }
    leveled
}
