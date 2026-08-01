use crate::data::{
    CALLS, CARD_ANSWERS, CARD_NAMES, CARD_SLOTS, KEPT_HIM_IN, MINUTES_PER_DECISION,
    MINUTES_PER_REPLY, REPLY_SLOTS, SENT_HIM_OUT,
};
use crate::resources::{CallPhase, NightState, NightwireResources, Screen};

const MAX_KEPT_LINES: usize = 240;

fn speak(night: &mut NightState, lines: &[&str]) {
    for line in lines {
        night.queue.push_back((*line).to_string());
    }
}

fn keep_recent(night: &mut NightState) {
    while night.lines.len() > MAX_KEPT_LINES {
        night.lines.remove(0);
    }
}

pub fn begin_shift(resources: &mut NightwireResources) {
    resources.night = NightState {
        screen: Screen::Shift,
        ..Default::default()
    };
    open_call(resources, 0);
    resources.controls_dirty = true;
}

pub fn open_call(resources: &mut NightwireResources, index: usize) {
    let night = &mut resources.night;
    night.call = index;
    night.phase = CallPhase::Speaking;
    night.pending_phase = Some(CallPhase::Asking);
    night.spent = [false; REPLY_SLOTS];
    night.raised = [false; CARD_SLOTS];
    night.revealed = 0;
    night.reveal_timer = 0.0;
    if index > 0 {
        night.lines.push(String::new());
    }
    let deputy_out = night.deputy_out;
    speak(night, CALLS[index].opening);
    if index == 2 {
        let aside = if deputy_out {
            SENT_HIM_OUT
        } else {
            KEPT_HIM_IN
        };
        speak(night, aside);
    }
    keep_recent(night);
    resources.rows_dirty = true;
    resources.controls_dirty = true;
}

pub fn answer_reply(resources: &mut NightwireResources, index: usize) {
    let call = resources.night.call;
    let Some(reply) = CALLS[call].replies.get(index) else {
        return;
    };
    let night = &mut resources.night;
    night.spent[index] = true;
    night.minutes += MINUTES_PER_REPLY;
    night.lines.push(format!("YOU: {}", reply.prompt));
    speak(night, reply.lines);
    if let Some(card) = reply.gives_card {
        night.cards[card] = true;
    }
    night.pending_phase = Some(if reply.closes_call {
        if CALLS[call].can_dispatch {
            CallPhase::Deciding
        } else {
            CallPhase::Closed
        }
    } else {
        CallPhase::Asking
    });
    night.phase = CallPhase::Speaking;
    keep_recent(night);
    resources.rows_dirty = true;
    resources.controls_dirty = true;
}

pub fn raise_card(resources: &mut NightwireResources, index: usize) {
    let call = resources.night.call;
    let answer = CARD_ANSWERS
        .iter()
        .find(|answer| answer.call == call && answer.card == index);
    let night = &mut resources.night;
    night.raised[index] = true;
    night.minutes += MINUTES_PER_REPLY;
    night
        .lines
        .push(format!("YOU: About {}.", CARD_NAMES[index]));
    match answer {
        Some(answer) => {
            speak(night, answer.lines);
            if let Some(card) = answer.gives_card {
                night.cards[card] = true;
                if card == 1 {
                    night.knows_truck_empty = true;
                }
            }
        }
        None => speak(night, CALLS[call].deflection),
    }
    night.pending_phase = Some(CallPhase::Asking);
    night.phase = CallPhase::Speaking;
    keep_recent(night);
    resources.rows_dirty = true;
    resources.controls_dirty = true;
}

pub fn decide(resources: &mut NightwireResources, send: bool) {
    let call = resources.night.call;
    let night = &mut resources.night;
    night.minutes += MINUTES_PER_DECISION;
    if send && night.deputy_out {
        speak(night, CALLS[call].busy_lines);
    } else if send {
        night.deputy_out = true;
        if call == 1 {
            night.sent_to_water = true;
        }
        speak(night, CALLS[call].dispatch_lines);
    } else {
        speak(night, CALLS[call].hold_lines);
    }
    night.pending_phase = Some(CallPhase::Closed);
    night.phase = CallPhase::Speaking;
    keep_recent(night);
    resources.rows_dirty = true;
    resources.controls_dirty = true;
}

pub fn advance(resources: &mut NightwireResources) {
    let next = resources.night.call + 1;
    if next < CALLS.len() {
        open_call(resources, next);
    } else {
        resources.night.screen = Screen::Over;
        resources.controls_dirty = true;
    }
}

pub fn return_to_title(resources: &mut NightwireResources) {
    resources.night = NightState::default();
    resources.rows_dirty = true;
    resources.controls_dirty = true;
}
