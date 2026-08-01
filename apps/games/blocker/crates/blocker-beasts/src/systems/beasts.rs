use crate::commands::spawn_beast;
use crate::components::Beast;
use crate::data::beasts::BEASTS;
use crate::data::settings::Settings;
use crate::theme;
use blocker_blocks::queries::walkable;
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::{Eye, Plate};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_quests::commands::reward;
use blocker_quests::resources::Journal;
use blocker_screen::data::notice::Notice;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_health::{Health, queries::fallen};
use nightshade_random::Random;
use nightshade_random::commands::between;
use nightshade_scene::prelude::LocalTransform;

pub mod fight;
pub mod plates;
pub mod roam;

fn pick_kind(random: &mut Random) -> usize {
    let total: u32 = BEASTS.iter().map(|beast| beast.weight).sum::<u32>().max(1);
    let mut roll = random.u32(0..total);
    for (index, beast) in BEASTS.iter().enumerate() {
        if roll < beast.weight {
            return index;
        }
        roll -= beast.weight;
    }
    0
}

pub fn stir_the_beasts(
    world: Res<VoxelWorld>,
    config: Res<Config<Settings>>,
    hero: Res<Hero>,
    mut random: ResMut<Random>,
    mut commands: Commands,
    beasts: Query<(&Beast,)>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(place,)| place.0) else {
        return;
    };
    let mut count = 0usize;
    beasts.for_each(|_, _| count += 1);
    if count >= theme::BEAST_CAP {
        return;
    }
    let angle = between(&mut random, 0.0, std::f32::consts::TAU);
    let near = config.number(Settings::Near);
    let far = config.number(Settings::Far).max(near);
    let distance = between(&mut random, near, far);
    let x = (eye.x + angle.cos() * distance).floor() as i32;
    let z = (eye.z + angle.sin() * distance).floor() as i32;
    let Some(ground) = walkable(&world, x, z) else {
        return;
    };
    let kind = pick_kind(&mut random);
    let spot = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
    let heading = between(&mut random, 0.0, std::f32::consts::TAU);
    spawn_beast(
        &mut commands,
        Vec3::new(spot.x, ground as f32 + 1.0, spot.y),
        heading,
        kind,
        spot,
    );
}

pub fn reap_the_beasts(
    hero: Res<Hero>,
    mut journal: ResMut<Journal>,
    mut notices: EventWriter<Notice>,
    mut commands: Commands,
    beasts: Query<(&Beast, &Health, &LocalTransform, Option<&Plate>)>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(place,)| place.0) else {
        return;
    };
    let player_ground = Vec2::new(eye.x, eye.z);
    let mut gone: Vec<Entity> = Vec::new();
    let mut slain: Vec<usize> = Vec::new();
    beasts.for_each(|entity, (beast, health, transform, plate)| {
        let ground = Vec2::new(transform.translation.x, transform.translation.z);
        let leaving = fallen(health) || (ground - player_ground).magnitude() > theme::BEAST_CULL;
        if !leaving {
            return;
        }
        if fallen(health) {
            slain.push(beast.0);
        }
        gone.push(entity);
        if let Some(plate) = plate {
            gone.push(plate.0);
        }
    });
    for kind in slain {
        let renown = BEASTS[kind].renown;
        journal.record_slain(kind);
        reward(
            &mut journal,
            &mut notices,
            renown,
            renown,
            renown,
            format!("You slew a {} (+{} xp)", BEASTS[kind].name, renown),
        );
    }
    for entity in gone {
        commands.despawn_recursive(entity);
    }
}
