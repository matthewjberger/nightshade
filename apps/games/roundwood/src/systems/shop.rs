use crate::commands::pocket::{sell_the_pocket, show_toast};
use crate::data::Ask;
use crate::resources::{Hud, Pocket, Screen, Store, Toast};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;

pub fn keep_the_store(
    hud: Res<Hud>,
    retained_ui_frame: Res<RetainedUiFrame>,
    mut pocket: ResMut<Pocket>,
    mut store: ResMut<Store>,
    mut toast: ResMut<Toast>,
    actions: Res<Actions<Ask>>,
    mut state_next: ResMut<NextState<Screen>>,
) {
    let hud = *hud;
    let clicked: Vec<Entity> = ui_button_clicks(&retained_ui_frame).collect();
    let mut sell = false;
    let mut leave = actions.just_pressed(Ask::Leave);
    for entity in clicked {
        if entity == hud.sell_button {
            sell = true;
        } else if entity == hud.leave_button {
            leave = true;
        }
    }
    if sell {
        let (goods, bells) = sell_the_pocket(&mut pocket);
        store.sold += goods;
        let line = if goods == 0 {
            "Nothing in your pockets today.".to_string()
        } else {
            format!("Sold {goods} for {bells} bells.")
        };
        show_toast(&mut toast, line);
    }
    if leave {
        store.keeper = None;
        state_next.pending = Some(Screen::Playing);
    }
}
