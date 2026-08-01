use crate::commands::dungeon::give_dagger;
use crate::commands::pocket::{add_item, show_toast, take_item};
use crate::data::{Ask, Item};
use crate::queries::talk::{current, fully_shown, is_last};
use crate::resources::{AfterTalk, Dialogue, Hero, Inventory, OpenShop, Quests, Screen, Toast};
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;

pub fn hold_the_conversation(
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    mut dialogue_state: ResMut<Dialogue>,
    mut hero: ResMut<Hero>,
    mut inventory: ResMut<Inventory>,
    mut quests: ResMut<Quests>,
    mut toast: ResMut<Toast>,
    mut open_shop: ResMut<OpenShop>,
    mut next: ResMut<NextState<Screen>>,
) {
    let advance = actions.just_pressed(Ask::Advance);
    let delta_time = time.delta_time;

    let outcome = {
        let dialogue = &mut *dialogue_state;
        dialogue.revealed += delta_time * theme::TYPEWRITER_RATE;
        if !advance {
            None
        } else if !fully_shown(dialogue) {
            dialogue.revealed = current(dialogue).chars().count() as f32;
            None
        } else if !is_last(dialogue) {
            dialogue.line += 1;
            dialogue.revealed = 0.0;
            None
        } else {
            Some(dialogue.after)
        }
    };

    let Some(after) = outcome else {
        return;
    };
    match after {
        AfterTalk::Nothing => next.pending = Some(Screen::Playing),
        AfterTalk::GiveDagger => {
            give_dagger(&mut hero, &mut toast);
            next.pending = Some(Screen::Playing);
        }
        AfterTalk::Temper(tool) => {
            let tempered = {
                let inventory = &mut *inventory;
                let level = inventory.levels[tool.index()];
                if level < theme::MAX_TOOL_LEVEL
                    && inventory.coins >= theme::TEMPER_COINS
                    && take_item(inventory, Item::Ore, theme::TEMPER_ORE)
                {
                    inventory.coins -= theme::TEMPER_COINS;
                    inventory.levels[tool.index()] = level + 1;
                    Some(level + 1)
                } else {
                    None
                }
            };
            match tempered {
                Some(level) => show_toast(
                    &mut toast,
                    format!(
                        "Wren tempered the {} to +{}",
                        tool.name().to_lowercase(),
                        level
                    ),
                ),
                None => show_toast(&mut toast, "Wren needs more ore and coin"),
            }
            next.pending = Some(Screen::Playing);
        }
        AfterTalk::OpenShop(kind) => {
            open_shop.kind = kind;
            next.pending = Some(Screen::Shop);
        }
        AfterTalk::Deliver(errand) => {
            let (item, wanted) = errand.wants();
            {
                let inventory = &mut *inventory;
                if take_item(inventory, item, wanted) {
                    inventory.coins += errand.reward();
                }
                if errand == crate::data::Errand::Wheat {
                    add_item(inventory, crate::data::Item::Bread, 2);
                }
            }
            quests.done[errand.index()] = true;
            show_toast(
                &mut toast,
                format!("{} coins from {}", errand.reward(), errand.asker().name()),
            );
            next.pending = Some(Screen::Playing);
        }
    }
}
