use crate::data::{Errand, Item, Slot, Tool};
use crate::resources::{Inventory, Quests};
use crate::theme;

pub fn count(inventory: &Inventory, item: Item) -> u32 {
    inventory.counts[item.index()]
}

pub fn has_tool(inventory: &Inventory, tool: Tool) -> bool {
    inventory.tools[tool.index()]
}

pub fn level_of(inventory: &Inventory, tool: Tool) -> u32 {
    inventory.levels[tool.index()]
}

pub fn effort_of(inventory: &Inventory, tool: Tool) -> f32 {
    let discount = theme::TOOL_LEVEL_DISCOUNT * level_of(inventory, tool) as f32;
    tool.effort() * (1.0 - discount).max(0.4)
}

pub fn can_temper(inventory: &Inventory) -> Option<Tool> {
    let Slot::Tool(tool) = selected(inventory) else {
        return None;
    };
    let level = level_of(inventory, tool);
    (level < theme::MAX_TOOL_LEVEL
        && count(inventory, Item::Ore) >= theme::TEMPER_ORE
        && inventory.coins >= theme::TEMPER_COINS)
        .then_some(tool)
}

pub fn selected(inventory: &Inventory) -> Slot {
    Slot::ALL[inventory.slot % Slot::ALL.len()]
}

pub fn is_usable(inventory: &Inventory, slot: Slot) -> bool {
    match slot {
        Slot::Tool(tool) => has_tool(inventory, tool),
        Slot::Seed(kind) => count(inventory, kind.seed()) > 0,
    }
}

pub fn hand_text(inventory: &Inventory) -> String {
    match selected(inventory) {
        Slot::Tool(tool) => {
            let level = level_of(inventory, tool);
            if level > 0 {
                format!("{} +{}: {}", tool.name(), level, tool.hint())
            } else {
                format!("{}: {}", tool.name(), tool.hint())
            }
        }
        Slot::Seed(kind) => format!("{} x{}", kind.seed().name(), count(inventory, kind.seed())),
    }
}

pub fn errand_progress(inventory: &Inventory, errand: Errand) -> (u32, u32) {
    let (item, wanted) = errand.wants();
    (count(inventory, item).min(wanted), wanted)
}

pub fn is_done(quests: &Quests, errand: Errand) -> bool {
    quests.done[errand.index()]
}

pub fn can_deliver(inventory: &Inventory, quests: &Quests, errand: Errand) -> bool {
    let (item, wanted) = errand.wants();
    !is_done(quests, errand) && count(inventory, item) >= wanted
}
