use crate::components::{
    Cooldown, Drift, EnemyKind, EnemyMood, Group, Health, Phase, PickupKind, RestingHeight, Timer,
};
use crate::data;
use crate::data::arsenal::{CHAINGUN, GUNS, LAUNCHER, PLASMA};
use crate::data::bestiary::beast;
use crate::data::levels::plan;
use crate::data::loot;
use crate::resources::{BoomstickResources, CLIP_IDLE};
use crate::tags::{LevelPiece, Warden};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_animation::contract::{AnimationClip, AnimationPlayer};
use nightshade_prefab::{Prefab, PrefabSpawn};
use nightshade_scene::contract::LocalTransform;

pub fn models(
    mut state: ResMut<BoomstickResources>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    let ready = state.mission.built && !state.mission.populated && state.assets.ready();
    if !ready {
        return;
    }

    let level = state.mission.level;
    let plan = plan(level);

    let mut standing = 0;
    for (at, kind, group) in plan.monsters {
        raise(&state, parts.reborrow(), storage, *kind, *at, *group, false);
        standing += 1;
    }
    if let Some(at) = plan.warden {
        raise(
            &state,
            parts.reborrow(),
            storage,
            EnemyKind::Warden,
            at,
            1,
            true,
        );
        standing += 1;
    }

    state.mission.populated = true;
    state.mission.standing = standing;
}

pub fn loot(mut resources: ResMut<BoomstickResources>, mut commands: Commands) {
    if !resources.mission.built || resources.mission.armed {
        return;
    }
    resources.mission.armed = true;
    let plan = plan(resources.mission.level);
    for (at, kind) in plan.loot {
        let resting = at.y + data::PICKUP_LIFT;
        let entity = match loot::shape(*kind) {
            Some((name, parts)) => crate::commands::build_parts(
                &mut commands,
                &format!("boomstick_loot_{name}"),
                parts,
                Vec3::new(at.x, resting, at.z),
                data::LOOT_SCALE,
            ),
            None => {
                let Some(slot) = gun_slot(*kind) else {
                    continue;
                };
                crate::commands::build_parts(
                    &mut commands,
                    &format!("boomstick_gun_{slot}"),
                    GUNS[slot].parts,
                    Vec3::new(at.x, resting, at.z),
                    data::GUN_PICKUP_SCALE,
                )
            }
        };
        commands.set(entity, *kind);
        commands.set(entity, RestingHeight(resting));
        commands.set(entity, Phase(at.x + at.z));
        commands.add_tag_type::<LevelPiece>(entity);
    }
}

fn gun_slot(kind: PickupKind) -> Option<usize> {
    match kind {
        PickupKind::Chaingun => Some(CHAINGUN),
        PickupKind::Launcher => Some(LAUNCHER),
        PickupKind::Plasma => Some(PLASMA),
        _ => None,
    }
}

fn raise(
    state: &BoomstickResources,
    parts: nightshade_prefab::PrefabParts<'_>,
    storage: &mut Storage,
    kind: EnemyKind,
    at: Vec3,
    group: u32,
    warden: bool,
) {
    let picked = state
        .assets
        .beast(kind)
        .map(|(prefab, clips)| (prefab.clone(), clips.to_vec()));
    let Some((prefab, clips)) = picked else {
        return;
    };
    let profile = beast(kind);
    let scale = profile.height / data::MODEL_HEIGHT;
    let entity = animate(parts, storage, &prefab, &clips, at, scale);
    storage.set(entity, kind);
    storage.set(entity, EnemyMood::Asleep);
    storage.set(entity, Health(profile.health));
    storage.set(entity, Timer(0.0));
    storage.set(entity, Cooldown(0.0));
    storage.set(entity, Group(group));
    let side = if (at.x + at.z) as i32 % 2 == 0 {
        1.0
    } else {
        -1.0
    };
    storage.set(entity, Drift(side));
    storage.add_tag_type::<LevelPiece>(entity);
    if warden {
        storage.add_tag_type::<Warden>(entity);
    }
}

fn animate(
    parts: nightshade_prefab::PrefabParts<'_>,
    storage: &mut Storage,
    prefab: &Prefab,
    clips: &[AnimationClip],
    position: Vec3,
    scale: f32,
) -> Entity {
    let entity = {
        let mut spawn = PrefabSpawn { storage, parts };
        nightshade_prefab::spawn_prefab_with_skins(&mut spawn, prefab, clips, &[], position)
    };
    if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
        transform.translation = position;
        transform.scale = Vec3::new(scale, scale, scale);
    }
    if let Some(player) = storage.get_mut::<AnimationPlayer>(entity) {
        player.current_clip = Some(CLIP_IDLE);
        player.time = 0.0;
        player.speed = 1.0;
        player.looping = true;
        player.playing = true;
    }
    entity
}
