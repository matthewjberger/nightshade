use crate::commands::{award, order_the_round, roll_below, show_toast};
use crate::components::{Foe, Home, Hurt, Lunge, Slot};
use crate::contract::{BattleCommand, Spell};
use crate::data;
use crate::queries::{
    actor_spot, foe_spot, foes_standing, hero_spot, heroes_standing, living_foe, neediest_hero,
    stage_center,
};
use crate::resources::{Actor, Battle, Party, Phase, Rig, Screen, ScreenRequest, Toast};
use crate::tags::StagePiece;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub type Players<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Foe,
        &'item Slot,
        &'item mut Hurt,
        &'item Home,
        &'item mut Lunge,
        &'item mut Sprite,
        &'item mut LocalTransform,
    ),
>;
pub type Leftovers<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item LocalTransform,), WithTag<StagePiece>>;

const STAGE_COLUMNS: u32 = 34;
const STAGE_ROWS: u32 = 24;
const STAGE_WALL_LINE: f32 = 3.2;

pub fn stage_the_battle(
    rig: Res<Rig>,
    battle: Res<Battle>,
    party: Res<Party>,
    mut commands: Commands,
) {
    lay_the_floor(&mut commands, &rig);
    for (slot, foe) in battle.foes.iter().enumerate() {
        let spot = foe_spot(slot, battle.foes.len());
        let tile = data::monster_stats(foe.kind).tile;
        spawn_actor(&mut commands, &rig, tile, spot, true, slot, true);
    }
    for (slot, _) in party.heroes.iter().enumerate() {
        let spot = hero_spot(slot, party.heroes.len());
        let tile = theme::HERO_TILES[slot % theme::HERO_TILES.len()];
        spawn_actor(&mut commands, &rig, tile, spot, false, slot, false);
    }
}

fn lay_the_floor(commands: &mut Commands, rig: &Rig) {
    let mut ground = Tilemap::new(theme::ATLAS, STAGE_COLUMNS, STAGE_ROWS, theme::TILE_SIZE)
        .with_sheet(rig.sheets.dungeon, theme::SHEET_COLUMNS, theme::SHEET_ROWS)
        .with_layer(theme::LAYER_GROUND)
        .with_color(theme::VAULT_TINT);
    let middle = STAGE_ROWS as f32 / 2.0;
    for row in 0..STAGE_ROWS {
        let height = row as f32 + 0.5 - middle;
        for column in 0..STAGE_COLUMNS {
            let tile = if height >= STAGE_WALL_LINE {
                theme::VAULT_WALL
            } else {
                theme::VAULT_FLOOR[((column + row) % 3) as usize]
            };
            ground.set_tile(column, row, Some(TileData::new(tile)));
        }
    }
    let corner = Vec3::new(
        theme::STAGE_ORIGIN.x - STAGE_COLUMNS as f32 / 2.0,
        theme::STAGE_ORIGIN.y - middle,
        0.0,
    );
    let entity = spawn_tilemap(commands, ground, corner);
    commands.add_tag_type::<StagePiece>(entity);
}

fn spawn_actor(
    commands: &mut Commands,
    rig: &Rig,
    tile: u32,
    spot: Vec2,
    foe: bool,
    slot: usize,
    face_left: bool,
) {
    let shade = Sprite::new(theme::ATLAS, rig.sheets.shade)
        .with_size(1.15, 0.42)
        .with_layer(theme::LAYER_SHADE);
    let under = spawn_sprite(commands, shade, Vec3::new(spot.x, spot.y - 0.6, 0.0));
    commands.add_tag_type::<StagePiece>(under);

    let sprite = Sprite::new(theme::ATLAS, rig.sheets.dungeon_tile(tile))
        .with_size(1.3, 1.3)
        .with_layer(theme::LAYER_ACTOR)
        .with_flip(!face_left, false);
    let entity = spawn_sprite(commands, sprite, Vec3::new(spot.x, spot.y, 0.0));
    commands.set(entity, Foe(foe));
    commands.set(entity, Slot(slot));
    commands.set(entity, Hurt(0.0));
    commands.set(entity, Home(spot));
    commands.set(entity, Lunge(0.0));
    commands.add_tag_type::<StagePiece>(entity);
}

pub fn clear_the_stage(mut commands: Commands, mut leftovers: Leftovers<'_, '_>) {
    let mut stale = Vec::new();
    leftovers.for_each_mut(|entity, _| stale.push(entity));
    commands.despawn_batch(stale);
}

pub fn run_the_battle(
    time: Res<Time>,
    mut battle: ResMut<Battle>,
    mut party: ResMut<Party>,
    mut rig: ResMut<Rig>,
    mut toast: ResMut<Toast>,
    mut request: ResMut<ScreenRequest>,
) {
    battle.timer = (battle.timer - time.delta_time).max(0.0);
    match battle.phase {
        Phase::Intro => {
            if battle.timer <= 0.0 {
                battle.step = 0;
                begin_turn(&mut battle, &mut party, &mut rig);
            }
        }
        Phase::Choosing => {
            if let Some(command) = battle.command.take() {
                resolve_hero(&mut battle, &mut party, &mut rig, command);
                settle(&mut battle, &party);
            }
        }
        Phase::Acting => {
            if battle.timer <= 0.0 {
                step_forward(&mut battle, &mut party, &mut rig);
            }
        }
        Phase::Won | Phase::Fled => {
            if battle.timer <= 0.0 {
                if battle.phase == Phase::Won {
                    let gold = battle.gold;
                    let experience = battle.experience;
                    let notes = award(&mut party, gold, experience);
                    let mut line = format!("{gold} gold and {experience} experience.");
                    if let Some(first) = notes.first() {
                        line.push(' ');
                        line.push_str(first);
                    }
                    show_toast(&mut toast, line);
                } else {
                    show_toast(&mut toast, "The party breaks away.");
                }
                request.wanted = Some(Screen::Field);
            }
        }
        Phase::Lost => {
            if battle.timer <= 0.0 {
                request.wanted = Some(Screen::Defeat);
            }
        }
    }
}

fn begin_turn(battle: &mut Battle, party: &mut Party, rig: &mut Rig) {
    match battle.order.get(battle.step).copied() {
        Some(Actor::Hero(index)) => {
            if party.heroes.get(index).is_some_and(|hero| hero.alive()) {
                battle.phase = Phase::Choosing;
                battle.strike = Some((false, index));
            } else {
                step_forward(battle, party, rig);
            }
        }
        Some(Actor::Foe(index)) => {
            if battle.foes.get(index).is_some_and(|foe| foe.alive()) {
                resolve_foe(battle, party, rig, index);
                settle(battle, party);
            } else {
                step_forward(battle, party, rig);
            }
        }
        None => {
            battle.round += 1;
            order_the_round(battle, party);
            if battle.order.is_empty() {
                battle.phase = Phase::Lost;
                battle.timer = theme::BEAT_CLOSE;
                return;
            }
            begin_turn(battle, party, rig);
        }
    }
}

fn step_forward(battle: &mut Battle, party: &mut Party, rig: &mut Rig) {
    if battle.phase == Phase::Won || battle.phase == Phase::Lost || battle.phase == Phase::Fled {
        return;
    }
    battle.step += 1;
    begin_turn(battle, party, rig);
}

fn settle(battle: &mut Battle, party: &Party) {
    if foes_standing(battle) == 0 {
        battle.phase = Phase::Won;
        battle.timer = theme::BEAT_CLOSE;
        battle.gold = battle
            .foes
            .iter()
            .map(|foe| data::monster_stats(foe.kind).gold)
            .sum();
        battle.experience = battle
            .foes
            .iter()
            .map(|foe| data::monster_stats(foe.kind).experience)
            .sum();
        battle.log = "The way is clear.".to_string();
        return;
    }
    if heroes_standing(party) == 0 {
        battle.phase = Phase::Lost;
        battle.timer = theme::BEAT_CLOSE;
        battle.log = "The party falls.".to_string();
        return;
    }
    battle.phase = Phase::Acting;
    battle.timer = theme::BEAT_ACTION;
}

fn jitter(rig: &mut Rig) -> i32 {
    roll_below(rig, 5) as i32 - 2
}

fn resolve_hero(battle: &mut Battle, party: &mut Party, rig: &mut Rig, command: BattleCommand) {
    let Some(index) = battle.current_hero() else {
        return;
    };
    let name = party.heroes[index].name;
    battle.strike = Some((false, index));
    match command {
        BattleCommand::Attack => {
            let Some(target) = living_foe(battle) else {
                return;
            };
            let guard = data::monster_stats(battle.foes[target].kind).guard;
            let power = party.heroes[index].power() + party.blade_level as i32 * 3;
            let damage = (power - guard + jitter(rig)).max(1);
            battle.foes[target].health -= damage;
            battle.flash = Some((true, target));
            rig.shake = rig.shake.max(0.16);
            battle.log = format!(
                "{name} strikes the {} for {damage}.",
                battle.foes[target].kind.name()
            );
        }
        BattleCommand::Magic => {
            let Some(spell) = party.heroes[index].job.spell() else {
                battle.log = format!("{name} knows no magic.");
                return;
            };
            if party.heroes[index].magic < spell.cost() as i32 {
                battle.log = format!("{name} has no power left for {}.", spell.name());
                return;
            }
            party.heroes[index].magic -= spell.cost() as i32;
            match spell {
                Spell::Cure => {
                    let Some(target) = neediest_hero(party) else {
                        return;
                    };
                    let level = party.heroes[index].level as i32;
                    let mended = 16 + level * 3;
                    let hero = &mut party.heroes[target];
                    hero.health = (hero.health + mended).min(hero.max_health);
                    let mended_name = hero.name;
                    battle.flash = Some((false, target));
                    battle.log = format!("{name} mends {mended_name} for {mended}.");
                }
                Spell::Fire => {
                    let Some(target) = living_foe(battle) else {
                        return;
                    };
                    let level = party.heroes[index].level as i32;
                    let damage = 14 + level * 3 + jitter(rig);
                    battle.foes[target].health -= damage;
                    battle.flash = Some((true, target));
                    rig.shake = rig.shake.max(0.24);
                    battle.log = format!(
                        "Fire takes the {} for {damage}.",
                        battle.foes[target].kind.name()
                    );
                }
            }
        }
        BattleCommand::Item => {
            if party.potions == 0 {
                battle.log = "The pack is empty.".to_string();
                return;
            }
            let Some(target) = neediest_hero(party) else {
                return;
            };
            party.potions -= 1;
            let hero = &mut party.heroes[target];
            hero.health = (hero.health + 40).min(hero.max_health);
            let mended_name = hero.name;
            battle.flash = Some((false, target));
            battle.log = format!("{name} gives {mended_name} a potion.");
        }
        BattleCommand::Run => {
            let speed = party.heroes[index].speed();
            if roll_below(rig, 100) < 45 + speed as u32 * 2 {
                battle.phase = Phase::Fled;
                battle.timer = theme::BEAT_ACTION;
                battle.log = "The party slips away.".to_string();
            } else {
                battle.log = "There is no way past them.".to_string();
            }
        }
    }
}

fn resolve_foe(battle: &mut Battle, party: &mut Party, rig: &mut Rig, index: usize) {
    let standing: Vec<usize> = party
        .heroes
        .iter()
        .enumerate()
        .filter(|(_, hero)| hero.alive())
        .map(|(slot, _)| slot)
        .collect();
    if standing.is_empty() {
        return;
    }
    let pick = roll_below(rig, standing.len() as u32) as usize;
    let target = standing[pick];
    let stats = data::monster_stats(battle.foes[index].kind);
    let guard = party.heroes[target].guard() + party.guard_level as i32 * 2;
    let damage = (stats.power - guard + jitter(rig)).max(1);
    let hero = &mut party.heroes[target];
    hero.health = (hero.health - damage).max(0);
    let hero_name = hero.name;
    let down = !hero.alive();
    battle.strike = Some((true, index));
    battle.flash = Some((false, target));
    rig.shake = rig.shake.max(0.2);
    battle.log = if down {
        format!("The {} fells {hero_name}.", battle.foes[index].kind.name())
    } else {
        format!(
            "The {} hits {hero_name} for {damage}.",
            battle.foes[index].kind.name()
        )
    };
}

pub fn frame_the_battle(battle: Res<Battle>, party: Res<Party>, mut rig: ResMut<Rig>) {
    rig.snap = false;
    rig.bounds = None;
    let center = stage_center();
    let (focus, zoom) = match battle.phase {
        Phase::Intro => (center, 1.70),
        Phase::Choosing => match battle.current_hero() {
            Some(index) => {
                let spot = hero_spot(index, party.heroes.len());
                (center + (spot - center) * 0.55, 2.05)
            }
            None => (center, 1.85),
        },
        Phase::Acting => match battle.flash {
            Some((foe, slot)) => {
                let spot = actor_spot(&battle, &party, foe, slot);
                (center + (spot - center) * 0.72, 2.55)
            }
            None => (center, 2.0),
        },
        Phase::Won | Phase::Fled => (center, 1.55),
        Phase::Lost => (center, 2.25),
    };
    rig.focus = focus;
    rig.zoom = rig.zoom + (zoom - rig.zoom) * 0.12;
}

pub fn dress_the_stage(
    time: Res<Time>,
    mut battle: ResMut<Battle>,
    party: Res<Party>,
    mut players: Players<'_, '_>,
) {
    let delta_time = time.delta_time;
    let flash = battle.flash;
    let strike = battle.strike.take();
    players.for_each_mut(
        |_entity, (is_foe, slot, hurt, home, lunge, sprite, transform)| {
            if flash == Some((is_foe.0, slot.0)) && hurt.0 <= 0.0 {
                hurt.0 = 0.26;
            }
            if strike == Some((is_foe.0, slot.0)) {
                lunge.0 = 1.0;
            }

            hurt.0 = (hurt.0 - delta_time).max(0.0);
            lunge.0 = (lunge.0 - delta_time * 2.6).max(0.0);

            let standing = if is_foe.0 {
                battle.foes.get(slot.0).is_some_and(|foe| foe.alive())
            } else {
                party.heroes.get(slot.0).is_some_and(|hero| hero.alive())
            };

            let mut color = if hurt.0 > 0.0 {
                theme::HURT_TINT
            } else {
                theme::ACTOR_TINT
            };
            if !standing {
                color[3] = 0.22;
            }
            sprite.color = color;

            let lean = if is_foe.0 { 1.0 } else { -1.0 };
            let reach = (lunge.0 * std::f32::consts::PI).sin() * 0.55;
            transform.translation.x = home.0.x + lean * reach;
            transform.translation.y = home.0.y + hurt.0 * 0.35;
        },
    );
    if battle.phase != crate::resources::Phase::Acting {
        battle.flash = None;
    }
}
