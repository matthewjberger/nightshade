use crate::commands::{plant, spawn_a_boat, toast_show};
use crate::components::{Block, Boat, Progress, Stage, Watered};
use crate::data::CropKind;
use crate::resources::{
    Chests, Generation, Inventory, Player, Save, Saved, SavedBoat, SavedCrop, Settings, Sheet,
    Toast, VoxelWorld,
};
use crate::theme;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_save::prelude::{SaveDue, save_read, save_write};
use nightshade_scene::prelude::LocalTransform;

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
    mut settings: ResMut<Settings>,
    mut toast: ResMut<Toast>,
    save: Res<Save>,
) {
    let Some(found) = save.found.as_ref() else {
        return;
    };
    settings.palette = found.palette;
    player.health = found.health;
    player.food = found.food;
    player.velocity = Vec3::zeros();
    inventory.slots = [None; theme::INVENTORY_SLOTS];
    for (slot, stack) in inventory.slots.iter_mut().zip(&found.slots) {
        *slot = *stack;
    }
    inventory.selected = found.selected.min(theme::HOTBAR_SLOTS - 1);
    inventory.coins = found.coins;
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
    for boat in &found.boats {
        spawn_a_boat(
            &mut commands,
            Vec3::new(boat.place[0], boat.place[1], boat.place[2]),
            boat.yaw,
        );
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
    settings: Res<Settings>,
    mut save: ResMut<Save>,
    mut toast: ResMut<Toast>,
    mut crops: Query<(&CropKind, &Block, &Progress)>,
    mut boats: Query<(&Boat, &LocalTransform)>,
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
    let mut sailing = Vec::new();
    boats.for_each_mut(|_, (boat, transform)| {
        let place = transform.translation;
        sailing.push(SavedBoat {
            place: [place.x, place.y, place.z],
            yaw: boat.yaw,
        })
    });
    let kept = Saved {
        seed: generation.seed,
        position: [player.position.x, player.position.y, player.position.z],
        home: [player.home.x, player.home.y, player.home.z],
        yaw: player.yaw,
        pitch: player.pitch,
        health: player.health,
        food: player.food,
        slots: inventory.slots.to_vec(),
        selected: inventory.selected,
        coins: inventory.coins,
        chests: chests.slots.clone(),
        edited: world.edited.clone(),
        crops: sown,
        boats: sailing,
        palette: settings.palette,
        plan: generation.plan,
    };
    let label = format!("Seed {}", generation.seed);
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
