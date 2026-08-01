use crate::resources::{Save, Saved, SavedBoat, SavedCrop};
use crate::theme;
use blocker_blocks::data::crops::CropKind;
use blocker_blocks::resources::Art;
use blocker_blocks::resources::{Sheet, VoxelWorld};
use blocker_farm::commands::plant;
use blocker_farm::components::{Block, Progress};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_player::commands::spawn_a_boat;
use blocker_player::components::{Boat, Food, Pitch, Spawn, Yaw};
use blocker_player::resources::{Resume, Resumed};
use blocker_pocket::commands::select_slot;
use blocker_pocket::queries::{slots_from, stacks_of};
use blocker_pocket::resources::{Chests, Inventory};
use blocker_screen::data::notice::Notice;
use blocker_world::resources::Generation;
use nalgebra_glm::{IVec3, Vec3, vec3};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_health::Health;
use nightshade_platform::prelude::*;
use nightshade_save::prelude::{SaveDue, save_read, save_write};
use nightshade_scene::prelude::LocalTransform;

pub fn read_the_save(
    mut world: ResMut<VoxelWorld>,
    mut chests: ResMut<Chests>,
    mut generation: ResMut<Generation>,
    mut resume: ResMut<Resume>,
    mut save: ResMut<Save>,
) {
    let Some(found) = save_read::<Saved>(theme::SAVE_SLOT) else {
        return;
    };
    generation.seed = found.seed;
    chests.0 = found
        .chests
        .iter()
        .map(|(key, stacks)| (*key, slots_from(stacks, stacks.len())))
        .collect();
    world.edited = found.edited.clone();
    resume.0 = Some(Resumed {
        position: vec3(found.position[0], found.position[1], found.position[2]),
        home: vec3(found.home[0], found.home[1], found.home[2]),
        yaw: found.yaw,
        pitch: found.pitch,
    });
    save.found = Some(found);
}

pub fn take_the_save(
    mut inventory: ResMut<Inventory>,
    mut art: ResMut<Art>,
    mut notices: EventWriter<Notice>,
    save: Res<Save>,
    hero: Res<Hero>,
    mut bodies: Query<(&mut Health, &mut Food)>,
) {
    let Some(found) = save.found.as_ref() else {
        return;
    };
    let Some(body) = hero.0 else {
        return;
    };
    art.palette = found.palette;
    bodies.for_entity(body, |(health, food)| {
        health.0 = found.health;
        food.0 = found.food;
    });
    inventory.slots = slots_from(&found.slots, blocker_blocks::data::items::INVENTORY_SLOTS);
    select_slot(&mut inventory, found.selected);
    notices.send(Notice("Welcome back".into()));
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
        commands.set(entity, Progress(crop.progress));
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
    inventory: Res<Inventory>,
    art: Res<Art>,
    hero: Res<Hero>,
    mut save: ResMut<Save>,
    mut notices: EventWriter<Notice>,
    mut crops: Query<(&CropKind, &Block, &Progress)>,
    mut boats: Query<(&Boat, &LocalTransform)>,
    mut bodies: Query<(&Health, &Food, &Yaw, &Pitch, &Spawn, &LocalTransform)>,
) {
    let leaving = window.should_exit || window.close_requested;
    let Some(due) = save.autosave.due(time.delta_time.min(0.05), leaving) else {
        return;
    };
    let Some((health, food, yaw, pitch, home, stood)) = read_the_hero(
        &hero,
        &mut bodies,
        |(health, food, yaw, pitch, spawn, transform)| {
            (
                health.0,
                food.0,
                yaw.0,
                pitch.0,
                spawn.0,
                transform.translation,
            )
        },
    ) else {
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
            yaw: boat.0,
        })
    });
    let kept = Saved {
        seed: generation.seed,
        position: [stood.x, stood.y, stood.z],
        home: [home.x, home.y, home.z],
        yaw,
        pitch,
        health,
        food,
        slots: stacks_of(&inventory.slots),
        selected: inventory.selected.0,
        chests: chests
            .0
            .iter()
            .map(|(key, slots)| (*key, stacks_of(slots)))
            .collect(),
        edited: world.edited.clone(),
        crops: sown,
        boats: sailing,
        palette: art.palette,
    };
    let label = format!("Seed {}", generation.seed);
    let written = save_write(theme::SAVE_SLOT, &label, &kept);
    if due == SaveDue::Asked {
        notices.send(Notice(
            if written {
                "The world is saved"
            } else {
                "The world could not be saved"
            }
            .into(),
        ));
    }
}
