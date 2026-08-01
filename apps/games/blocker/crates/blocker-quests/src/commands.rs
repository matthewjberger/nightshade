use crate::data::quests::{Objective, QUESTS};
use crate::data::trade::STOCK;
use crate::queries::level_from_xp;
use crate::resources::Journal;
use blocker_blocks::queries::item_value;
use blocker_pocket::commands::{inventory_add, inventory_take, inventory_take_selected};
use blocker_pocket::queries::{inventory_count, selected_stack};
use blocker_pocket::resources::Inventory;
use blocker_screen::data::notice::Notice;
use nightshade::ecs::system_param::EventWriter;

pub fn accept_quest(
    inventory: &mut Inventory,
    journal: &mut Journal,
    notices: &mut EventWriter<'_, Notice>,
    quest_index: usize,
) {
    if journal.active.contains(&quest_index) || journal.finished.contains(&quest_index) {
        return;
    }
    let quest = &QUESTS[quest_index];
    if let Objective::Slay { beast, .. } = quest.objective {
        journal.forget_slain(beast);
    }
    journal.active.push(quest_index);
    notices.send(Notice(format!("New quest: {}", quest.title)));
    for (item, count) in quest.loan {
        let leftover = inventory_add(inventory, *item, *count);
        if leftover < *count {
            notices.send(Notice(format!(
                "{} gave you {} x{}",
                crate::data::trade::GIVERS[quest.giver].name,
                item.name(),
                count - leftover
            )));
        }
    }
}

pub fn buy_good(
    inventory: &mut Inventory,
    journal: &mut Journal,
    notices: &mut EventWriter<'_, Notice>,
    index: usize,
) {
    let Some(good) = STOCK.get(index) else {
        return;
    };
    if journal.coin < good.price {
        notices.send(Notice("Not enough coin for that".into()));
        return;
    }
    let leftover = inventory_add(inventory, good.item, good.count);
    if leftover == good.count {
        notices.send(Notice("No room in your pockets".into()));
        return;
    }
    journal.coin -= good.price;
    notices.send(Notice(format!(
        "Bought {} x{} (-{} coin)",
        good.item.name(),
        good.count - leftover,
        good.price
    )));
}

pub fn sell_selected(
    inventory: &mut Inventory,
    journal: &mut Journal,
    notices: &mut EventWriter<'_, Notice>,
) {
    let Some(stack) = selected_stack(inventory) else {
        notices.send(Notice("Hold a stack to sell it".into()));
        return;
    };
    let count = stack.count;
    let gained = item_value(stack.item) * count;
    inventory_take_selected(inventory, count);
    journal.coin += gained;
    notices.send(Notice(format!(
        "Sold {} x{} (+{} coin)",
        stack.item.name(),
        count,
        gained
    )));
}

fn award_xp(journal: &mut Journal, amount: u32) -> Option<u32> {
    let before = level_from_xp(journal.xp);
    journal.xp += amount;
    let after = level_from_xp(journal.xp);
    (after > before).then_some(after)
}

pub fn reward(
    journal: &mut Journal,
    notices: &mut EventWriter<'_, Notice>,
    renown: u32,
    coin: u32,
    xp: u32,
    deed: String,
) {
    journal.renown += renown;
    journal.coin += coin;
    let message = match award_xp(journal, xp) {
        Some(level) => format!("{deed}   You reached level {level}"),
        None => deed,
    };
    notices.send(Notice(message));
}

pub fn complete_quest(
    inventory: &mut Inventory,
    journal: &mut Journal,
    notices: &mut EventWriter<'_, Notice>,
    quest_index: usize,
) {
    let quest = &QUESTS[quest_index];
    if let Objective::Gather { item, count } = &quest.objective {
        if inventory_count(inventory, *item) < *count {
            return;
        }
        inventory_take(inventory, *item, *count);
    }
    inventory_add(inventory, quest.reward, quest.reward_count);
    journal.active.retain(|entry| *entry != quest_index);
    journal.finished.push(quest_index);
    if let Some(next) = quest.next
        && matches!(QUESTS[next].objective, Objective::Report)
        && !journal.finished.contains(&next)
        && !journal.active.contains(&next)
    {
        journal.active.push(next);
        notices.send(Notice(format!("New quest: {}", QUESTS[next].title)));
    }
    reward(
        journal,
        notices,
        quest.renown,
        quest.renown,
        quest.renown,
        format!("Quest done: {} (+{} xp)", quest.title, quest.renown),
    );
}
