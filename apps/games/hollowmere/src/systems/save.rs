use crate::commands::{plant, toast_show};
use crate::components::{
    Block, Cooldowns, Experience, Health, Kills, Level, Mana, Progress, Stage, Watered,
};
use crate::data::{CropKind, GIVER_COUNT, PLACE_COUNT, QUEST_COUNT};
use crate::resources::{
    Chests, Generation, Hero, Inventory, Journal, Places, Player, Save, Saved, SavedCrop,
    SavedHero, Sheet, Toast, VoxelWorld,
};
use crate::theme;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_save::prelude::{SaveDue, save_read, save_write};

pub fn read_the_save(
    mut world: ResMut<VoxelWorld>,
    mut chests: ResMut<Chests>,
    mut generation: ResMut<Generation>,
    mut save: ResMut<Save>,
) {
    let Some(found) = save_read::<Saved>(theme::SAVE_SLOT) else {
        return;
    };
    generation.seed = found.seed;
    generation.plan = found.plan;
    chests.slots = found.chests.clone();
    world.edited = found.edited.clone();
    save.found = Some(found);
}

pub fn take_the_save(
    mut player: ResMut<Player>,
    mut inventory: ResMut<Inventory>,
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
    mut toast: ResMut<Toast>,
    mut save: ResMut<Save>,
) {
    let Some(found) = save.found.as_ref() else {
        return;
    };
    player.velocity = Vec3::zeros();
    player.yaw = found.yaw;
    inventory.slots = [None; theme::INVENTORY_SLOTS];
    for (slot, stack) in inventory.slots.iter_mut().zip(&found.slots) {
        *slot = *stack;
    }
    inventory.selected = found.selected.min(theme::HOTBAR_SLOTS - 1);
    inventory.coins = found.coins;
    if let Some(kept) = &found.hero {
        hero.class = kept.class.min(crate::data::CALLING_COUNT - 1);
        hero.name = kept.name.clone();
        hero.named = !kept.name.is_empty();
        journal.quests.active = kept.active.clone();
        journal.quests.finished = kept.finished.clone();
        journal.quests.finished.resize(QUEST_COUNT, false);
        journal.quests.near_giver = vec![false; GIVER_COUNT];
        journal.clock.discovered = kept.discovered.clone();
        journal.clock.discovered.resize(PLACE_COUNT, false);
    }
    save.hero = found.hero.clone();
    toast_show(&mut toast, "Welcome back");
}

pub fn sow_the_saved_things(
    world: Res<VoxelWorld>,
    sheet: Res<Sheet>,
    save: Res<Save>,
    mut commands: Commands,
) {
    let Some(found) = save.found.as_ref() else {
        return;
    };
    for crop in &found.crops {
        let block = IVec3::new(crop.block[0], crop.block[1], crop.block[2]);
        let entity = plant(&mut commands, &world, &sheet, crop.kind, block);
        commands.set(entity, crop.kind);
        commands.set(entity, Block(block));
        commands.set(entity, Progress(crop.progress));
        commands.set(entity, Stage(0));
        commands.set(entity, Watered(false));
    }
}

pub fn note_the_hero(
    hero: Res<Hero>,
    journal: Res<Journal>,
    places: Res<Places>,
    mut save: ResMut<Save>,
    mut players: Query<(&Level, &Experience, &Kills, &Health, &Mana), With<Cooldowns>>,
) {
    let Some(player) = hero.player else {
        return;
    };
    let mut kept = None;
    players.for_entity(player, |(level, experience, kills, health, mana)| {
        kept = Some(SavedHero {
            class: hero.class,
            name: hero.name.clone(),
            level: level.0,
            experience: experience.0,
            kills: kills.0,
            health: health.0,
            mana: mana.0,
            town: places.town,
            active: journal.quests.active.clone(),
            finished: journal.quests.finished.clone(),
            discovered: journal.clock.discovered.clone(),
        });
    });
    if kept.is_some() {
        save.hero = kept;
    }
}

pub fn keep_the_world(
    time: Res<Time>,
    window: Res<Window>,
    world: Res<VoxelWorld>,
    chests: Res<Chests>,
    generation: Res<Generation>,
    player: Res<Player>,
    inventory: Res<Inventory>,
    mut save: ResMut<Save>,
    mut toast: ResMut<Toast>,
    mut crops: Query<(&CropKind, &Block, &Progress)>,
) {
    let leaving = window.should_exit || window.close_requested;
    let Some(due) = save.autosave.due(time.delta_time.min(0.05), leaving) else {
        return;
    };
    let mut sown = Vec::new();
    crops.for_each_mut(|_, (kind, block, progress)| {
        sown.push(SavedCrop {
            kind: *kind,
            block: [block.0.x, block.0.y, block.0.z],
            progress: progress.0,
        })
    });
    let town = generation.zones.first().map(|zone| zone.center);
    let kept = Saved {
        seed: generation.seed,
        position: [player.position.x, player.position.y, player.position.z],
        home: [player.home.x, player.home.y, player.home.z],
        yaw: player.yaw,
        slots: inventory.slots.to_vec(),
        selected: inventory.selected,
        coins: inventory.coins,
        chests: chests.slots.clone(),
        edited: world.edited.clone(),
        crops: sown,
        plan: generation.plan,
        town,
        hero: save.hero.clone(),
    };
    let label = match &kept.hero {
        Some(hero) => format!("{}, level {}", hero.name, hero.level),
        None => format!("Seed {}", generation.seed),
    };
    let written = save_write(theme::SAVE_SLOT, &label, &kept);
    if due == SaveDue::Asked {
        toast_show(
            &mut toast,
            if written {
                "The world is saved"
            } else {
                "The world could not be saved"
            },
        );
    }
}
