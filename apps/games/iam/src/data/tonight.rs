use crate::data::script::Fact;
use crate::resources::AppResources;

pub const ROWS: usize = 4;

pub const REVEALED: &str = "that is one for tonight";
pub const TICKED: &str = "that is one of them";
pub const FINISHED: &str = "tonight is done";

pub const NOTHING_LEFT: &str = "that is all of it. you can sign off now";

pub const FIRST_ROW: f32 = 46.0;
pub const ROW_PITCH: f32 = 30.0;

pub const UNKNOWN: &str = "????";

pub struct Gate {
    pub fact: Fact,
    line: &'static str,
    known_by: &'static [Fact],
}

const fn gate(fact: Fact, line: &'static str, known_by: &'static [Fact]) -> Gate {
    Gate {
        fact,
        line,
        known_by,
    }
}

pub const LINES: &[Gate] = &[
    gate(
        "posted_in_a_room",
        "say something in a room",
        &["opened_a_server"],
    ),
    gate(
        "saw_the_intranet",
        "have a look at the intranet",
        &["told_about_the_browser"],
    ),
    gate(
        "heard_the_hour",
        "ask somebody what their row says",
        &["saw_my_own_row"],
    ),
    gate(
        "saw_the_server_log",
        "read the server's own session log",
        &["told_about_the_log"],
    ),
    gate(
        crate::data::lights::COUNTED,
        "finish a board of lights out",
        &[crate::data::lights::DOWNLOADED],
    ),
    gate(
        "counted_the_difference",
        "run the cross-check on the two counts",
        &["heard_the_count"],
    ),
    gate(
        "opened_an_account",
        "get into one of the old accounts",
        &["knows_the_rollover"],
    ),
    gate(
        "read_a_leftover",
        "read something they left behind",
        &["opened_an_account"],
    ),
];

pub fn heading(resources: &AppResources) -> &'static str {
    resources
        .script
        .nights
        .get(resources.session.night)
        .map(|night| night.day)
        .unwrap_or_default()
}

pub fn left_to_do(resources: &AppResources) -> Vec<(Fact, &'static str, bool)> {
    let Some(night) = resources.script.nights.get(resources.session.night) else {
        return Vec::new();
    };
    let facts = &resources.session.facts;
    night
        .condition
        .requires
        .iter()
        .filter_map(|wanted| {
            LINES.iter().find(|gate| gate.fact == *wanted).map(|gate| {
                let done = facts.contains(&gate.fact);
                let known = done || gate.known_by.iter().all(|fact| facts.contains(fact));
                (gate.fact, if known { gate.line } else { UNKNOWN }, done)
            })
        })
        .collect()
}
