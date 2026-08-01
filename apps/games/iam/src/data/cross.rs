use crate::data::script::Fact;

pub struct Pairing {
    pub source: &'static str,
    pub claim: &'static str,
    pub record: &'static str,
    pub agrees: bool,
    pub reading: &'static str,
    pub requires: &'static [Fact],
    pub asserts: &'static [Fact],
}

pub const PAIRINGS: &[Pairing] = &[
    Pairing {
        source: "halloran, last night",
        claim: "my row says connected 3:07 AM",
        record: "session log: 3:07 AM, every entry, 8,815 of them",
        agrees: true,
        reading: "they agree. nobody has connected at any other minute since the log was opened.",
        requires: &["heard_the_hour", "read_the_logs"],
        asserts: &["matched_the_minute"],
    },
    Pairing {
        source: "the signup form, field 03",
        claim: "connected: 03:07",
        record: "session log: 8,815 rows, all of them 3:07 AM",
        agrees: true,
        reading: "the field is a constant. nothing on this network has ever recorded when anybody arrived.",
        requires: &["knows_the_form_is_constant", "read_the_logs"],
        asserts: &["entered_the_hour"],
    },
    Pairing {
        source: "countbot, in the toolbox",
        claim: "registered users: 8814",
        record: "session log: 8,815 sessions opened",
        agrees: false,
        reading: "one apart. the room is counting people. the machine is counting sessions.",
        requires: &["knows_the_count_was_wrong"],
        asserts: &["counted_the_difference"],
    },
    Pairing {
        source: "qwrtyu, every friday for six years",
        claim: "friday count: 8815",
        record: "helpdesk: seven accounts opened this week. nobody hired.",
        agrees: false,
        reading: "seven a week and the total has not moved in five years. they are not new accounts.",
        requires: &["helped_qwrtyu"],
        asserts: &["read_the_tally"],
    },
    Pairing {
        source: "plum, in the lounge",
        claim: "i signed off on tuesday",
        record: "session log: no entry has a close time",
        agrees: false,
        reading: "she closed it. the machine did not write that down. it has never written that down for anybody.",
        requires: &["heard_she_signed_off", "knows_sessions_never_close"],
        asserts: &["matched_the_absence"],
    },
    Pairing {
        source: "helpdesk ticket 0001, raised 2016",
        claim: "configure the session close handler",
        record: "ticket 0001: open. assigned to unset.",
        agrees: true,
        reading: "it was not left broken. it was never turned on, and the ticket saying so is the oldest thing here.",
        requires: &["saw_the_first_ticket", "knows_sessions_never_close"],
        asserts: &["saw_the_server_log"],
    },
    Pairing {
        source: "the lounge topic",
        claim: "open since 2016. nobody has closed it",
        record: "server archive: nothing added since 2016",
        agrees: true,
        reading: "both true, and the room has five years of posts in it that neither of them counts.",
        requires: &["saw_a_missing_line"],
        asserts: &["knows_the_archive_lies"],
    },
    Pairing {
        source: "sundial, in the records server",
        claim: "i posted eleven lines about this",
        record: "server: four lines retained. dropped: 0 rows.",
        agrees: false,
        reading: "the seven it dropped were the seven with a question in them. it kept every statement.",
        requires: &["knows_which_lines_go"],
        asserts: &["knows_why_questions_go"],
    },
    Pairing {
        source: "wordcountbot, in the toolbox",
        claim: "transcript, newuser_41: 4212",
        record: "transcript, newuser_41_: 4198",
        agrees: false,
        reading: "fourteen words apart. everything i have said this week is in one of them.",
        requires: &["knows_the_copy_is_close"],
        asserts: &["knows_how_to_stay"],
    },
    Pairing {
        source: "my start date, on the intranet",
        claim: "newuser_41 started 11 nov 2024",
        record: "session log: newuser_41 opened 05 nov 2024, 3:07 AM",
        agrees: false,
        reading: "the account is six days older than the job. it was not told about me. it filed me.",
        requires: &["saw_my_own_record"],
        asserts: &["knows_the_account_comes_first"],
    },
    Pairing {
        source: "the newstarters room",
        claim: "412 hellos. no replies.",
        record: "archive: posts in archive: 0",
        agrees: true,
        reading: "nobody answered any of them, and the machine did not keep them either.",
        requires: &["saw_the_join_repeat", "knows_the_archive_lies"],
        asserts: &["knows_there_is_no_first"],
    },
    Pairing {
        source: "the offline list",
        claim: "newuser_41 - offline",
        record: "session log: newuser_41, 3:07 AM, open",
        agrees: false,
        reading: "the server has me signed off. the machine has me signed on. i am typing this.",
        requires: &["saw_own_session"],
        asserts: &["matched_myself"],
    },
];

pub fn readable(facts: &[Fact]) -> Vec<usize> {
    PAIRINGS
        .iter()
        .enumerate()
        .filter(|(_, pairing)| pairing.requires.iter().all(|fact| facts.contains(fact)))
        .map(|(index, _)| index)
        .collect()
}
