use crate::commands::{craft, inventory_add, inventory_take, select_slot, swap_slots, toast_show};
use crate::data::{Ask, ItemStack, RECIPES, SHOP};
use crate::queries::inventory_count;
use crate::resources::{
    Chests, Draft, Generation, Hero, Inventory, Save, Screen, Settings, Toast, UiHandles,
};
use Random;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_random::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;

pub fn handle_the_keys(
    state: Res<State<Screen>>,
    actions: Res<Actions<Ask>>,
    mut chests: ResMut<Chests>,
    mut next: ResMut<NextState<Screen>>,
) {
    let back = actions.just_pressed(Ask::Back);
    let pocket = actions.just_pressed(Ask::Pocket);
    match state.current {
        Screen::Title | Screen::Creation => {}
        Screen::NewWorld => {
            if back {
                next.pending = Some(Screen::Title);
            }
        }
        Screen::Playing => {
            if pocket {
                next.pending = Some(Screen::Inventory);
            } else if actions.just_pressed(Ask::Menu) {
                next.pending = Some(Screen::Settings);
            }
        }
        Screen::Settings | Screen::Inventory => {
            if back || pocket {
                next.pending = Some(Screen::Playing);
            }
        }
        Screen::Crafting | Screen::Shop => {
            if back {
                next.pending = Some(Screen::Playing);
            }
        }
        Screen::Chest => {
            if back || pocket {
                chests.open = None;
                next.pending = Some(Screen::Playing);
            }
        }
    }
}

pub fn title_buttons(
    handles: Res<UiHandles>,
    generation: Res<Generation>,
    hero: Res<Hero>,
    mut draft: ResMut<Draft>,
    mut window: ResMut<Window>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, handles.title_continue) {
        next.pending = Some(if hero.named {
            Screen::Playing
        } else {
            Screen::Creation
        });
        return;
    }
    if ui_clicked(&mut buttons, handles.title_new) {
        draft.seed = generation.seed;
        next.pending = Some(Screen::NewWorld);
        return;
    }
    if ui_clicked(&mut buttons, handles.title_leave) {
        window.should_exit = true;
    }
}

pub fn new_world_buttons(
    handles: Res<UiHandles>,
    mut draft: ResMut<Draft>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, handles.new_world_back) {
        next.pending = Some(Screen::Title);
        return;
    }
    if ui_clicked(&mut buttons, handles.new_world_make) {
        draft.asked = true;
        return;
    }
    let rolled = ui_clicked(&mut buttons, handles.seed_button);
    let turned = [
        (handles.land_button, 0),
        (handles.woods_button, 1),
        (handles.caves_button, 2),
        (handles.villages_button, 3),
        (handles.dungeons_button, 4),
        (handles.weather_button, 5),
        (handles.start_button, 6),
    ]
    .into_iter()
    .find(|(entity, _)| ui_clicked(&mut buttons, *entity))
    .map(|(_, row)| row);
    if rolled {
        draft.seed = Random::seeded(fastrand::u64(..)).u32(..);
    }
    match turned {
        Some(0) => draft.plan.land = draft.plan.land.next(),
        Some(1) => draft.plan.woods = draft.plan.woods.next(),
        Some(2) => draft.plan.caves = !draft.plan.caves,
        Some(3) => draft.plan.villages = !draft.plan.villages,
        Some(4) => draft.plan.dungeons = !draft.plan.dungeons,
        Some(5) => draft.plan.weather = !draft.plan.weather,
        Some(6) => draft.plan.start = draft.plan.start.next(),
        _ => {}
    }
}

pub fn settings_buttons(
    handles: Res<UiHandles>,
    mut settings: ResMut<Settings>,
    mut save: ResMut<Save>,
    mut window: ResMut<Window>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, handles.settings_close) {
        next.pending = Some(Screen::Playing);
        return;
    }
    if ui_clicked(&mut buttons, handles.rate_toggle) {
        settings.show_rate = !settings.show_rate;
    }
    if ui_clicked(&mut buttons, handles.settings_save) {
        save.autosave.ask();
    }
    if ui_clicked(&mut buttons, handles.settings_title) {
        save.autosave.ask();
        next.pending = Some(Screen::Title);
        return;
    }
    if ui_clicked(&mut buttons, handles.settings_leave) {
        save.autosave.ask();
        window.should_exit = true;
    }
}

pub fn hotbar_buttons(
    handles: Res<UiHandles>,
    mut inventory: ResMut<Inventory>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    let picked = handles
        .slots
        .iter()
        .position(|slot| ui_clicked(&mut buttons, slot.frame));
    if let Some(index) = picked {
        select_slot(&mut inventory, index);
    }
}

pub fn shop_buttons(
    handles: Res<UiHandles>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, handles.shop_close) {
        next.pending = Some(Screen::Playing);
        return;
    }
    for handle in &handles.shop_rows {
        if !ui_clicked(&mut buttons, handle.button) {
            continue;
        }
        let row = SHOP[handle.index];
        let message = if let Some(price) = row.buy {
            if inventory.coins < price {
                "Not enough copper".to_string()
            } else if inventory_add(&mut inventory, row.item, 1) > 0 {
                "No room in the pocket".to_string()
            } else {
                inventory.coins -= price;
                format!("Bought {}", row.item.name())
            }
        } else if let Some(price) = row.sell {
            if inventory_take(&mut inventory, row.item, 1) {
                inventory.coins += price;
                format!("Sold {} for {price} copper", row.item.name())
            } else {
                format!("No {} to sell", row.item.name())
            }
        } else {
            String::new()
        };
        toast_show(&mut toast, message);
    }
}

pub fn crafting_buttons(
    handles: Res<UiHandles>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, handles.crafting_close) {
        next.pending = Some(Screen::Playing);
        return;
    }
    for handle in &handles.crafting_rows {
        if !ui_clicked(&mut buttons, handle.button) {
            continue;
        }
        let recipe = &RECIPES[handle.index];
        let message = if craft(&mut inventory, recipe) {
            format!(
                "Made {} {}  (have {})",
                recipe.count,
                recipe.output.name(),
                inventory_count(&inventory, recipe.output)
            )
        } else {
            "Missing the parts, or no room".to_string()
        };
        toast_show(&mut toast, message);
    }
}

pub fn pocket_buttons(
    mut handles: ResMut<UiHandles>,
    mut inventory: ResMut<Inventory>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, handles.inventory_craft) {
        next.pending = Some(Screen::Crafting);
        return;
    }
    if ui_clicked(&mut buttons, handles.inventory_close) {
        next.pending = Some(Screen::Playing);
        return;
    }
    let picked = handles
        .pocket_slots
        .iter()
        .position(|cell| ui_clicked(&mut buttons, cell.frame));
    let Some(index) = picked else {
        return;
    };
    let holding = handles.holding;
    match holding {
        Some(first) if first == index => {
            handles.holding = None;
        }
        Some(first) => {
            swap_slots(&mut inventory, first, index);
            handles.holding = None;
            handles.shown_pocket.clear();
            handles.shown_marks = None;
        }
        None => {
            select_slot(&mut inventory, index);
            handles.holding = Some(index);
            handles.shown_marks = None;
        }
    }
}

pub fn chest_buttons(
    mut handles: ResMut<UiHandles>,
    mut chests: ResMut<Chests>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, handles.chest_close) {
        chests.open = None;
        next.pending = Some(Screen::Playing);
        return;
    }
    let Some(key) = chests.open else {
        return;
    };
    let from_chest = handles
        .chest_slots
        .iter()
        .position(|cell| ui_clicked(&mut buttons, cell.frame));
    if let Some(index) = from_chest {
        let Some(slots) = chests.slots.get_mut(&key) else {
            return;
        };
        let Some(stack) = slots.get_mut(index).and_then(|slot| slot.take()) else {
            return;
        };
        let left = inventory_add(&mut inventory, stack.item, stack.count);
        if left > 0 {
            if let Some(slot) = slots.get_mut(index) {
                *slot = Some(ItemStack {
                    item: stack.item,
                    count: left,
                });
            }
            toast_show(&mut toast, "No room in the pocket");
        }
        handles.shown_chest.clear();
        handles.shown_chest_pocket.clear();
        return;
    }
    let from_pocket = handles
        .chest_pocket
        .iter()
        .position(|cell| ui_clicked(&mut buttons, cell.frame));
    let Some(index) = from_pocket else {
        return;
    };
    let Some(slot) = inventory.slots.get_mut(index) else {
        return;
    };
    let Some(stack) = slot.take() else {
        return;
    };
    let Some(slots) = chests.slots.get_mut(&key) else {
        return;
    };
    if let Some(left) = store_in_chest(slots, stack) {
        *slot = Some(left);
        toast_show(&mut toast, "The chest is full");
    }
    handles.shown_chest.clear();
    handles.shown_chest_pocket.clear();
}

fn store_in_chest(slots: &mut [Option<ItemStack>], stack: ItemStack) -> Option<ItemStack> {
    let mut remaining = stack.count;
    for slot in slots.iter_mut() {
        if remaining == 0 {
            break;
        }
        match slot {
            Some(held) if held.item == stack.item && held.count < stack.item.max_stack() => {
                let room = stack.item.max_stack() - held.count;
                let taken = room.min(remaining);
                held.count += taken;
                remaining -= taken;
            }
            _ => {}
        }
    }
    for slot in slots.iter_mut() {
        if remaining == 0 {
            break;
        }
        if slot.is_none() {
            let taken = remaining.min(stack.item.max_stack());
            *slot = Some(ItemStack {
                item: stack.item,
                count: taken,
            });
            remaining -= taken;
        }
    }
    (remaining > 0).then_some(ItemStack {
        item: stack.item,
        count: remaining,
    })
}
