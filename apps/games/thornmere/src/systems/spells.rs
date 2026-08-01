use crate::components::{Age, Aura, Form, Health, Home, Lift, Peak, Span, Spin};
use crate::data::Shape;
use nightshade::prelude::*;
use nightshade_graphics::prelude::Light;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use std::collections::HashMap;

fn smooth(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn bell(t: f32) -> f32 {
    let sine = (std::f32::consts::PI * t.clamp(0.0, 1.0)).sin();
    sine * sine
}

type Effects<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Age,
        &'item Span,
        &'item Form,
        &'item Peak,
        &'item Spin,
        &'item Home,
        &'item Lift,
        &'item mut LocalTransform,
    ),
>;

pub fn animate(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Effects<'_, '_>,
    mut lamps: Query<(&mut Age, &Span, &Peak, &mut Light), With<Form>>,
) {
    let delta_time = time.delta_time;
    let mut doomed = Vec::new();
    effects.for_each_mut(
        |entity, (age, span, form, peak, spin, home, lift, transform)| {
            age.0 += delta_time;
            let t = age.0 / span.0;
            if t >= 1.0 {
                doomed.push(entity);
                return;
            }
            let peak = peak.0;
            match form.0 {
                Shape::Blast => {
                    let size = peak * smooth(t / 0.22) * (1.0 - smooth((t - 0.45) / 0.55));
                    let size = size.max(0.01);
                    transform.scale = Vec3::new(size, size, size);
                }
                Shape::Shard => {
                    let rise = smooth(t / 0.1);
                    let melt = 1.0 - smooth((t - 0.78) / 0.22);
                    let height = (peak * rise * melt).max(0.01);
                    transform.scale = Vec3::new(height * 0.3, height, height * 0.3);
                    transform.translation = home.0 + Vec3::new(0.0, height * 0.45, 0.0);
                    let tilt = nalgebra_glm::quat_angle_axis(0.35 + spin.0, &Vec3::x());
                    let turn = nalgebra_glm::quat_angle_axis(lift.0, &Vec3::y());
                    transform.rotation = turn * tilt;
                }
                Shape::Slash => {
                    let radius = (peak * (0.35 + 0.65 * smooth(t / 0.6))).max(0.01);
                    let thickness = (peak * 0.08 * (1.0 - smooth((t - 0.3) / 0.7))).max(0.005);
                    transform.scale = Vec3::new(radius, thickness, radius);
                    let flat = nalgebra_glm::quat_angle_axis(0.0, &Vec3::x());
                    let tilt = nalgebra_glm::quat_angle_axis(spin.0, &Vec3::y())
                        * nalgebra_glm::quat_angle_axis(-1.2, &Vec3::x());
                    transform.rotation = if spin.0 == 0.0 { flat } else { tilt };
                }
                Shape::Whirl => {
                    let radius = (peak * bell(t).sqrt()).max(0.01);
                    transform.scale = Vec3::new(radius, peak * 0.07, radius);
                    transform.rotation = nalgebra_glm::quat_angle_axis(age.0 * spin.0, &Vec3::y());
                }
                Shape::Halo => {
                    let radius = (peak * (1.0 - 0.6 * smooth(t)) * bell(t.min(0.5) * 2.0).max(0.2))
                        .max(0.01);
                    let thickness = (peak * 0.09 * (1.0 - smooth((t - 0.6) / 0.4))).max(0.005);
                    transform.scale = Vec3::new(radius, thickness, radius);
                    transform.translation = home.0 + Vec3::new(0.0, lift.0 * smooth(t), 0.0);
                    transform.rotation = nalgebra_glm::quat_angle_axis(age.0 * spin.0, &Vec3::y());
                }
                Shape::Patch => {
                    let size =
                        (peak * smooth(t / 0.08) * (1.0 - smooth((t - 0.55) / 0.45))).max(0.01);
                    transform.scale = Vec3::new(size, 0.03, size);
                    transform.translation = home.0 + Vec3::new(0.0, 0.05, 0.0);
                }
                Shape::Lamp => {}
            }
        },
    );
    lamps.for_each_mut(|entity, (age, span, peak, light)| {
        age.0 += delta_time;
        let t = age.0 / span.0;
        if t >= 1.0 {
            light.intensity = 0.0;
            doomed.push(entity);
            return;
        }
        light.intensity = peak.0 * smooth(t / 0.08) * (1.0 - smooth((t - 0.25) / 0.75));
    });
    for entity in doomed {
        commands.despawn_recursive(entity);
    }
}

pub fn auras(
    hosts: Query<(&LocalTransform,), With<Health>>,
    mut emitters: Query<(&Aura, &Lift, &mut ParticleEmitter)>,
) {
    let mut spots: HashMap<Entity, Vec3> = HashMap::new();
    hosts.for_each(|entity, (transform,)| {
        spots.insert(entity, transform.translation);
    });
    emitters.for_each_mut(|_entity, (aura, lift, emitter)| {
        if let Some(target) = aura.0
            && let Some(spot) = spots.get(&target)
        {
            emitter.position = spot + Vec3::new(0.0, lift.0, 0.0);
        }
    });
}
