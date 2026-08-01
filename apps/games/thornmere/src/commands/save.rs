use crate::data::save::Save;
use crate::resources::{QuestState, Stack, ThornmereResources};

pub fn restore(resources: &mut ThornmereResources, save: &Save) {
    resources.purse = save.purse;
    for stack in resources.bag.slots.iter_mut() {
        *stack = None;
    }
    for stored in save.bag.iter() {
        if stored.item >= crate::data::items::ITEM_COUNT || stored.count == 0 {
            continue;
        }
        if let Some(slot) = resources.bag.slots.get_mut(stored.slot) {
            *slot = Some(Stack {
                item: stored.item,
                count: stored.count,
            });
        }
    }
    for (place, worn) in resources.gear.worn.iter_mut().enumerate() {
        *worn = save
            .worn
            .get(place)
            .copied()
            .flatten()
            .filter(|item| *item < crate::data::items::ITEM_COUNT);
    }
    resources.gear.change += 1;
    for (quest, done) in save.finished.iter().enumerate() {
        if let Some(slot) = resources.quests.finished.get_mut(quest) {
            *slot = *done;
        }
    }
    resources.quests.active = save
        .active
        .iter()
        .filter(|state| state.quest < crate::data::QUESTS.len())
        .map(|state| QuestState {
            quest: state.quest,
            progress: state.progress,
            complete: state.complete,
        })
        .collect();
    for (place, found) in save.discovered.iter().enumerate() {
        if let Some(slot) = resources.clock.discovered.get_mut(place) {
            *slot = *found;
        }
    }
}
