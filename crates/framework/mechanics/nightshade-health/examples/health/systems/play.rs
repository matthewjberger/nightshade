use crate::resources::Rounds;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::prelude::*;
use nightshade_health::commands::{heal, hurt};
use nightshade_health::queries::{fallen, share, shielded};
use nightshade_health::{Armor, Died, Health, MaxHealth, PendingDamage, Shield};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    mut rounds: ResMut<Rounds>,
    mut bodies: Query<(&mut PendingDamage, &mut Health, &MaxHealth)>,
) {
    if rounds.bodies.is_empty() {
        return;
    }
    let last = rounds.bodies.len() - 1;
    if input.keyboard.just_pressed(KeyCode::ArrowLeft) {
        rounds.chosen = rounds.chosen.saturating_sub(1);
    }
    if input.keyboard.just_pressed(KeyCode::ArrowRight) {
        rounds.chosen = (rounds.chosen + 1).min(last);
    }
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        bodies.for_each_mut(|_entity, (_pending, health, ceiling)| {
            heal(&mut health.0, ceiling.0, ceiling.0);
        });
        rounds.fallen = 0;
        rounds.bounty = 0;
        return;
    }
    let blow = input.keyboard.just_pressed(KeyCode::Space);
    let mend = input.keyboard.just_pressed(KeyCode::KeyF);
    if !blow && !mend {
        return;
    }
    let body = rounds.bodies[rounds.chosen];
    bodies.for_entity(body, |(pending, health, ceiling)| {
        if blow {
            hurt(pending, theme::BLOW);
        }
        if mend {
            heal(&mut health.0, ceiling.0, theme::MEND);
        }
    });
}

pub fn show_the_share(
    rounds: Res<Rounds>,
    bodies: Query<(&mut LocalTransform, &Health, &MaxHealth)>,
) {
    let chosen = rounds.bodies.get(rounds.chosen).copied();
    bodies.for_each(|entity, (transform, health, ceiling)| {
        let left = share(health, ceiling).max(0.02);
        transform.scale = vec3(1.0, 2.0 * left, 1.0);
        transform.translation.y = left;
        transform.translation.z = if Some(entity) == chosen {
            theme::STEP_OUT
        } else {
            0.0
        };
    });
}

pub fn watch_the_bodies(
    died: EventReader<Died>,
    mut rounds: ResMut<Rounds>,
    mut readout: ResMut<Readout>,
    mut bodies: Query<(&Health, &MaxHealth, &Shield, &Armor)>,
) {
    for word in died.iter() {
        rounds.fallen += 1;
        rounds.bounty += word.bounty;
    }
    readout.notes.clear();
    readout.notes.push(format!(
        "body {} of {}",
        rounds.chosen + 1,
        rounds.bodies.len()
    ));
    if let Some(body) = rounds.bodies.get(rounds.chosen).copied() {
        bodies.for_entity(body, |(health, ceiling, shield, armor)| {
            readout
                .notes
                .push(format!("health {:.0} of {:.0}", health.0, ceiling.0));
            readout
                .notes
                .push(format!("shield {:.0}, armor {:.0}", shield.0, armor.0));
            if shielded(shield) {
                readout.notes.push("shield holding".to_string());
            }
            if fallen(health) {
                readout.notes.push("down".to_string());
            }
        });
    }
    readout.notes.push(format!("{} fallen", rounds.fallen));
    readout.notes.push(format!("bounty {}", rounds.bounty));
}
