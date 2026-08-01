use crate::data::battle::{BOARD_LIMIT, Battle, Target, Who};
use crate::data::cards::{Kind, Spell, TargetRule};
use crate::data::decks::HeroPower;
use crate::queries::battle::{attack_targets, can_play, can_use_power, power_rule, targets_for};

#[derive(Clone, Copy, Default)]
pub struct Plan {
    pub strike: i32,
    pub power: i32,
    pub guarded: bool,
    pub lethal: bool,
}

impl Plan {
    pub fn total(self) -> i32 {
        self.strike + self.power
    }
}

pub fn warden_plan(battle: &Battle) -> Plan {
    let enemy = battle.side(Who::Enemy);
    let strike: i32 = enemy
        .board
        .iter()
        .filter(|minion| minion.attack > 0 && !minion.frozen)
        .map(|minion| minion.attack)
        .sum();
    let power = match enemy.power {
        HeroPower::Spark | HeroPower::Bolt => 1,
        HeroPower::Mend | HeroPower::Sprout => 0,
    };
    let guarded = battle.player.has_taunt();
    let standing = battle.player.hero_health + battle.player.armor;
    Plan {
        strike,
        power,
        guarded,
        lethal: !guarded && strike + power >= standing,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Play { id: u32, target: Option<Target> },
    Attack { attacker: u32, target: Target },
    Power { target: Option<Target> },
    End,
}

pub fn next_action(battle: &Battle) -> Action {
    let who = Who::Enemy;
    if let Some(action) = best_play(battle, who) {
        return action;
    }
    if let Some(action) = best_attack(battle, who) {
        return action;
    }
    if can_use_power(battle, who)
        && let Some(action) = power_action(battle, who)
    {
        return action;
    }
    Action::End
}

fn best_play(battle: &Battle, who: Who) -> Option<Action> {
    let side = battle.side(who);
    let mut playable: Vec<_> = side
        .hand
        .iter()
        .copied()
        .filter(|instance| can_play(battle, who, instance.id))
        .collect();
    playable.sort_by_key(|instance| std::cmp::Reverse(instance.card.def().cost));
    for instance in playable {
        let def = instance.card.def();
        match def.kind {
            Kind::Minion { .. } => {
                return Some(Action::Play {
                    id: instance.id,
                    target: None,
                });
            }
            Kind::Spell { effect, target } => {
                if let Some(chosen) = spell_target(battle, who, effect, target) {
                    return Some(Action::Play {
                        id: instance.id,
                        target: chosen,
                    });
                }
            }
        }
    }
    None
}

fn spell_target(
    battle: &Battle,
    who: Who,
    effect: Spell,
    rule: TargetRule,
) -> Option<Option<Target>> {
    let foe = crate::data::battle::other(who);
    let foe_side = battle.side(foe);
    let own = battle.side(who);
    match effect {
        Spell::Damage(amount) => {
            let killable = foe_side
                .board
                .iter()
                .filter(|minion| minion.health <= amount && !minion.shield)
                .max_by_key(|minion| minion.attack);
            if let Some(minion) = killable
                && minion.attack >= 2
            {
                return Some(Some(Target::Minion(foe, minion.id)));
            }
            if foe_side.hero_health + foe_side.armor <= amount * 2 || own.aggressive {
                return Some(Some(Target::Hero(foe)));
            }
            let biggest = foe_side.board.iter().max_by_key(|minion| minion.attack);
            match biggest {
                Some(minion) if minion.attack >= 4 => Some(Some(Target::Minion(foe, minion.id))),
                _ => Some(Some(Target::Hero(foe))),
            }
        }
        Spell::Heal(_) => {
            if own.hero_health <= own.hero_max - 6 {
                Some(Some(Target::Hero(who)))
            } else {
                None
            }
        }
        Spell::Buff(..) | Spell::Shield => own
            .board
            .iter()
            .max_by_key(|minion| minion.attack + minion.health)
            .map(|minion| Some(Target::Minion(who, minion.id))),
        Spell::Storm(amount) => {
            let hits = foe_side
                .board
                .iter()
                .filter(|minion| minion.health <= amount)
                .count();
            if hits >= 2 || (hits >= 1 && foe_side.board.len() >= 3) {
                Some(None)
            } else {
                None
            }
        }
        Spell::Draw(_) => {
            if own.hand.len() <= 3 {
                Some(None)
            } else {
                None
            }
        }
        Spell::Freeze => foe_side
            .board
            .iter()
            .filter(|minion| !minion.frozen && minion.attack >= 3)
            .max_by_key(|minion| minion.attack)
            .map(|minion| Some(Target::Minion(foe, minion.id))),
        Spell::Summon(..) => {
            if own.board.len() + 1 < BOARD_LIMIT {
                Some(None)
            } else {
                None
            }
        }
        Spell::Armor(_) => {
            if own.armor <= 4 {
                Some(None)
            } else {
                None
            }
        }
    }
    .filter(|chosen| match (rule, chosen) {
        (TargetRule::None, None) => true,
        (TargetRule::None, Some(_)) | (_, None) => false,
        (rule, Some(target)) => targets_for(battle, who, rule).contains(target),
    })
}

fn best_attack(battle: &Battle, who: Who) -> Option<Action> {
    let foe = crate::data::battle::other(who);
    let side = battle.side(who);
    let foe_side = battle.side(foe);
    let targets = attack_targets(battle, who);
    let mut attackers: Vec<_> = side
        .board
        .iter()
        .filter(|minion| minion.can_attack())
        .collect();
    attackers.sort_by_key(|minion| std::cmp::Reverse(minion.attack));
    let attacker = attackers.first()?;

    let lethal = foe_side.hero_health + foe_side.armor
        <= side
            .board
            .iter()
            .filter(|minion| minion.can_attack())
            .map(|minion| minion.attack)
            .sum::<i32>();
    if lethal && targets.contains(&Target::Hero(foe)) {
        return Some(Action::Attack {
            attacker: attacker.id,
            target: Target::Hero(foe),
        });
    }

    let mut best_trade: Option<(i32, Target)> = None;
    for target in &targets {
        let Target::Minion(_, id) = target else {
            continue;
        };
        let Some(defender) = foe_side.minion(*id) else {
            continue;
        };
        let kills = (defender.health <= attacker.attack || attacker.venom) && !defender.shield;
        let survives = defender.attack < attacker.health && !defender.venom;
        let score = if kills && survives {
            30 + defender.attack * 3 + defender.health
        } else if kills && defender.attack + defender.health > attacker.attack + attacker.health {
            15 + defender.attack * 2
        } else if defender.guard {
            5
        } else {
            -10
        };
        if best_trade.is_none_or(|(best, _)| score > best) {
            best_trade = Some((score, *target));
        }
    }

    let face_allowed = targets.contains(&Target::Hero(foe));
    match best_trade {
        Some((score, target)) if score >= 15 || !face_allowed => Some(Action::Attack {
            attacker: attacker.id,
            target,
        }),
        Some((score, target)) if score >= 5 && !side.aggressive => Some(Action::Attack {
            attacker: attacker.id,
            target,
        }),
        _ if face_allowed => Some(Action::Attack {
            attacker: attacker.id,
            target: Target::Hero(foe),
        }),
        Some((_, target)) => Some(Action::Attack {
            attacker: attacker.id,
            target,
        }),
        None => None,
    }
}

fn power_action(battle: &Battle, who: Who) -> Option<Action> {
    let side = battle.side(who);
    let foe = crate::data::battle::other(who);
    match side.power {
        HeroPower::Spark => {
            let weak = battle
                .side(foe)
                .board
                .iter()
                .find(|minion| minion.health == 1 && !minion.shield);
            let target = match weak {
                Some(minion) => Target::Minion(foe, minion.id),
                None => Target::Hero(foe),
            };
            Some(Action::Power {
                target: Some(target),
            })
        }
        HeroPower::Mend => {
            if side.hero_health < side.hero_max {
                Some(Action::Power {
                    target: Some(Target::Hero(who)),
                })
            } else {
                None
            }
        }
        HeroPower::Sprout | HeroPower::Bolt => {
            if power_rule(side.power) == TargetRule::None {
                Some(Action::Power { target: None })
            } else {
                None
            }
        }
    }
}
