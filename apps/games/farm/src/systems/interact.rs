use crate::commands::farming::{harvest, plant, till, water};

use crate::commands::fishing::{cast, land, reel_in};
use crate::commands::pocket::{show_toast, start_dialogue};
use crate::commands::tools::{chop, clear_stump, dig, milk, pick, swing};
use crate::commands::world::free;
use crate::components::{Bush, Hits, Inside, Milkable, Npc, Progress, Rest, Stump, Wood};
use crate::data::{Ask, CropKind, Slot, Tool};
use crate::queries::pocket::{count, effort_of, is_usable, level_of, selected};
use crate::queries::prompt::{indoors, repeats};
use crate::queries::save::snapshot;
use crate::queries::talk::{Mood, lines_for};
use crate::queries::terrain::{
    beside_water, center, column_of, ground_of, is_tillable, is_water, neighbors, plot_at,
};
use crate::resources::{
    Diorama, Doors, Dungeon, Farmland, Fishing, FishingState, Hero, Home, Inventory, Prompt,
    Quests, Screen, Sheet, Terrain, Weather,
};
use crate::tags::Rocks;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_render::data::config::RendererState;
use nightshade_save::prelude::save_write;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::Sprite;

pub fn find_the_prompt(
    diorama: Res<Diorama>,
    terrain: Res<Terrain>,
    farmland: Res<Farmland>,
    inventory: Res<Inventory>,
    doors: Res<Doors>,
    renderer_state: Res<RendererState>,
    fishing: Res<Fishing>,
    dungeon: Res<Dungeon>,
    home: Res<Home>,
    mut prompt: ResMut<Prompt>,
    people: Query<(&Npc, &Inside, &LocalTransform)>,
    mut crops: Query<(&CropKind, &Progress)>,
    trees: Query<(&Hits, &LocalTransform)>,
    rocks: Query<(&LocalTransform,), WithTag<Rocks>>,
    cows: Query<(&Milkable, &LocalTransform)>,
    bushes: Query<(&Bush, &LocalTransform)>,
    stumps: Query<(&Stump, &LocalTransform)>,
) {
    *prompt = Prompt::None;
    if dungeon.active {
        return;
    }
    if home.active {
        *prompt = indoors(&home, diorama.feet, renderer_state.day_night.hour);
        return;
    }
    match fishing.state {
        FishingState::Waiting(_) => {
            *prompt = Prompt::Reel;
            return;
        }
        FishingState::Bite(_) => {
            *prompt = Prompt::Pull;
            return;
        }
        FishingState::Idle => {}
    }
    let ground = Vec2::new(diorama.feet.x, diorama.feet.z);
    let day = diorama.day;

    let mut nearest: Option<(f32, Entity, String)> = None;
    people.for_each(|entity, (npc, inside, transform)| {
        if inside.0 {
            return;
        }
        let distance = (ground_of(transform) - ground).magnitude();
        if distance < theme::TALK_RADIUS
            && nearest.as_ref().is_none_or(|(best, _, _)| distance < *best)
        {
            nearest = Some((distance, entity, npc.person.name().to_string()));
        }
    });
    if let Some((_, entity, name)) = nearest {
        *prompt = Prompt::Talk(entity, name);
        return;
    }

    for door in &doors.doors {
        if (door.spot - ground).magnitude() > theme::DOOR_RADIUS {
            continue;
        }
        *prompt = Prompt::EnterHouse(door.kind);
        return;
    }

    let mut ripe_bush: Option<(f32, Entity)> = None;
    bushes.for_each(|entity, (bush, transform)| {
        if bush.picked_day == Some(day) {
            return;
        }
        let distance = (ground_of(transform) - ground).magnitude();
        if distance < theme::PICK_RADIUS && ripe_bush.is_none_or(|(near, _)| distance < near) {
            ripe_bush = Some((distance, entity));
        }
    });
    if let Some((_, entity)) = ripe_bush {
        *prompt = Prompt::Pick(entity);
        return;
    }

    let Some((column, row)) = column_of(&terrain, ground.x, ground.y) else {
        return;
    };
    let slot = selected(&inventory);
    let usable = is_usable(&inventory, slot);
    if let Some(plot_index) = plot_at(&farmland, column, row) {
        let plot = &farmland.plots[plot_index];
        match plot.crop {
            Some(crop) => {
                let mut ripe = false;
                crops.for_entity(crop, |(_, progress)| ripe = progress.0 >= 1.0);
                if ripe {
                    *prompt = Prompt::Harvest(plot_index);
                } else if slot == Slot::Tool(Tool::WateringCan)
                    && usable
                    && plot.watered_day != Some(day)
                {
                    *prompt = Prompt::Water(plot_index);
                }
            }
            None => {
                if let Slot::Seed(kind) = slot
                    && count(&inventory, kind.seed()) > 0
                {
                    *prompt = Prompt::Plant(plot_index, kind);
                }
            }
        }
        if *prompt != Prompt::None {
            return;
        }
    }
    if !usable {
        return;
    }
    match slot {
        Slot::Tool(Tool::Hoe) => {
            if is_tillable(&terrain, &farmland, column, row) {
                *prompt = Prompt::Till(column, row);
            }
        }
        Slot::Tool(Tool::Axe) => {
            let mut best: Option<(f32, Entity)> = None;
            trees.for_each(|entity, (_, transform)| {
                let distance = (ground_of(transform) - ground).magnitude();
                if distance < theme::CHOP_RADIUS && best.is_none_or(|(near, _)| distance < near) {
                    best = Some((distance, entity));
                }
            });
            if let Some((_, entity)) = best {
                *prompt = Prompt::Chop(entity);
                return;
            }
            let mut nearest_stump: Option<(f32, Entity)> = None;
            stumps.for_each(|entity, (_stump, transform)| {
                let distance = (ground_of(transform) - ground).magnitude();
                if distance < theme::CHOP_RADIUS
                    && nearest_stump.is_none_or(|(near, _)| distance < near)
                {
                    nearest_stump = Some((distance, entity));
                }
            });
            if let Some((_, entity)) = nearest_stump {
                *prompt = Prompt::Clear(entity);
            }
        }
        Slot::Tool(Tool::Shovel) => {
            let mut best: Option<(f32, Entity)> = None;
            rocks.for_each(|entity, (transform,)| {
                let distance = (ground_of(transform) - ground).magnitude();
                if distance < theme::DIG_RADIUS && best.is_none_or(|(near, _)| distance < near) {
                    best = Some((distance, entity));
                }
            });
            if let Some((_, entity)) = best {
                *prompt = Prompt::Dig(entity);
            }
        }
        Slot::Tool(Tool::Bucket) => {
            let mut best: Option<(f32, Entity)> = None;
            cows.for_each(|entity, (cow, transform)| {
                if cow.milked_day == Some(day) {
                    return;
                }
                let distance = (ground_of(transform) - ground).magnitude();
                if distance < theme::MILK_RADIUS && best.is_none_or(|(near, _)| distance < near) {
                    best = Some((distance, entity));
                }
            });
            if let Some((_, entity)) = best {
                *prompt = Prompt::Milk(entity);
            }
        }
        Slot::Tool(Tool::FishingRod) => {
            if beside_water(&terrain, column, row) {
                *prompt = Prompt::Cast;
            }
        }
        Slot::Tool(Tool::WateringCan) | Slot::Seed(_) => {}
    }
}

pub fn act_on_the_prompt(
    actions: Res<Actions<Ask>>,
    prompt_state: Res<Prompt>,
    sheet_state: Res<Sheet>,
    hero: Res<Hero>,
    quests: Res<Quests>,
    weather: Res<Weather>,
    dungeon: Res<Dungeon>,
    farm: crate::resources::Homestead,
    house: crate::resources::Household,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let commands = &mut commands;
    let (tapped, held) = (actions.just_pressed(Ask::Act), actions.pressed(Ask::Act));
    if !held {
        return;
    }
    let prompt = prompt_state.clone();
    if !tapped && !repeats(&prompt) {
        return;
    }
    let sheet = *sheet_state;
    let hour = house.renderer_state.day_night.hour;
    let day = house.diorama.day;
    let feet = house.diorama.feet;
    let farmer = house.diorama.farmer;
    let mut flip = false;
    let mut resting = true;
    if let Some(farmer) = farmer {
        storage
            .query::<(&mut Rest, &Sprite)>()
            .for_entity(farmer, |(rest, sprite)| {
                flip = sprite.flip_x;
                resting = rest.0 > 0.0;
                if !resting {
                    rest.0 = theme::ACT_COOLDOWN;
                }
            });
    }
    if resting {
        return;
    }
    let held = match prompt {
        Prompt::Till(_, _) => Some(Tool::Hoe),
        Prompt::Water(_) => Some(Tool::WateringCan),
        Prompt::Chop(_) | Prompt::Clear(_) => Some(Tool::Axe),
        Prompt::Dig(_) => Some(Tool::Shovel),
        Prompt::Milk(_) => Some(Tool::Bucket),
        Prompt::Cast | Prompt::Reel | Prompt::Pull => Some(Tool::FishingRod),
        _ => None,
    };
    let mut terrain = std::mem::take(farm.terrain);
    let mut farmland = std::mem::take(farm.farmland);
    let mut inventory = std::mem::take(farm.inventory);
    let mut toast = std::mem::take(farm.toast);
    let mut fishing = std::mem::take(farm.fishing);
    let mut vigor = std::mem::take(farm.vigor);
    if let (Some(farmer), Some(tool)) = (farmer, held)
        && vigor.stamina >= effort_of(&inventory, tool)
    {
        swing(commands, &sheet, farmer, tool, flip);
    }
    if let Some(tool) = held {
        let cost = effort_of(&inventory, tool);
        if vigor.stamina < cost {
            show_toast(&mut toast, "Too tired. Eat something, or sleep.");
            *farm.terrain = terrain;
            *farm.farmland = farmland;
            *farm.inventory = inventory;
            *farm.toast = toast;
            *farm.fishing = fishing;
            *farm.vigor = vigor;
            return;
        }
        vigor.stamina -= cost;
    }
    match prompt {
        Prompt::None => {}
        Prompt::Talk(entity, _) => {
            let mut person = None;
            storage
                .query::<(&Npc,)>()
                .for_entity(entity, |(npc,)| person = Some(npc.person));
            if let Some(person) = person {
                let mood = Mood {
                    hour,
                    raining: weather.raining,
                    day,
                    forecast: crate::queries::weather::forecast(&weather),
                    has_weapon: hero.weapon != crate::data::Weapon::Fists,
                };
                let (lines, after) = lines_for(person, &mood, &inventory, &quests);
                start_dialogue(house.dialogue, person.name(), lines, after);
                house.next_screen.pending = Some(Screen::Talking);
            }
        }
        Prompt::Sleep => {
            let skipped = if hour >= theme::SLEEP_FROM {
                24.0 - hour + theme::MORNING_HOUR
            } else {
                theme::MORNING_HOUR - hour
            };
            house.renderer_state.day_night.hour = theme::MORNING_HOUR;
            {
                let diorama = &mut *house.diorama;
                diorama.skipped_hours += skipped;
                diorama.last_hour = theme::MORNING_HOUR;
                if hour >= theme::SLEEP_FROM {
                    diorama.day += 1;
                }
            }
            if hour >= theme::SLEEP_FROM {
                for plot in &mut farmland.plots {
                    if plot.watered_day == Some(day) {
                        plot.watered_day = Some(day + 1);
                    }
                }
            }
            vigor.stamina = theme::MAX_STAMINA;
            let mut growing = Vec::new();
            for (index, plot) in farmland.plots.iter().enumerate() {
                let Some(entity) = plot.crop else {
                    continue;
                };
                storage
                    .query::<(&CropKind, &Progress)>()
                    .for_entity(entity, |(kind, progress)| {
                        growing.push((index, *kind, progress.0))
                    });
            }
            let written = snapshot(
                house.diorama,
                theme::MORNING_HOUR,
                &inventory,
                &quests,
                &hero,
                &vigor,
                &dungeon,
                &farmland,
                &growing,
            );
            let kept = save_write(theme::SAVE_SLOT, &format!("Day {}", written.day), &written);
            show_toast(
                &mut toast,
                if kept {
                    "Slept until morning. The farm is written down."
                } else {
                    "Slept until morning"
                },
            );
        }
        Prompt::EnterShop(kind) => {
            house.open_shop.kind = kind;
            house.next_screen.pending = Some(Screen::Shop);
        }
        Prompt::Till(column, row) => {
            let reach = level_of(&inventory, Tool::Hoe);
            let mut cells = vec![(column, row)];
            if reach > 0 {
                cells.extend(
                    neighbors(&terrain, column, row)
                        .into_iter()
                        .flatten()
                        .filter(|(near_column, near_row)| {
                            is_tillable(&terrain, &farmland, *near_column, *near_row)
                        }),
                );
            }
            let tilled = cells.len();
            for (near_column, near_row) in cells {
                till(commands, &terrain, &mut farmland, near_column, near_row);
            }
            show_toast(&mut toast, format!("Tilled {} plots", tilled));
        }
        Prompt::Water(plot_index) => {
            let reach = level_of(&inventory, Tool::WateringCan);
            let plot = farmland.plots[plot_index].clone();
            let mut wet = vec![plot_index];
            if reach > 0 {
                for (index, near) in farmland.plots.iter().enumerate() {
                    let apart = plot.column.abs_diff(near.column) + plot.row.abs_diff(near.row);
                    if index != plot_index && apart <= reach {
                        wet.push(index);
                    }
                }
            }
            let count = wet.len();
            for index in wet {
                water(commands, &mut farmland, index, day);
            }
            show_toast(&mut toast, format!("Watered {} plots", count));
        }
        Prompt::Chop(entity) => {
            storage
                .query::<(&mut Hits, &Wood, &LocalTransform)>()
                .for_entity(entity, |(hits, wood, transform)| {
                    chop(
                        commands,
                        &sheet,
                        &mut toast,
                        entity,
                        hits,
                        wood,
                        transform.translation,
                        day,
                        1 + level_of(&inventory, Tool::Axe),
                    );
                });
        }
        Prompt::Clear(entity) => {
            let mut spot = None;
            storage
                .query::<(&Stump, &LocalTransform)>()
                .for_entity(entity, |(_stump, transform)| {
                    spot = Some(transform.translation)
                });
            if let Some(spot) = spot {
                clear_stump(commands, &sheet, &mut toast, entity, spot);
                if let Some((column, row)) = column_of(&terrain, spot.x, spot.z) {
                    free(&mut terrain, column, row);
                }
            }
        }
        Prompt::Pick(entity) => {
            storage
                .query::<(&mut Bush, &mut Sprite, &LocalTransform)>()
                .for_entity(entity, |(bush, sprite, transform)| {
                    pick(
                        commands,
                        &sheet,
                        &mut inventory,
                        &mut toast,
                        bush,
                        sprite,
                        transform.translation,
                        day,
                    );
                });
        }
        Prompt::Dig(entity) => {
            storage
                .query::<(&LocalTransform,)>()
                .with_tag_type::<Rocks>()
                .for_entity(entity, |(transform,)| {
                    dig(commands, &sheet, &mut toast, entity, transform.translation);
                });
        }
        Prompt::Milk(entity) => {
            storage
                .query::<(&mut Milkable, &LocalTransform)>()
                .for_entity(entity, |(cow, transform)| {
                    cow.milked_day = Some(day);
                    milk(commands, &mut inventory, &mut toast, transform.translation);
                });
        }
        Prompt::Cast => {
            let ground = Vec2::new(feet.x, feet.z);
            if let Some((column, row)) = column_of(&terrain, ground.x, ground.y) {
                let spot = neighbors(&terrain, column, row)
                    .into_iter()
                    .flatten()
                    .find(|(c, r)| is_water(&terrain, *c, *r))
                    .map(|(c, r)| center(&terrain, c, r));
                if let Some(spot) = spot {
                    cast(commands, &sheet, &mut fishing, &mut toast, spot);
                }
            }
        }
        Prompt::Reel => {
            reel_in(commands, &mut fishing);
            show_toast(&mut toast, "Reeled in");
        }
        Prompt::Pull => {
            land(commands, &mut fishing, &mut inventory, &mut toast);
        }
        Prompt::Plant(plot_index, kind) => {
            plant(
                commands,
                &sheet,
                &terrain,
                &mut farmland,
                &mut inventory,
                &mut toast,
                plot_index,
                kind,
            );
        }
        Prompt::EnterHollow | Prompt::ClimbUp | Prompt::ClimbDown | Prompt::Open(_) => {}
        Prompt::EnterHouse(kind) => {
            house.home.pending = Some(kind);
        }
        Prompt::LeaveHouse => {
            house.home.leaving = true;
        }
        Prompt::Harvest(plot_index) => {
            if let Some(crop) = farmland.plots[plot_index].crop {
                let mut kind = None;
                storage
                    .query::<(&CropKind, &Progress)>()
                    .for_entity(crop, |(crop_kind, _)| kind = Some(*crop_kind));
                if let Some(kind) = kind {
                    harvest(
                        commands,
                        &terrain,
                        &mut farmland,
                        &mut inventory,
                        &mut toast,
                        plot_index,
                        kind,
                    );
                }
            }
        }
    }
    *farm.terrain = terrain;
    *farm.farmland = farmland;
    *farm.inventory = inventory;
    *farm.toast = toast;
    *farm.fishing = fishing;
    *farm.vigor = vigor;
}
