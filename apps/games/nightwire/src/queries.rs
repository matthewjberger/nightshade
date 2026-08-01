use crate::data::{CALLS, CARD_NAMES, CARD_SLOTS, LINE_SLOTS, REPLY_SLOTS, SUMMARY_SLOTS};
use crate::resources::{CallPhase, NightState};

const CARET: &str = "_";

pub fn clock_text(night: &NightState) -> String {
    let hour = (night.minutes / 60) % 24;
    let minute = night.minutes % 60;
    format!("{hour:02}:{minute:02}   Ashgrove Valley   night line")
}

pub fn caller_text(night: &NightState) -> String {
    match night.phase {
        CallPhase::Closed => "LINE CLEAR".to_string(),
        _ => CALLS[night.call].caller.to_string(),
    }
}

pub fn transcript_rows(night: &NightState) -> Vec<String> {
    let mut shown: Vec<String> = night.lines.clone();
    if let Some(head) = night.queue.front() {
        shown.push(head.chars().take(night.revealed).collect());
    }
    if night.queue.is_empty() {
        shown.push(if night.caret_on {
            CARET.to_string()
        } else {
            String::new()
        });
    } else if night.caret_on
        && let Some(last) = shown.last_mut()
    {
        last.push_str(CARET);
    }

    let start = shown.len().saturating_sub(LINE_SLOTS);
    let mut rows: Vec<String> = shown[start..].to_vec();
    while rows.len() < LINE_SLOTS {
        rows.insert(0, String::new());
    }
    rows
}

pub fn reply_text(night: &NightState, index: usize) -> String {
    if night.phase != CallPhase::Asking || index >= REPLY_SLOTS {
        return String::new();
    }
    match CALLS[night.call].replies.get(index) {
        Some(reply) if !night.spent[index] => reply.prompt.to_string(),
        _ => String::new(),
    }
}

pub fn card_text(night: &NightState, index: usize) -> String {
    if index >= CARD_SLOTS || !night.cards[index] {
        return String::new();
    }
    if night.phase == CallPhase::Asking && !night.raised[index] {
        format!("Raise {}", CARD_NAMES[index])
    } else {
        CARD_NAMES[index].to_string()
    }
}

pub fn card_is_open(night: &NightState, index: usize) -> bool {
    night.cards[index] && night.phase == CallPhase::Asking && !night.raised[index]
}

pub fn notebook_note(night: &NightState) -> String {
    if night.cards.iter().any(|found| *found) {
        "Click a name to raise it with the caller.".to_string()
    } else {
        "Nothing written down yet.".to_string()
    }
}

pub fn glass_label(glass_on: bool) -> &'static str {
    if glass_on {
        "Glass on   F1"
    } else {
        "Glass off   F1"
    }
}

pub fn advance_text(night: &NightState) -> String {
    if night.call + 1 < CALLS.len() {
        "Take the next call".to_string()
    } else {
        "End the shift".to_string()
    }
}

pub fn summary_rows(night: &NightState) -> Vec<String> {
    let hour = (night.minutes / 60) % 24;
    let minute = night.minutes % 60;
    let found = night.cards.iter().filter(|found| **found).count();
    let deputy = if night.sent_to_water {
        "The deputy went down to the water. The log does not say what he found."
    } else if night.deputy_out {
        "The deputy is somewhere on the Ridge Road. He has not called in."
    } else {
        "The deputy sat at the station all night, as you told him to."
    };
    let judgement = if night.knows_truck_empty && !night.deputy_out {
        "You knew the truck was empty. You sent nobody."
    } else if night.knows_truck_empty {
        "You knew the truck was empty, and you did what you could."
    } else {
        "You never asked Rourke about the truck on the shoulder."
    };
    let mut rows = vec![
        format!("Shift closed at {hour:02}:{minute:02}."),
        format!("Three calls taken. {found} of 3 names written down."),
        deputy.to_string(),
        judgement.to_string(),
        String::new(),
        "Marion Ash is still out. The valley keeps the rest.".to_string(),
    ];
    while rows.len() < SUMMARY_SLOTS {
        rows.push(String::new());
    }
    rows.truncate(SUMMARY_SLOTS);
    rows
}
