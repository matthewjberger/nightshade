use crate::commands::pocket::{add_item, select_slot, show_toast, take_item};
use crate::data::{Ask, Slot};
use crate::queries::pocket::count;
use crate::resources::{Inventory, OpenShop, Screen, Toast, UiHandles, Vigor};
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;

pub fn handle_the_keys(
    screen_state: Res<State<Screen>>,
    actions: Res<Actions<Ask>>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
) {
    let screen = screen_state.current;
    let escape = actions.just_pressed(Ask::Pause);
    let pocket = actions.just_pressed(Ask::Pocket);
    let quit = actions.just_pressed(Ask::Quit);
    match screen {
        Screen::Title => {
            if escape {
                next.pending = Some(Screen::Playing);
            }
        }
        Screen::Playing => {
            if escape {
                next.pending = Some(Screen::Paused);
            } else if pocket {
                next.pending = Some(Screen::Inventory);
            }
        }
        Screen::Paused => {
            if escape {
                next.pending = Some(Screen::Playing);
            }
        }
        Screen::Inventory => {
            if escape || pocket {
                next.pending = Some(Screen::Playing);
            }
        }
        Screen::Shop => {
            if escape {
                next.pending = Some(Screen::Playing);
            }
        }
        Screen::Talking => {}
    }
    if quit {
        window.should_exit = true;
    }
}

pub fn pick_the_slot(
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    mut inventory: ResMut<Inventory>,
) {
    let hand_next = actions.just_pressed(Ask::NextTool);
    if hand_next || actions.just_pressed(Ask::PreviousTool) {
        let step = if hand_next { 1 } else { -1 };
        let next = (inventory.slot as i32 + step).rem_euclid(Slot::ALL.len() as i32) as usize;
        select_slot(&mut inventory, next);
        return;
    }
    for (index, ask) in Ask::SLOTS.iter().enumerate() {
        if actions.just_pressed(*ask) {
            select_slot(&mut inventory, index);
            return;
        }
    }
    let wheel = input.mouse.wheel_delta.y;
    if wheel.abs() > 0.01 {
        let step = if wheel > 0.0 { -1 } else { 1 };
        let next = (inventory.slot as i32 + step).rem_euclid(Slot::ALL.len() as i32) as usize;
        select_slot(&mut inventory, next);
    }
}

pub fn pause_buttons(
    ui_handles: Res<UiHandles>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    let handles = ui_handles.clone();
    if let Some(resume) = handles.resume_button
        && ui_clicked(&mut buttons, resume)
    {
        next.pending = Some(Screen::Playing);
    }
    if let Some(menu) = handles.menu_button
        && ui_clicked(&mut buttons, menu)
    {
        next.pending = Some(Screen::Title);
    }
}

pub fn title_buttons(
    ui_handles: Res<UiHandles>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    let handles = ui_handles.clone();
    if let Some(play) = handles.title_play
        && ui_clicked(&mut buttons, play)
    {
        next.pending = Some(Screen::Playing);
    }
    if let Some(quit) = handles.title_quit
        && ui_clicked(&mut buttons, quit)
    {
        window.should_exit = true;
    }
}

pub fn shop_buttons(
    open_shop: Res<OpenShop>,
    ui_handles: Res<UiHandles>,
    mut next: ResMut<NextState<Screen>>,
    mut inventory_state: ResMut<Inventory>,
    mut toast_state: ResMut<Toast>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    let kind = open_shop.kind;
    let handles = ui_handles.clone();
    let Some(panel) = handles.shop_panels.get(kind.index()) else {
        return;
    };
    if let Some(leave) = panel.close_button
        && ui_clicked(&mut buttons, leave)
    {
        next.pending = Some(Screen::Playing);
        return;
    }
    let wares = kind.wares();
    for row in &panel.rows {
        let Some(button) = row.button else {
            continue;
        };
        if !ui_clicked(&mut buttons, button) {
            continue;
        }
        let Some(ware) = wares.get(row.ware) else {
            continue;
        };
        let mut inventory = std::mem::take(&mut *inventory_state);
        let mut toast = std::mem::take(&mut *toast_state);
        if ware.sells {
            if inventory.coins >= ware.price {
                inventory.coins -= ware.price;
                add_item(&mut inventory, ware.item, 1);
                show_toast(&mut toast, format!("Bought {}", ware.item.name()));
            } else {
                show_toast(&mut toast, "Not enough coins");
            }
        } else if take_item(&mut inventory, ware.item, 1) {
            inventory.coins += ware.price;
            show_toast(
                &mut toast,
                format!("Sold {} for {}", ware.item.name(), ware.price),
            );
        } else {
            show_toast(
                &mut toast,
                format!("No {} to sell", ware.item.name().to_lowercase()),
            );
        }
        *inventory_state = inventory;
        *toast_state = toast;
    }
}

pub fn eat_from_the_pocket(
    ui_handles: Res<UiHandles>,
    mut vigor: ResMut<Vigor>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    let handles = ui_handles.clone();
    let mut eaten = None;
    for meal in &handles.meal_buttons {
        if ui_clicked(&mut buttons, meal.button) {
            eaten = Some(meal.item);
            break;
        }
    }
    let Some(item) = eaten else {
        return;
    };
    let stamina = vigor.stamina;
    if count(&inventory, item) == 0 {
        show_toast(
            &mut toast,
            format!("No {} in the pocket", item.name().to_lowercase()),
        );
        return;
    }
    if stamina >= theme::MAX_STAMINA {
        show_toast(&mut toast, "Not hungry yet");
        return;
    }
    take_item(&mut inventory, item, 1);
    let gained = {
        let vigor = &mut *vigor;
        let before = vigor.stamina;
        vigor.stamina = (before + item.food()).min(theme::MAX_STAMINA);
        vigor.stamina - before
    };
    show_toast(
        &mut toast,
        format!(
            "Ate the {} for {} stamina",
            item.name().to_lowercase(),
            gained.ceil() as u32
        ),
    );
}
