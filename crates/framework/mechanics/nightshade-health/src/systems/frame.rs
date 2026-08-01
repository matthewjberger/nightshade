use crate::components::{Armor, Bounty, Health, PendingDamage, Shield};
use crate::data::{Died, Hurt};
use crate::queries::after_armor;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;

pub fn take_the_damage(
    mut hurt: EventWriter<Hurt>,
    mut died: EventWriter<Died>,
    bodies: Query<(
        &mut Health,
        &mut Shield,
        &Armor,
        &mut PendingDamage,
        &Bounty,
    )>,
) {
    bodies.for_each(|entity, (health, shield, armor, pending, bounty)| {
        if pending.0 <= 0.0 {
            return;
        }
        let mut amount = after_armor(pending.0, armor);
        pending.0 = 0.0;
        if shield.0 > 0.0 {
            let stopped = amount.min(shield.0);
            shield.0 -= stopped;
            amount -= stopped;
        }
        if amount <= 0.0 {
            hurt.send(Hurt {
                entity,
                amount: 0.0,
            });
            return;
        }
        let before = health.0;
        health.0 = (health.0 - amount).max(0.0);
        hurt.send(Hurt { entity, amount });
        if before > 0.0 && health.0 <= 0.0 {
            died.send(Died {
                entity,
                bounty: bounty.0,
            });
        }
    });
}
