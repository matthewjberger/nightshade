use crate::data::save::{Save, SavedQuest, SavedStack};
use crate::resources::ThornmereResources;

pub fn gather(
    resources: &ThornmereResources,
    level: u32,
    experience: u32,
    kills: u32,
    spot: [f32; 3],
    hour: f32,
) -> Save {
    Save {
        name: resources.player_name.clone(),
        calling: resources.class,
        level,
        experience,
        kills,
        purse: resources.purse,
        hour,
        spot,
        bag: resources
            .bag
            .slots
            .iter()
            .enumerate()
            .filter_map(|(slot, stack)| {
                stack.map(|stack| SavedStack {
                    slot,
                    item: stack.item,
                    count: stack.count,
                })
            })
            .collect(),
        worn: resources.gear.worn.to_vec(),
        finished: resources.quests.finished.clone(),
        active: resources
            .quests
            .active
            .iter()
            .map(|state| SavedQuest {
                quest: state.quest,
                progress: state.progress,
                complete: state.complete,
            })
            .collect(),
        discovered: resources.clock.discovered.clone(),
    }
}
