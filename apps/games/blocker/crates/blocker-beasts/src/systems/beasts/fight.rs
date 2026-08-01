use crate::commands::hurt_the_body;
use crate::components::{Angry, Beast, Flinch, Shove, Swing};
use crate::data::beasts::BEASTS;
use crate::data::settings::Settings;
use crate::resources::Quarry;
use crate::theme;
use blocker_blocks::data::items::Item;
use blocker_blocks::data::tools::ToolKind;
use blocker_blocks::queries::step_holds;
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::{Eye, Gaze, Heading};
use blocker_figure::queries::{looks_at, read_the_hero};
use blocker_figure::resources::Hero;
use blocker_player::data::ask::Ask;
use blocker_pocket::queries::selected_stack;
use blocker_pocket::resources::Inventory;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_health::{Health, queries::fallen};
use nightshade_input::Actions;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn pose_the_beasts(
    config: Res<Config<Settings>>,
    mut beasts: Query<(&Heading, &Swing, &mut LocalTransform), With<Beast>>,
) {
    let attack_period = config.number(Settings::AttackPeriod);
    beasts.for_each_mut(|_entity, (heading, swing, transform)| {
        let elapsed = attack_period - swing.0;
        let lean = if (0.0..=theme::STRIKE_SECONDS).contains(&elapsed) {
            (elapsed / theme::STRIKE_SECONDS * std::f32::consts::PI).sin() * theme::LUNGE_PITCH
        } else {
            0.0
        };
        transform.rotation = nalgebra_glm::quat_angle_axis(heading.0, &Vec3::y())
            * nalgebra_glm::quat_angle_axis(lean, &Vec3::x());
    });
}

pub fn shove_the_beasts(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    mut beasts: Query<(&mut Shove, &mut LocalTransform), With<Beast>>,
) {
    let delta = time.delta_time.min(0.05);
    beasts.for_each_mut(|_entity, (shove, transform)| {
        if shove.0.magnitude() < theme::KNOCKBACK_REST {
            if shove.0 != Vec2::zeros() {
                shove.0 = Vec2::zeros();
            }
            return;
        }
        let ground = Vec2::new(transform.translation.x, transform.translation.z);
        let next = ground + shove.0 * delta;
        let stand = transform.translation.y.floor() as i32;
        if step_holds(&world, ground, next, stand) {
            transform.translation.x = next.x;
            transform.translation.z = next.y;
        } else {
            shove.0 = Vec2::zeros();
            return;
        }
        let keep = (1.0 - theme::KNOCKBACK_DECAY * delta).clamp(0.0, 1.0);
        shove.0 *= keep;
    });
}

pub fn flinch_the_beasts(
    time: Res<Time>,
    mut beasts: Query<(&mut Flinch, &mut LocalTransform), With<Beast>>,
) {
    let delta = time.delta_time.min(0.05);
    beasts.for_each_mut(|_entity, (flinch, transform)| {
        if flinch.0 <= 0.0 {
            return;
        }
        flinch.0 = (flinch.0 - delta).max(0.0);
        let progress = 1.0 - flinch.0 / theme::FLINCH_SECONDS;
        let squash = (progress * std::f32::consts::PI).sin() * theme::FLINCH_SQUASH;
        transform.scale = Vec3::new(1.0 + squash, 1.0 - squash * 1.3, 1.0 + squash);
    });
}

pub fn fight_the_beasts(
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    inventory: Res<Inventory>,
    config: Res<Config<Settings>>,
    hero: Res<Hero>,
    mut quarry: ResMut<Quarry>,
    mut commands: Commands,
    mut beasts: Query<(
        &Beast,
        &Health,
        &mut Angry,
        &mut Swing,
        &mut Flinch,
        &mut Shove,
        &LocalTransform,
    )>,
    mut sighted: Query<(&Eye, &Gaze)>,
) {
    let Some(body) = hero.0 else {
        return;
    };
    let Some((eye, gaze)) = read_the_hero(&hero, &mut sighted, |(place, ahead)| (place.0, ahead.0))
    else {
        return;
    };
    let delta = time.delta_time.min(0.05);
    let damage = match selected_stack(&inventory).map(|stack| stack.item) {
        Some(Item::Tool(ToolKind::Sword)) => config.number(Settings::SwordDamage),
        Some(Item::Tool(ToolKind::Axe)) => config.number(Settings::AxeDamage),
        _ => config.number(Settings::FistDamage),
    };
    let player_melee = config.number(Settings::PlayerMelee);
    let melee = config.number(Settings::Melee);
    let anger = config.number(Settings::Anger);
    let knockback = config.number(Settings::Knockback);
    let attack_period = config.number(Settings::AttackPeriod);
    if actions.just_pressed(Ask::Mine) {
        let mut best: Option<(f32, Entity)> = None;
        if let Some(target) = quarry.0 {
            beasts.for_entity(target, |(beast, health, _, _, _, _, transform)| {
                if fallen(health) {
                    return;
                }
                let head = transform.translation + Vec3::new(0.0, BEASTS[beast.0].head, 0.0);
                let distance = (head - eye).magnitude();
                if distance < player_melee {
                    best = Some((distance, target));
                }
            });
        }
        if best.is_none() {
            beasts.for_each_mut(|entity, (beast, health, _, _, _, _, transform)| {
                if fallen(health) {
                    return;
                }
                let head = transform.translation + Vec3::new(0.0, BEASTS[beast.0].head, 0.0);
                if looks_at(
                    eye,
                    gaze,
                    head,
                    player_melee,
                    blocker_figure::data::sight::LOOK_COSINE,
                ) {
                    let distance = (head - eye).magnitude();
                    if best.is_none_or(|(nearest, _)| distance < nearest) {
                        best = Some((distance, entity));
                    }
                }
            });
        }
        if let Some((_, entity)) = best {
            beasts.for_entity(entity, |(_, _, angry, _, flinch, shove, transform)| {
                angry.0 = anger;
                flinch.0 = theme::FLINCH_SECONDS;
                let away = Vec2::new(
                    transform.translation.x - eye.x,
                    transform.translation.z - eye.z,
                );
                if away.magnitude() > 1.0e-3 {
                    shove.0 = away.normalize() * knockback;
                }
            });
            hurt_the_body(&mut commands, entity, damage);
        }
    }
    let player_ground = Vec2::new(eye.x, eye.z);
    beasts.for_each_mut(|entity, (beast, health, angry, swing, _, _, transform)| {
        if fallen(health) {
            return;
        }
        let ground = Vec2::new(transform.translation.x, transform.translation.z);
        let distance = (ground - player_ground).magnitude();
        swing.0 = (swing.0 - delta).max(0.0);
        if angry.0 > 0.0 && distance < melee && swing.0 <= 0.0 {
            hurt_the_body(&mut commands, body, BEASTS[beast.0].damage);
            swing.0 = attack_period;
            if quarry.0.is_none() {
                quarry.0 = Some(entity);
            }
        }
    });
}
