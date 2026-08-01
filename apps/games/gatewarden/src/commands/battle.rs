use crate::data::battle::{
    ARMOR_CAP, BOARD_LIMIT, Battle, Event, HAND_LIMIT, Instance, MAX_MANA, Minion, OPENING_HAND,
    SECOND_PLAYER_BONUS, Side, Target, Who,
};
use crate::data::cards::{CardId, Element, Kind, Spell, TargetRule};
use crate::data::decks::{HeroPower, WardenDef, starting_deck, warden, warden_deck};
use crate::data::gates::{
    ASHFALL_DAMAGE, ECLIPSE_SURCHARGE, GateRule, STONEWALL_ARMOR, THORNS_DAMAGE, gate_rule,
};
use crate::data::random::random_below;
use crate::data::relics::Relic;
use crate::data::story::{Shard, warden_shard};
use crate::queries::battle::{
    attack_targets, can_play, can_use_power, card_cost, power_rule, targets_for,
};

pub struct BattleSetup<'a> {
    pub gate: u32,
    pub pick: u8,
    pub deck: &'a [CardId],
    pub forged: &'a [CardId],
    pub health: Option<i32>,
    pub relics: &'a [Relic],
    pub shards: &'a [Shard],
    pub borrowed_bolt: bool,
    pub warden_bonus: i32,
    pub seed: u64,
}

pub fn new_battle(setup: BattleSetup<'_>) -> Battle {
    let mut battle = Battle {
        seed: setup.seed,
        ..Battle::default()
    };
    let warden_def: WardenDef = warden(setup.gate, setup.pick);

    let charm = if setup.relics.contains(&Relic::WardingCharm) {
        5
    } else {
        0
    };
    let hide = if setup.relics.contains(&Relic::ThickHide) {
        8
    } else {
        0
    };
    let hero_bonus = charm + hide;
    let deep_bonus = if setup.shards.contains(&Shard::Deep) {
        5
    } else {
        0
    };
    let hero_max = 30 + hero_bonus + deep_bonus;
    let player_health = setup.health.unwrap_or(hero_max).clamp(1, hero_max);
    let power = if setup.borrowed_bolt {
        HeroPower::Bolt
    } else {
        HeroPower::Spark
    };
    battle.rule = gate_rule(setup.gate);
    battle.player = Side {
        name: "You".to_string(),
        hero_health: player_health,
        hero_max,
        power,
        power_cost: if setup.relics.contains(&Relic::SparkFocus) {
            1
        } else {
            power.cost()
        },
        minion_health_bonus: if setup.relics.contains(&Relic::BattleStandard) {
            1
        } else {
            0
        },
        minion_attack_bonus: i32::from(setup.relics.contains(&Relic::Whetstone)),
        guard_health_bonus: if setup.relics.contains(&Relic::StoneCollar) {
            2
        } else {
            0
        },
        turn_heal: if setup.relics.contains(&Relic::TideFlask) {
            2
        } else {
            0
        },
        turn_burn: i32::from(setup.relics.contains(&Relic::KilnCoal)),
        rush_all: setup.relics.contains(&Relic::QuickBoots),
        spell_bonus: i32::from(setup.shards.contains(&Shard::Storm))
            + i32::from(setup.relics.contains(&Relic::GlassLens)),
        shards: setup.shards.to_vec(),
        ..Side::default()
    };
    let held: Vec<Shard> = warden_shard(setup.gate).into_iter().collect();
    let held_deep = if held.contains(&Shard::Deep) { 5 } else { 0 };
    let warden_health = warden_def.health + setup.warden_bonus + held_deep;
    battle.enemy = Side {
        name: warden_def.name.to_string(),
        hero_health: warden_health,
        hero_max: warden_health,
        power: warden_def.power,
        power_cost: warden_def.power.cost(),
        aggressive: warden_def.aggressive,
        spell_bonus: i32::from(held.contains(&Shard::Storm)),
        shards: held,
        ..Side::default()
    };

    let player_cards = if setup.deck.is_empty() {
        starting_deck()
    } else {
        setup.deck.to_vec()
    };
    if battle.rule == GateRule::Stonewall {
        battle.enemy.armor = STONEWALL_ARMOR;
    }
    if battle.rule == GateRule::Eclipse {
        battle.player.power_cost += ECLIPSE_SURCHARGE;
    }
    fill_deck(&mut battle, Who::Player, &player_cards, setup.forged);
    fill_deck(
        &mut battle,
        Who::Enemy,
        &warden_deck(setup.gate, warden_def.element),
        &[],
    );

    let extra = usize::from(setup.relics.contains(&Relic::ScholarsQuill))
        + usize::from(setup.shards.contains(&Shard::Elder));
    for _ in 0..OPENING_HAND + extra {
        draw(&mut battle, Who::Player);
    }
    let warden_extra = usize::from(battle.enemy.shards.contains(&Shard::Elder));
    for _ in 0..OPENING_HAND + SECOND_PLAYER_BONUS + warden_extra {
        draw(&mut battle, Who::Enemy);
    }
    if setup.relics.contains(&Relic::DeepWell) {
        battle.player.max_mana = 1;
    }
    battle.events.clear();
    battle.active = Who::Enemy;
    begin_turn(&mut battle, Who::Player);
    battle
}

fn fill_deck(battle: &mut Battle, who: Who, cards: &[CardId], forged: &[CardId]) {
    let mut pending: Vec<CardId> = forged.to_vec();
    let mut instances: Vec<Instance> = cards
        .iter()
        .map(|card| {
            let upgraded = match pending.iter().position(|held| held == card) {
                Some(index) => {
                    pending.remove(index);
                    true
                }
                None => false,
            };
            Instance {
                id: battle.allocate(),
                card: *card,
                upgraded,
            }
        })
        .collect();
    for index in (1..instances.len()).rev() {
        let other = random_below(&mut battle.seed, index + 1);
        instances.swap(index, other);
    }
    battle.side_mut(who).deck = instances;
}

pub fn begin_turn(battle: &mut Battle, who: Who) {
    if who == Who::Player {
        battle.turn += 1;
    }
    battle.active = who;
    let turn = battle.turn;
    battle.played_this_turn = 0;
    let side = battle.side_mut(who);
    side.max_mana = (side.max_mana + 1).min(MAX_MANA);
    side.mana = side.max_mana;
    side.power_used = false;
    side.turn_element = None;
    for minion in side.board.iter_mut() {
        minion.ready = !minion.frozen;
        minion.attacked = false;
        minion.frozen = false;
    }
    let turn_heal = side.turn_heal;
    let turn_burn = side.turn_burn;
    battle.events.push_back(Event::Turn { who, turn });
    draw(battle, who);
    if battle.rule == GateRule::Riptide && who == Who::Enemy {
        draw(battle, who);
    }
    if battle.side(who).shards.contains(&Shard::Tide) && turn % 2 == 1 {
        heal(battle, Target::Hero(who), 1);
    }
    if turn_heal > 0 {
        heal(battle, Target::Hero(who), turn_heal);
    }
    if turn_burn > 0 {
        damage(
            battle,
            Target::Hero(crate::data::battle::other(who)),
            turn_burn,
        );
    }
}

pub fn end_turn(battle: &mut Battle) {
    if battle.rule == GateRule::Ashfall {
        damage(battle, Target::Hero(Who::Player), ASHFALL_DAMAGE);
        damage(battle, Target::Hero(Who::Enemy), ASHFALL_DAMAGE);
    }
    if battle.outcome.is_some() {
        return;
    }
    let next = crate::data::battle::other(battle.active);
    begin_turn(battle, next);
}

pub fn draw(battle: &mut Battle, who: Who) {
    let side = battle.side_mut(who);
    match side.deck.pop() {
        Some(instance) => {
            if side.hand.len() >= HAND_LIMIT {
                battle.events.push_back(Event::Note(format!(
                    "{} burns {}",
                    side_label(who),
                    instance.card.def().name
                )));
            } else {
                side.hand.push(instance);
                battle.events.push_back(Event::Draw { who });
            }
        }
        None => {
            side.fatigue += 1;
            let amount = side.fatigue;
            battle.events.push_back(Event::Fatigue { who, amount });
            damage(battle, Target::Hero(who), amount);
        }
    }
}

fn side_label(who: Who) -> &'static str {
    match who {
        Who::Player => "You",
        Who::Enemy => "The warden",
    }
}

pub fn play_card(
    battle: &mut Battle,
    who: Who,
    id: u32,
    target: Option<Target>,
) -> Result<(), &'static str> {
    if battle.active != who || battle.outcome.is_some() {
        return Err("not your turn");
    }
    if !can_play(battle, who, id) {
        return Err("cannot play that card");
    }
    let instance = battle
        .side(who)
        .hand_card(id)
        .ok_or("card is not in hand")?;
    let def = instance.card.def();
    if let Kind::Spell { target: rule, .. } = def.kind
        && rule != TargetRule::None
    {
        let Some(chosen) = target else {
            return Err("that spell needs a target");
        };
        if !targets_for(battle, who, rule).contains(&chosen) {
            return Err("not a valid target");
        }
    }

    let cost = card_cost(battle, who, instance);
    let element = def.element;
    let chained = element != Element::Neutral && battle.side(who).turn_element == Some(element);
    let side = battle.side_mut(who);
    side.hand.retain(|held| held.id != id);
    side.mana -= cost;
    side.turn_element = Some(element);
    battle.played_this_turn += 1;
    battle.events.push_back(Event::Play {
        who,
        card: instance.card,
    });
    if chained {
        battle.events.push_back(Event::Chain { who, element });
    }
    let forged = instance.upgraded && instance.card.forge_number();
    let bonus = i32::from(chained) + i32::from(forged);

    match def.kind {
        Kind::Minion { .. } => {
            summon(battle, who, instance.card, id, bonus);
        }
        Kind::Spell { effect, .. } => cast(battle, who, effect, target, bonus),
    }
    resolve_deaths(battle);
    Ok(())
}

pub fn summon(battle: &mut Battle, who: Who, card: CardId, origin: u32, bonus: i32) {
    let Kind::Minion {
        attack,
        health,
        guard,
        rush,
        shield,
        venom,
        drain,
        warcry,
    } = card.def().kind
    else {
        return;
    };
    if battle.side(who).board.len() >= BOARD_LIMIT {
        return;
    }
    let id = battle.allocate();
    let side = battle.side_mut(who);
    let mut health = health + side.minion_health_bonus + bonus;
    let mut attack = attack + side.minion_attack_bonus + bonus;
    let rushes = rush || side.rush_all;
    if guard {
        health += side.guard_health_bonus;
    }
    if side.shards.contains(&Shard::Ember)
        && card.def().element == crate::data::cards::Element::Ember
    {
        attack += 1;
    }
    if side.shards.contains(&Shard::Cinder) && rush {
        attack += 1;
    }
    if side.shards.contains(&Shard::Grove) && guard {
        health += 1;
    }
    side.board.push(Minion {
        id,
        origin,
        card,
        attack,
        health,
        max_health: health,
        guard,
        shield,
        venom,
        drain,
        frozen: false,
        ready: rushes,
        attacked: false,
    });
    battle.events.push_back(Event::Summon { who, id });
    if let Some(effect) = warcry {
        cast(battle, who, effect, None, bonus);
    }
}

fn cast(battle: &mut Battle, who: Who, effect: Spell, target: Option<Target>, bonus: i32) {
    match effect {
        Spell::Damage(amount) => {
            let amount = amount + battle.side(who).spell_bonus + bonus;
            if let Some(target) = target {
                damage(battle, target, amount);
            }
        }
        Spell::Heal(amount) => {
            if let Some(target) = target {
                heal(battle, target, amount + bonus);
            }
        }
        Spell::Buff(attack, health) => {
            if let Some(Target::Minion(side, id)) = target
                && let Some(minion) = battle.side_mut(side).minion_mut(id)
            {
                minion.attack += attack + bonus;
                minion.health += health + bonus;
                minion.max_health += health + bonus;
                battle.events.push_back(Event::Heal {
                    target: Target::Minion(side, id),
                    amount: health,
                });
            }
        }
        Spell::Storm(amount) => {
            let amount = amount + battle.side(who).spell_bonus + bonus;
            let foe = crate::data::battle::other(who);
            let ids: Vec<u32> = battle
                .side(foe)
                .board
                .iter()
                .map(|minion| minion.id)
                .collect();
            for id in ids {
                damage(battle, Target::Minion(foe, id), amount);
            }
        }
        Spell::Draw(count) => {
            for _ in 0..count + bonus.max(0) as u32 {
                draw(battle, who);
            }
        }
        Spell::Summon(card, count) => {
            for _ in 0..count + bonus.max(0) as u32 {
                let origin = battle.allocate();
                summon(battle, who, card, origin, 0);
            }
        }
        Spell::Armor(amount) => {
            let side = battle.side_mut(who);
            let before = side.armor;
            side.armor = (side.armor + amount + bonus).min(ARMOR_CAP);
            let gained = side.armor - before;
            if gained > 0 {
                battle.events.push_back(Event::Armor {
                    who,
                    amount: gained,
                });
            }
        }
        Spell::Freeze => {
            if let Some(Target::Minion(side, id)) = target
                && let Some(minion) = battle.side_mut(side).minion_mut(id)
            {
                minion.frozen = true;
                minion.ready = false;
                minion.attacked = true;
                battle.events.push_back(Event::Hit {
                    target: Target::Minion(side, id),
                    amount: 0,
                });
            }
        }
        Spell::Shield => {
            if let Some(Target::Minion(side, id)) = target
                && let Some(minion) = battle.side_mut(side).minion_mut(id)
            {
                minion.shield = true;
                battle.events.push_back(Event::Heal {
                    target: Target::Minion(side, id),
                    amount: 0,
                });
            }
        }
    }
}

pub fn attack(
    battle: &mut Battle,
    who: Who,
    attacker: u32,
    target: Target,
) -> Result<(), &'static str> {
    if battle.active != who || battle.outcome.is_some() {
        return Err("not your turn");
    }
    let Some(minion) = battle.side(who).minion(attacker).copied() else {
        return Err("no such minion");
    };
    if !minion.can_attack() {
        return Err("that minion cannot attack");
    }
    if !attack_targets(battle, who).contains(&target) {
        return Err("not a valid target");
    }
    if let Some(attacking) = battle.side_mut(who).minion_mut(attacker) {
        attacking.attacked = true;
    }
    battle.events.push_back(Event::Attack {
        attacker: Target::Minion(who, attacker),
        target,
    });
    let back = battle.minion(target).map_or(0, |defender| defender.attack);
    let venomous_defender = battle.minion(target).is_some_and(|defender| defender.venom);
    damage(battle, target, minion.attack);
    if minion.venom
        && let Target::Minion(side, id) = target
        && let Some(defender) = battle.side_mut(side).minion_mut(id)
    {
        defender.health = 0;
    }
    if minion.drain {
        heal(battle, Target::Hero(who), minion.attack);
    }
    if battle.rule == GateRule::Thorns {
        damage(battle, Target::Minion(who, attacker), THORNS_DAMAGE);
    }
    if back > 0 {
        damage(battle, Target::Minion(who, attacker), back);
        if venomous_defender && let Some(attacking) = battle.side_mut(who).minion_mut(attacker) {
            attacking.health = 0;
        }
    }
    resolve_deaths(battle);
    Ok(())
}

pub fn use_power(
    battle: &mut Battle,
    who: Who,
    target: Option<Target>,
) -> Result<(), &'static str> {
    if !can_use_power(battle, who) || battle.outcome.is_some() {
        return Err("cannot use the hero power");
    }
    let power = battle.side(who).power;
    let rule = power_rule(power);
    if rule != TargetRule::None {
        let Some(chosen) = target else {
            return Err("the hero power needs a target");
        };
        if !targets_for(battle, who, rule).contains(&chosen) {
            return Err("not a valid target");
        }
    }
    let turn = battle.turn;
    let side = battle.side_mut(who);
    side.power_used = true;
    side.power_turn = turn;
    side.mana -= side.power_cost;
    battle.events.push_back(Event::Note(format!(
        "{} uses {}",
        side_label(who),
        power.name()
    )));
    match power {
        HeroPower::Spark => {
            if let Some(target) = target {
                damage(battle, target, 1);
            }
        }
        HeroPower::Mend => {
            if let Some(target) = target {
                heal(battle, target, 1);
            }
        }
        HeroPower::Sprout => {
            let origin = battle.allocate();
            summon(battle, who, CardId::Sapling, origin, 0);
        }
        HeroPower::Bolt => {
            let foe = crate::data::battle::other(who);
            let mut targets = vec![Target::Hero(foe)];
            targets.extend(
                battle
                    .side(foe)
                    .board
                    .iter()
                    .map(|minion| Target::Minion(foe, minion.id)),
            );
            let pick = random_below(&mut battle.seed, targets.len());
            damage(battle, targets[pick], 1);
        }
    }
    resolve_deaths(battle);
    Ok(())
}

pub fn damage(battle: &mut Battle, target: Target, amount: i32) {
    if amount <= 0 {
        return;
    }
    match target {
        Target::Hero(who) => {
            let side = battle.side_mut(who);
            let held = amount.min(side.armor);
            side.armor -= held;
            let through = amount - held;
            side.hero_health -= through;
            battle.events.push_back(Event::Hit {
                target,
                amount: through,
            });
            if battle.side(who).hero_health <= 0 && battle.outcome.is_none() {
                battle.outcome = Some(crate::data::battle::other(who));
                battle.events.push_back(Event::Died { target, card: None });
            }
        }
        Target::Minion(who, id) => {
            let Some(minion) = battle.side_mut(who).minion_mut(id) else {
                return;
            };
            if minion.shield {
                minion.shield = false;
                battle.events.push_back(Event::Hit { target, amount: 0 });
                return;
            }
            minion.health -= amount;
            battle.events.push_back(Event::Hit { target, amount });
        }
    }
}

pub fn heal(battle: &mut Battle, target: Target, amount: i32) {
    let dry = battle.rule == GateRule::Drought
        && matches!(
            target,
            Target::Hero(Who::Player) | Target::Minion(Who::Player, _)
        );
    let amount = if dry && amount > 0 {
        (amount / 2).max(1)
    } else {
        amount
    };
    match target {
        Target::Hero(who) => {
            let side = battle.side_mut(who);
            let before = side.hero_health;
            side.hero_health = (side.hero_health + amount).min(side.hero_max);
            let healed = side.hero_health - before;
            battle.events.push_back(Event::Heal {
                target,
                amount: healed,
            });
        }
        Target::Minion(who, id) => {
            if let Some(minion) = battle.side_mut(who).minion_mut(id) {
                let before = minion.health;
                minion.health = (minion.health + amount).min(minion.max_health);
                let healed = minion.health - before;
                battle.events.push_back(Event::Heal {
                    target,
                    amount: healed,
                });
            }
        }
    }
}

pub fn resolve_deaths(battle: &mut Battle) {
    for who in [Who::Player, Who::Enemy] {
        let dead: Vec<(u32, CardId)> = battle
            .side(who)
            .board
            .iter()
            .filter(|minion| minion.health <= 0)
            .map(|minion| (minion.id, minion.card))
            .collect();
        for (id, card) in dead {
            battle.side_mut(who).board.retain(|minion| minion.id != id);
            battle.events.push_back(Event::Died {
                target: Target::Minion(who, id),
                card: Some(card),
            });
        }
    }
}
