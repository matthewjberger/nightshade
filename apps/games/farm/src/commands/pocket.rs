use crate::data::Item;
use crate::resources::{AfterTalk, Dialogue, Inventory, Toast};

pub fn add_item(inventory: &mut Inventory, item: Item, amount: u32) {
    inventory.counts[item.index()] += amount;
}

pub fn take_item(inventory: &mut Inventory, item: Item, amount: u32) -> bool {
    let slot = &mut inventory.counts[item.index()];
    if *slot < amount {
        return false;
    }
    *slot -= amount;
    true
}

pub fn select_slot(inventory: &mut Inventory, slot: usize) {
    inventory.slot = slot % crate::data::Slot::ALL.len();
}

pub fn show_toast(toast: &mut Toast, text: impl Into<String>) {
    toast.text = text.into();
    toast.remaining = crate::theme::TOAST_SECONDS;
}

pub fn start_dialogue(
    dialogue: &mut Dialogue,
    speaker: &str,
    lines: Vec<String>,
    after: AfterTalk,
) {
    dialogue.speaker = speaker.to_string();
    dialogue.lines = lines;
    dialogue.line = 0;
    dialogue.revealed = 0.0;
    dialogue.after = after;
}
