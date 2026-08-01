use crate::contract::{Job, MonsterKind, Place, Ware};
use crate::data;
use crate::resources::{Battle, Field, Foe, Party, Phase, Rig, Toast};
use crate::theme;
use nalgebra_glm::Vec2;

pub fn roll_below(rig: &mut Rig, limit: u32) -> u32 {
    if limit == 0 {
        return 0;
    }
    rig.rng.u32(..limit)
}

pub fn show_toast(toast: &mut Toast, text: impl Into<String>) {
    toast.text = text.into();
    toast.life = theme::TOAST_LIFE;
}

pub fn travel(field: &mut Field, place: Place, arrive: Option<Vec2>) {
    field.pending = Some((place, arrive));
}

pub fn rest(party: &mut Party) -> bool {
    if party.gold < theme::INN_PRICE {
        return false;
    }
    party.gold -= theme::INN_PRICE;
    for hero in &mut party.heroes {
        hero.health = hero.max_health;
        hero.magic = hero.max_magic;
    }
    true
}

pub fn buy(party: &mut Party, ware: Ware) -> Result<String, String> {
    if party.gold < ware.price() {
        return Err("Not enough gold.".to_string());
    }
    match ware {
        Ware::Potion => {
            party.gold -= ware.price();
            party.potions += 1;
            Ok(format!("Bought a potion. You carry {}.", party.potions))
        }
        Ware::Sword => {
            if party.blade_level >= theme::MAX_UPGRADE {
                return Err("The blades are as keen as they get.".to_string());
            }
            party.gold -= ware.price();
            party.blade_level += 1;
            Ok(format!(
                "The blades bite deeper. Blade {}.",
                party.blade_level
            ))
        }
        Ware::Charm => {
            if party.guard_level >= theme::MAX_UPGRADE {
                return Err("The guards are as strong as they get.".to_string());
            }
            party.gold -= ware.price();
            party.guard_level += 1;
            Ok(format!(
                "The guards hold harder. Guard {}.",
                party.guard_level
            ))
        }
    }
}

pub fn take_shard(party: &mut Party) -> String {
    party.shards += 1;
    format!("A crystal shard. You hold {}.", party.shards)
}

pub fn unlock_job(party: &mut Party) -> Result<String, String> {
    if party.shards == 0 {
        return Err("The crystal is dark. Bring it a shard.".to_string());
    }
    let Some(job) = Job::LADDER
        .iter()
        .copied()
        .find(|job| !party.unlocked.contains(job))
    else {
        return Err("The crystal has nothing left to give.".to_string());
    };
    party.shards -= 1;
    party.unlocked.push(job);
    Ok(format!("The crystal opens the way of the {}.", job.name()))
}

pub fn assign_job(party: &mut Party, index: usize, job: Job) -> String {
    let stats = data::job_stats(job);
    let Some(hero) = party.heroes.get_mut(index) else {
        return String::new();
    };
    let level = hero.level as i32;
    hero.job = job;
    hero.max_health = stats.health + (level - 1) * 6;
    hero.max_magic = stats.magic + (level - 1) * 3;
    hero.health = hero.health.min(hero.max_health).max(1);
    hero.magic = hero.magic.min(hero.max_magic);
    format!("{} takes up the {}.", hero.name, job.name())
}

pub fn award(party: &mut Party, gold: u32, experience: u32) -> Vec<String> {
    party.gold += gold;
    let mut notes = Vec::new();
    for hero in &mut party.heroes {
        if !hero.alive() {
            continue;
        }
        hero.experience += experience;
        while hero.experience >= hero.level * theme::LEVEL_STEP {
            hero.experience -= hero.level * theme::LEVEL_STEP;
            hero.level += 1;
            hero.max_health += 6;
            hero.max_magic += 3;
            hero.health = hero.max_health;
            hero.magic = hero.max_magic;
            notes.push(format!("{} reaches level {}.", hero.name, hero.level));
        }
    }
    notes
}

pub fn revive(party: &mut Party) {
    for hero in &mut party.heroes {
        hero.health = hero.max_health.max(1);
        hero.magic = hero.max_magic;
    }
}

pub fn open_battle(battle: &mut Battle, rig: &mut Rig, party: &Party) {
    let count = 1 + roll_below(rig, 3) as usize;
    let mut foes = Vec::new();
    for _ in 0..count {
        let pick = roll_below(rig, MonsterKind::VAULT.len() as u32) as usize;
        let kind = MonsterKind::VAULT[pick];
        let stats = data::monster_stats(kind);
        foes.push(Foe {
            kind,
            health: stats.health,
            max_health: stats.health,
        });
    }
    let names = foes
        .iter()
        .map(|foe| foe.kind.name())
        .collect::<Vec<_>>()
        .join(", ");
    *battle = Battle {
        foes,
        log: format!("{names} block the way."),
        phase: Phase::Intro,
        timer: theme::BEAT_INTRO,
        ..Default::default()
    };
    order_the_round(battle, party);
}

pub fn order_the_round(battle: &mut Battle, party: &Party) {
    let mut ranked: Vec<(i32, crate::resources::Actor)> = Vec::new();
    for (index, hero) in party.heroes.iter().enumerate() {
        if hero.alive() {
            ranked.push((hero.speed(), crate::resources::Actor::Hero(index)));
        }
    }
    for (index, foe) in battle.foes.iter().enumerate() {
        if foe.alive() {
            ranked.push((
                data::monster_stats(foe.kind).speed,
                crate::resources::Actor::Foe(index),
            ));
        }
    }
    ranked.sort_by_key(|(speed, _)| -speed);
    battle.order = ranked.into_iter().map(|(_, actor)| actor).collect();
    battle.step = 0;
}
