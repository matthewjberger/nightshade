use crate::commands::effects::spawn_bobber;
use crate::commands::pickups::scatter_items;
use crate::commands::pocket::{show_toast, take_item};
use crate::components::{Regrow, Ripe, Rocks, Shaking, Spot, Tree, Villager};
use crate::data;
use crate::data::Ask;
use crate::queries::cylinder;
use crate::queries::interact::{shop_doors, shop_name, water_near};
use crate::queries::sheet::tile;
use crate::queries::town::lift_at;
use crate::resources::{
    Chat, Deed, Fishing, FishingState, Pocket, Prompt, Screen, Sheet, Store, Toast, Town, Village,
};
use crate::tags::Bush;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_sprite::prelude::*;
use nightshade_state::prelude::*;

pub fn find_the_deed(
    village: Res<Village>,
    town: Res<Town>,
    fishing: Res<Fishing>,
    mut prompt: ResMut<Prompt>,
    villagers: Query<(&Villager, &Spot)>,
    bushes: Query<(&Ripe, &Spot), WithTag<Bush>>,
    trees: Query<(&Tree, &Spot)>,
    rocks: Query<(&Rocks, &Spot)>,
) {
    let feet = village.feet;
    if fishing.state != FishingState::Idle {
        prompt.deed = Deed::Reel;
        prompt.line = match fishing.state {
            FishingState::Bite => "Pull the line".to_string(),
            _ => "Wait for a bite".to_string(),
        };
        return;
    }

    let mut nearest: Option<(f32, Deed)> = None;
    let mut line = String::new();
    let closer = |distance: f32, nearest: &Option<(f32, Deed)>| {
        nearest.is_none_or(|(near, _)| distance < near)
    };

    villagers.for_each(|_entity, (villager, spot)| {
        let distance = cylinder::town_distance(feet, spot.0);
        if distance <= data::TALK_RANGE && closer(distance, &nearest) {
            nearest = Some((distance, Deed::Talk(villager.0)));
            line = format!("Say hello to {}", data::NEIGHBORS[villager.0].name);
        }
    });
    bushes.for_each(|entity, (is_ripe, spot)| {
        let distance = cylinder::town_distance(feet, spot.0);
        if is_ripe.0 && distance <= data::ACT_REACH && closer(distance, &nearest) {
            nearest = Some((distance, Deed::Pick(entity)));
            line = "Pick the berries".to_string();
        }
    });
    trees.for_each(|entity, (_tree, spot)| {
        let distance = cylinder::town_distance(feet, spot.0);
        if distance <= data::ACT_REACH && closer(distance, &nearest) {
            nearest = Some((distance, Deed::Shake(entity)));
            line = "Shake the tree".to_string();
        }
    });
    rocks.for_each(|entity, (_rocks, spot)| {
        let distance = cylinder::town_distance(feet, spot.0);
        if distance <= data::ACT_REACH && closer(distance, &nearest) {
            nearest = Some((distance, Deed::Knock(entity)));
            line = "Knock the stones".to_string();
        }
    });
    for (index, door) in shop_doors() {
        let distance = cylinder::town_distance(feet, door);
        if distance <= data::ACT_REACH && closer(distance, &nearest) {
            nearest = Some((distance, Deed::Shop(index)));
            line = format!("Look in {}", shop_name(index));
        }
    }
    if nearest.is_none()
        && let Some(water) = water_near(&town, feet)
    {
        nearest = Some((cylinder::town_distance(feet, water), Deed::Cast));
        line = "Cast a line".to_string();
    }

    prompt.deed = nearest.map_or(Deed::None, |(_, deed)| deed);
    prompt.line = line;
}

pub fn act_on_the_deed(
    actions: Res<Actions<Ask>>,
    prompt: Res<Prompt>,
    sheet: Res<Sheet>,
    village: Res<Village>,
    mut fishing: ResMut<Fishing>,
    mut chat: ResMut<Chat>,
    mut store: ResMut<Store>,
    town: Res<Town>,
    mut toast: ResMut<Toast>,
    mut pocket: ResMut<Pocket>,
    mut state_next: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut bushes: Query<(&mut Ripe, &mut Regrow, &Spot, &mut Sprite), WithTag<Bush>>,
    mut trees: Query<(&mut Tree, &Spot, &mut Shaking)>,
    mut rocks: Query<(&mut Rocks, &Spot)>,
) {
    if !actions.just_pressed(Ask::Interact) {
        return;
    }
    let deed = prompt.deed;
    let sheet = *sheet;
    let Some(turntable) = village.turntable else {
        return;
    };
    let mut rng = fishing.rng.clone();

    match deed {
        Deed::None => {}
        Deed::Talk(index) => {
            chat.villager = Some(index);
            chat.line = 0;
            chat.shown = true;
            state_next.pending = Some(Screen::Talking);
        }
        Deed::Shop(index) => {
            store.keeper = Some(index);
            store.sold = 0;
            state_next.pending = Some(Screen::Shop);
        }
        Deed::Pick(entity) => {
            let mut picked = None;
            bushes.for_entity(entity, |(is_ripe, regrow, spot, sprite)| {
                if !is_ripe.0 {
                    return;
                }
                is_ripe.0 = false;
                regrow.0 = data::BERRY_RIPEN;
                sprite.region = tile(&sheet, data::BUSH);
                picked = Some(spot.0);
            });
            if let Some(spot) = picked {
                let lift = lift_at(&town, spot);
                scatter_items(
                    &mut commands,
                    &sheet,
                    turntable,
                    data::Item::Berry,
                    spot,
                    lift,
                    data::BERRY_YIELD,
                    &mut rng,
                );
            }
        }
        Deed::Shake(entity) => {
            let mut shaken = None;
            trees.for_entity(entity, |(tree, spot, shaking)| {
                if tree.0 > 0.0 {
                    return;
                }
                tree.0 = data::TREE_REST;
                shaking.0 = data::SHAKE_TIME;
                shaken = Some(spot.0);
            });
            match shaken {
                Some(spot) => {
                    let lift = lift_at(&town, spot);
                    if rng.u32(..) % 100 < data::WOOD_CHANCE {
                        scatter_items(
                            &mut commands,
                            &sheet,
                            turntable,
                            data::Item::Wood,
                            spot,
                            lift,
                            1,
                            &mut rng,
                        );
                    } else {
                        show_toast(&mut toast, "Only leaves this time.");
                    }
                }
                None => show_toast(&mut toast, "That tree has nothing left today."),
            }
        }
        Deed::Knock(entity) => {
            let mut knocked = None;
            rocks.for_entity(entity, |(rocks, spot)| {
                if rocks.0 > 0.0 {
                    return;
                }
                rocks.0 = data::ROCK_REST;
                knocked = Some(spot.0);
            });
            match knocked {
                Some(spot) => {
                    let lift = lift_at(&town, spot);
                    scatter_items(
                        &mut commands,
                        &sheet,
                        turntable,
                        data::Item::Stone,
                        spot,
                        lift,
                        1,
                        &mut rng,
                    );
                }
                None => show_toast(&mut toast, "The stones are picked clean today."),
            }
        }
        Deed::Cast => {
            let feet = village.feet;
            let water = water_near(&town, feet);
            if let Some(water) = water {
                let bobber = spawn_bobber(
                    &mut commands,
                    &sheet,
                    turntable,
                    water,
                    data::SEA_LEVEL + 0.12,
                );
                let wait =
                    data::BITE_WAIT_MIN + rng.f32() * (data::BITE_WAIT_MAX - data::BITE_WAIT_MIN);
                fishing.state = FishingState::Waiting;
                fishing.timer = wait;
                fishing.bobber = Some(bobber);
            }
        }
        Deed::Reel => {
            let caught = {
                let caught = fishing.state == FishingState::Bite;
                fishing.state = FishingState::Idle;
                fishing.timer = 0.0;
                if let Some(bobber) = fishing.bobber.take() {
                    commands.despawn(bobber);
                }
                caught
            };
            if caught {
                take_item(&mut pocket, data::Item::Fish, 1);
                show_toast(&mut toast, "You landed a fish.");
            } else {
                show_toast(&mut toast, "It got away.");
            }
        }
    }

    fishing.rng = rng;
}
