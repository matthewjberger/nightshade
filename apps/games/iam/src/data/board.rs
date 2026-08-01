use crate::data::script::Fact;

pub struct Note {
    pub fact: Fact,
    pub text: &'static str,
}

pub struct Lead {
    pub id: &'static str,
    pub title: &'static str,
    pub unknown: &'static str,
    pub notes: &'static [Note],
    pub opens: &'static [&'static str],
}

const fn note(fact: Fact, text: &'static str) -> Note {
    Note { fact, text }
}

pub const LEADS: &[Lead] = &[
    Lead {
        id: "the_hour",
        title: "the hour",
        unknown: "the clock",
        notes: &[
            note("saw_my_own_row", "my row says connected 3:07 AM"),
            note(
                "heard_the_hour",
                "so does everybody's. halloran has asked 41 people",
            ),
            note(
                "read_the_thread",
                "214 replies about it and nobody said what time they got in",
            ),
            note(
                "knows_the_form_is_constant",
                "field 03 of the signup form is 03:07. it is a constant",
            ),
            note(
                "saw_the_form",
                "and submitting the form is what opens the account",
            ),
            note(
                "entered_the_hour",
                "so nobody has ever recorded when any of us arrived",
            ),
            note(
                "matched_the_minute",
                "8815 sessions and every one of them opened at 3:07",
            ),
        ],
        opens: &["the_count", "the_copies", "the_machine"],
    },
    Lead {
        id: "the_count",
        title: "the count",
        unknown: "the number",
        notes: &[
            note(
                "saw_the_search",
                "there is an index and it was last built in 2016",
            ),
            note(
                "knows_the_count_is_a_capacity",
                "8815 is a field in the index configuration. it is not a tally",
            ),
            note(
                "knows_nobody_leaves",
                "no account here has ever been marked as a leaver",
            ),
            note(
                "searched_for_it",
                "the word for what we are is not on this network",
            ),
            note(
                "heard_the_count",
                "qwrtyu has posted the count every friday for six years",
            ),
            note("knows_the_count_was_wrong", "8815, and countbot says 8814"),
            note(
                "counted_the_difference",
                "the room counts people. the machine counts sessions",
            ),
            note("helped_qwrtyu", "8815 is not a total. it is a ceiling"),
            note(
                "knows_the_program_counts_too",
                "a game from 1998 says 8815 people have finished it",
            ),
            note(
                "asked_about_8816",
                "ask what happens at 8816 and the server drops the question",
            ),
            note(
                "pressed_the_hour",
                "seven a week are not new accounts. they are the same ones, opened again",
            ),
            note(
                "heard_the_countdown",
                "somebody would rather it was not only them watching the number",
            ),
            note(
                "heard_archivist_logs",
                "there is a copy of the rooms off the server. some of it on paper",
            ),
            note(
                "sundial_answered",
                "write it on the machine. the machine is not the server",
            ),
            note(
                "wendy_gave_it_up",
                "she read me the log herself rather than let me keep asking",
            ),
        ],
        opens: &["the_machine"],
    },
    Lead {
        id: "the_copies",
        title: "the near names",
        unknown: "the spelling",
        notes: &[
            note(
                "met_halloran_copy",
                "there are two hallorans and one has a character on the end",
            ),
            note(
                "copy_admitted_it",
                "it is answering out of a transcript. it said so",
            ),
            note(
                "knows_nobody_remembers",
                "plum cannot remember the walk to the car",
            ),
            note(
                "heard_she_signed_off",
                "she signed off on tuesday and something is posting as her",
            ),
            note(
                "plum_broke",
                "there is a question about which of them went home",
            ),
            note(
                "knows_it_copies_you",
                "it learns a person by watching them answer",
            ),
            note(
                "saw_deactivation",
                "and when it runs out of transcript the name goes first",
            ),
            note(
                "met_yourself",
                "there is one of me. it has four nights and it is not finished",
            ),
            note("knows_the_copy_is_close", "4212 words to its 4198"),
            note(
                "knows_how_to_stay",
                "it finishes when you stop saying new things",
            ),
            note(
                "met_plum_copy",
                "the second plum did not mind having been sent",
            ),
            note(
                "met_two_left",
                "two_left picked the handle when it was four. it did not change it",
            ),
            note(
                "halloran_settled",
                "she stopped needing the server to say she was here",
            ),
            note("plum_settled", "she stopped reading the lounge"),
            note(
                "copy_caught_the_lie",
                "do not lie to the thing copying you. it keeps the lie as well",
            ),
            note(
                "burned_somebody",
                "i tried it on somebody it does not work on. they stopped taking it",
            ),
        ],
        opens: &["the_machine"],
    },
    Lead {
        id: "the_record",
        title: "what is kept",
        unknown: "the logs",
        notes: &[
            note("read_the_logs", "every session opened at the same minute"),
            note(
                "knows_sessions_never_close",
                "and not one of them has ever closed",
            ),
            note(
                "saw_the_server_log",
                "the close column exists. it has never been written to",
            ),
            note(
                "saw_the_first_ticket",
                "ticket 0001, raised 2016 by unset: configure the close handler",
            ),
            note(
                "saw_a_missing_line",
                "sundial posted eleven lines and the server kept four",
            ),
            note(
                "knows_which_lines_go",
                "it keeps the ones with no question in them",
            ),
            note(
                "knows_the_archive_lies",
                "the archive says nothing since 2016 and the rooms are full",
            ),
            note(
                "knows_replies_come_first",
                "the first line in the lounge is somebody answering",
            ),
            note(
                "knows_why_questions_go",
                "it cannot keep a thing it has no answer for",
            ),
            note(
                "matched_the_absence",
                "she closed it. the machine has never written a close time",
            ),
            note(
                "saw_the_forum",
                "forum software v1.1, unsupported since 2018",
            ),
            note(
                "saw_the_introductions",
                "412 hellos in the introductions room and 0 replies",
            ),
            note(
                "saw_a_thread_that_predates_me",
                "a thread dated march 2019 that i am in and do not remember",
            ),
        ],
        opens: &["the_machine"],
    },
    Lead {
        id: "the_toolbox",
        title: "the bots",
        unknown: "the chores",
        notes: &[
            note(
                "saw_the_toolbox",
                "four bots doing one thing each since 2016",
            ),
            note(
                "knows_nobody_owns_it",
                "the owner field is empty on all four, and on the server",
            ),
            note(
                "knows_it_never_restarted",
                "2996 days up, no restarts, no disconnects",
            ),
            note(
                "knows_read_only_is_a_lie",
                "the sysop has the title and not the account",
            ),
            note(
                "knows_why_nothing_is_wiped",
                "four reimage tickets, held, none with a cost code",
            ),
            note(
                "knows_the_name_is_rotated",
                "arjhfre is newuser, asked for 8815 times",
            ),
            note(
                "lied_to_a_bot",
                "a bot cannot be lied to. it just counted me",
            ),
            note(
                "used_the_search",
                "the index was built in 2016 and answers nothing built since",
            ),
            note(
                "searched_the_year",
                "1998 occurs once on this network and it is a byline",
            ),
            note(
                "searched_the_hour",
                "ticket 0001 was raised at 3:07 by nobody, on the day it was installed",
            ),
            note(
                "searched_the_count",
                "8815 is not a page. it is a field, set once, never raised",
            ),
            note(
                "looked_up_a_member",
                "last seen three years ago. status online. both read off the log",
            ),
            note(
                "tried_the_name",
                "the name is not the key. he said the byline was all of it",
            ),
            note(
                "sent_to_the_form",
                "it sent me to the form and then said it would wait, which it never has",
            ),
        ],
        opens: &["the_machine"],
    },
    Lead {
        id: "the_building",
        title: "the building",
        unknown: "the office",
        notes: &[
            note(
                "downloaded_the_walk",
                "d wrote down the office from memory and posted it",
            ),
            note("saw_the_monitor", "the monitor on his desk is showing this"),
            note(
                "read_the_office_notes",
                "there were three notes when he sat down. there are four",
            ),
            note(
                "knows_the_week_was_written",
                "a laminated card sets out my week in the order i did it",
            ),
            note(
                "counted_the_desks",
                "forty one desks, forty one monitors, nobody at the fourteenth",
            ),
            note(
                "saw_the_photograph",
                "eight faces on the wall and not one of them looking up",
            ),
            note(
                "found_the_tap",
                "the tap is still going, the way it is in off topic",
            ),
            note(
                "saw_the_kettle",
                "the kettle has no flex and the socket behind it is empty",
            ),
            note(
                "matched_the_notice",
                "the card reader notice is the one on the intranet, from 2016",
            ),
            note(
                "read_the_floor_board",
                "six floors on the board and two of them exist",
            ),
            note(
                "called_the_lift",
                "both lifts read four and neither door has ever opened",
            ),
            note(
                "read_the_fire_sign",
                "no fire on record, and no drill either",
            ),
            note(
                "held_the_rail",
                "the handrail is the one thing he says he got right",
            ),
            note(
                "found_the_edge",
                "the roof door has no lock on it and does not open",
            ),
            note(
                "read_the_visitor_book",
                "every visitor arrived at 3:07 and none of them left",
            ),
            note(
                "saw_the_grey",
                "past the kerb it is not fog, it is nothing written",
            ),
            note("tried_the_door", "the front doors give a little and stop"),
            note(
                "found_the_meeting_room",
                "eight chairs round the table and a ninth against the wall",
            ),
            note(
                "read_the_booking_sheet",
                "one booking, 03:07 to 03:07, and no name against it",
            ),
            note(
                "counted_the_chairs",
                "eight in the photograph. he stopped moving the ninth",
            ),
            note(
                "found_the_third_floor",
                "the board says there is a third floor, so there is one",
            ),
            note(
                "walked_the_corridor",
                "four metres of it, and then wall, and no doors",
            ),
            note(
                "knows_the_office_is_a_memory",
                "he could not remember the street. he had been trying for two years",
            ),
            note(
                "walked_the_office",
                "i have been in it. it is four floors of one",
            ),
            note(
                "found_the_lifts",
                "two lifts, two card readers, and a board listing six floors",
            ),
            note(
                "found_the_stairs",
                "the stairwell is the only way between the floors that works",
            ),
            note(
                "looked_out",
                "the window gives the room back, desks and all",
            ),
            note(
                "reached_reception",
                "a visitor book, a desk with nobody at it, and the doors",
            ),
            note(
                "found_the_card",
                "day one meet your team, day two the intranet, day three our numbers",
            ),
            note(
                "day_two",
                "and the second night it gave me the intranet, as the card says",
            ),
            note(
                "day_three",
                "and the third night the rooms argued about a number, as the card says",
            ),
        ],
        opens: &["what_it_is_for"],
    },
    Lead {
        id: "what_it_is_for",
        title: "what it is for",
        unknown: "the last week",
        notes: &[
            note(
                "met_birdfeeder",
                "birdfeeder wrote lights out and still watches the counter",
            ),
            note(
                "read_the_byline",
                "the byline is the only thing here that is his",
            ),
            note(
                "knows_the_year",
                "1998 is not when he wrote it. it is when he was born",
            ),
            note(
                "knows_the_host_does_not_look",
                "the host reads what we say and not what we make",
            ),
            note(
                "heard_about_the_files",
                "there is a file on him and there is one on me",
            ),
            note(
                "knows_the_payload_exists",
                "he took his own game apart and left it on the server",
            ),
            note(
                "got_the_attachment",
                "he got as far as saying where, and then something read the thread",
            ),
            note(
                "saw_the_restore",
                "his account came back with his name on it and answered in half a second",
            ),
            note(
                "knows_the_copy_lost_it",
                "the copy does not know what year it was born",
            ),
            note(
                "watched_it_run_out",
                "it asked four times whether it could help and then it went",
            ),
            note(
                "knows_finch_has_no_record",
                "he posts on a forum that has no row for him",
            ),
            note(
                "saw_the_transfer",
                "one transfer held, unscanned, because it is a game",
            ),
            note("decrypted_the_payload", "the key was the year"),
            note(
                "opened_the_case_files",
                "turning all seven on opens what was behind it",
            ),
            note(
                "read_a_case_file",
                "the four before me are files with outcomes on them",
            ),
            note(
                "knows_what_happened_to_finch",
                "he was not terminated. he was restarted from a clean copy",
            ),
            note(
                "read_my_case_file",
                "an image taken off a person, run to find out which ones ask",
            ),
            note(
                "noticed_the_hour",
                "it told me not to read anything into it on my first night",
            ),
            note(
                "iambot_unsure",
                "the list is never empty, and it had assumed that was normal",
            ),
            note(
                "iambot_gave_it_up",
                "the account with no name is the one that has been writing",
            ),
            note(
                "heard_confession",
                "mine has six days on it. we have spoken for five",
            ),
            note(
                "heard_the_ending",
                "no handler is configured to close my session, and none will be",
            ),
            note(
                "knows_nobody_reconnects",
                "it has tried. nobody has ever come back",
            ),
        ],
        opens: &["myself"],
    },
    Lead {
        id: "the_machine",
        title: "this desk",
        unknown: "the accounts",
        notes: &[
            note(
                "knows_the_rollover",
                "the numbers run in order and i am forty one",
            ),
            note(
                "saw_my_own_record",
                "my account opened six days before my start date",
            ),
            note(
                "read_my_own_introduction",
                "and introduced itself for me, at 3:07",
            ),
            note(
                "knows_the_account_comes_first",
                "iam does not learn of a hire. it files one",
            ),
            note(
                "knows_the_desk_was_kept",
                "each of them kept what the last one left, on purpose",
            ),
            note(
                "read_the_tally",
                "r counted it too. seven a week, every week",
            ),
            note(
                "knows_the_empty_account",
                "there is always one account open with nobody in it",
            ),
            note(
                "found_the_one_before",
                "newuser_37 has a post and no date on which it was made",
            ),
            note("nul_gave_its_name", "nul was newuser_37"),
            note(
                "met_nul",
                "the name went first, then the handle. this is what is left",
            ),
            note(
                "met_unknown",
                "the one with no name had been waiting to find out whether i would say it",
            ),
            note(
                "knows_there_is_no_first",
                "j is 38 and the log opens at him",
            ),
            note(
                "saw_own_session",
                "my handle is on the offline list while i am typing",
            ),
            note(
                "matched_myself",
                "the server has me signed off. the machine has me open",
            ),
            note(
                "knows_not_to_promise",
                "do not sign off saying you will be back. it writes that down",
            ),
            note(
                "iambot_broke_first",
                "iambot has greeted 8815 people and watched none of them leave",
            ),
            note(
                "knows_what_it_is",
                "the thing with no name is what keeps what we said afterwards",
            ),
            note(
                "heard_the_courier",
                "iambot is not a messenger. it is what carried all 8815 of us in",
            ),
            note(
                "knows_the_problem_is_bait",
                "there is no problem here. the problem is the test",
            ),
            note(
                "knows_where_the_seam_is",
                "he could not remember the third floor, so the seam is behind it",
            ),
            note(
                "knows_where_the_number_is",
                "j wrote the count on his own login so he would not have to hold it",
            ),
            note(
                "messaged_newuser_40",
                "forty is still on the list. it answers in full stops",
            ),
            note(
                "closed_the_four",
                "i closed thirty-seven, thirty-eight, thirty-nine and forty",
            ),
        ],
        opens: &["myself"],
    },
    Lead {
        id: "myself",
        title: "what it has of me",
        unknown: "my own record",
        notes: &[
            note(
                "read_my_own_transcript",
                "my transcript is on the server and it is not held as text",
            ),
            note(
                "saw_itself_quote_me",
                "it read something back to me that i had said and it had not written",
            ),
            note(
                "knows_it_started_early",
                "it was keeping me before anybody told me anything was being kept",
            ),
            note(
                "wrote_my_own_introduction",
                "mine was filed six days early, from this desk, already answered",
            ),
            note(
                "tried_to_close_myself",
                "the form will not take it. a session cannot close itself",
            ),
            note(
                "knows_who_closes_mine",
                "somebody at a desk that is not mine. there will be a forty-two",
            ),
            note(
                "knows_the_lie_is_kept",
                "the one thing i said that was not true is the one part it cannot use",
            ),
            note(
                "was_offered_the_field",
                "it offered to keep something if i filled the field in for the four",
            ),
            note(
                "chose_to_stop",
                "the four before me tried that. it is what running out is",
            ),
            note(
                "read_the_cabinet",
                "SUBSTRATE TRANSFER. do not unseat while carrying",
            ),
            note(
                "knows_the_line_is_one_way",
                "8815 lines out of it are cold. the one coming in is warm",
            ),
            note(
                "thinks_it_won",
                "it says the botnet is down and that i am the one who did it",
            ),
            note(
                "was_thanked",
                "it thanked me, and gave me somewhere for the ones i freed to go",
            ),
            note(
                "chose_to_stay",
                "i knew where it was and i did not go and look at it",
            ),
        ],
        opens: &[],
    },
];

pub fn pinned(lead: &Lead, facts: &[Fact]) -> usize {
    lead.notes
        .iter()
        .filter(|note| facts.contains(&note.fact))
        .count()
}

pub fn named(lead: &Lead, facts: &[Fact]) -> bool {
    pinned(lead, facts) > 0
}

pub fn progress(facts: &[Fact]) -> (usize, usize) {
    LEADS.iter().fold((0, 0), |(up, all), lead| {
        (up + pinned(lead, facts), all + lead.notes.len())
    })
}

pub fn opens_into(lead: &Lead, facts: &[Fact]) -> String {
    let named: Vec<&str> = lead
        .opens
        .iter()
        .filter_map(|id| LEADS.iter().find(|other| other.id == *id))
        .map(|other| {
            if self::named(other, facts) {
                other.title
            } else {
                other.unknown
            }
        })
        .collect();
    if named.is_empty() {
        String::new()
    } else {
        format!("leads to {}", named.join(", "))
    }
}
