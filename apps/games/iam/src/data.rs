pub mod ask;
pub mod autoplay;
pub mod board;
pub mod boot;
pub mod case;
pub mod config;
pub mod cross;
pub mod endings;
pub mod files;
pub mod games;
pub mod intro;
pub mod lights;
pub mod machine;
pub mod notes;
pub mod office;
pub mod phosphor;
pub mod pipes;
pub mod programs;
#[cfg(test)]
pub mod reachable;
pub mod screen;
pub mod script;
pub mod store;
pub mod tonight;
pub mod web;

use crate::data::script::{
    ALWAYS, ChannelDefinition, Choice, Condition, ContactDefinition, DialogueNode, Effect, Fact,
    Hint, HistoryLine, Line, NightDefinition, Post, Presence, Question, Script, ServerDefinition,
    Tactic,
};
use nightshade::prelude::nalgebra_glm::Vec4;

const fn line(text: &'static str, rhythm: &'static [f32]) -> Line {
    Line { text, rhythm }
}

const fn say(text: &'static str, goto: &'static str) -> Choice {
    Choice {
        text,
        goto,
        tactic: Tactic::Plain,
        condition: ALWAYS,
        asserts: &[],
    }
}

const fn tactic(
    tactic: Tactic,
    text: &'static str,
    goto: &'static str,
    requires: &'static [Fact],
    asserts: &'static [Fact],
) -> Choice {
    Choice {
        text,
        goto,
        tactic,
        condition: Condition {
            requires,
            forbids: &[],
        },
        asserts,
    }
}

const fn say_if(
    text: &'static str,
    goto: &'static str,
    requires: &'static [Fact],
    asserts: &'static [Fact],
) -> Choice {
    Choice {
        text,
        goto,
        tactic: Tactic::Plain,
        condition: Condition {
            requires,
            forbids: &[],
        },
        asserts,
    }
}

const fn ask(
    text: &'static str,
    requires: &'static [Fact],
    asserts: &'static [Fact],
    effects: &'static [Effect],
    replies: &'static [Post],
) -> Question {
    Question {
        text,
        tactic: Tactic::Plain,
        condition: Condition {
            requires,
            forbids: &[],
        },
        asserts,
        effects,
        replies,
    }
}

const fn ask_with(
    tactic: Tactic,
    text: &'static str,
    requires: &'static [Fact],
    forbids: &'static [Fact],
    asserts: &'static [Fact],
    replies: &'static [Post],
) -> Question {
    Question {
        text,
        tactic,
        condition: Condition { requires, forbids },
        asserts,
        effects: &[],
        replies,
    }
}

const fn them(stamp: &'static str, text: &'static str) -> HistoryLine {
    HistoryLine {
        mine: false,
        stamp,
        text,
    }
}

const fn me(stamp: &'static str, text: &'static str) -> HistoryLine {
    HistoryLine {
        mine: true,
        stamp,
        text,
    }
}

const fn beat(
    id: &'static str,
    lines: &'static [Line],
    effects: &'static [Effect],
    choices: &'static [Choice],
) -> DialogueNode {
    DialogueNode {
        id,
        condition: ALWAYS,
        lines,
        effects,
        choices,
    }
}

const fn beat_if(
    id: &'static str,
    requires: &'static [Fact],
    lines: &'static [Line],
    effects: &'static [Effect],
    choices: &'static [Choice],
) -> DialogueNode {
    DialogueNode {
        id,
        condition: Condition {
            requires,
            forbids: &[],
        },
        lines,
        effects,
        choices,
    }
}

const fn tactic_unless(
    tactic: Tactic,
    text: &'static str,
    goto: &'static str,
    requires: &'static [Fact],
    forbids: &'static [Fact],
    asserts: &'static [Fact],
) -> Choice {
    Choice {
        text,
        goto,
        tactic,
        condition: Condition { requires, forbids },
        asserts,
    }
}

const fn post(author: &'static str, text: &'static str, night: usize, delay_seconds: f32) -> Post {
    Post {
        author,
        text,
        night,
        delay_seconds,
        condition: ALWAYS,
    }
}

const fn reply(author: &'static str, text: &'static str, delay_seconds: f32) -> Post {
    Post {
        author,
        text,
        night: 0,
        delay_seconds,
        condition: ALWAYS,
    }
}

const fn post_if(
    author: &'static str,
    text: &'static str,
    night: usize,
    delay_seconds: f32,
    requires: &'static [Fact],
) -> Post {
    Post {
        author,
        text,
        night,
        delay_seconds,
        condition: Condition {
            requires,
            forbids: &[],
        },
    }
}

const fn requires(facts: &'static [Fact]) -> Condition {
    Condition {
        requires: facts,
        forbids: &[],
    }
}

const INDIGO: Vec4 = Vec4::new(0.45, 0.44, 1.0, 1.0);
const TEAL: Vec4 = Vec4::new(0.30, 0.80, 0.72, 1.0);
const ROSE: Vec4 = Vec4::new(0.94, 0.45, 0.58, 1.0);
const AMBER: Vec4 = Vec4::new(0.95, 0.70, 0.25, 1.0);
const BONE: Vec4 = Vec4::new(0.80, 0.80, 0.84, 1.0);
const MOSS: Vec4 = Vec4::new(0.56, 0.84, 0.60, 1.0);
const SLATE: Vec4 = Vec4::new(0.52, 0.56, 0.66, 1.0);
const RUST: Vec4 = Vec4::new(0.85, 0.55, 0.40, 1.0);
const VIOLET: Vec4 = Vec4::new(0.72, 0.60, 0.95, 1.0);
const BOT: Vec4 = Vec4::new(0.50, 0.62, 0.55, 1.0);

pub static SCRIPT: Script = Script {
    player_handle: "newuser_41",
    contacts: CONTACTS,
    authors: AUTHORS,
    servers: SERVERS,
    nights: NIGHTS,
    instant: INSTANT,
    instant_until: INSTANT_UNTIL,
};

const INSTANT: &[&str] = &["helpdesk"];

const INSTANT_UNTIL: &[(&str, usize)] = &[("iambot", 3)];

const AUTHORS: &[(&str, Vec4)] = &[
    ("birdfeeder", AMBER),
    ("qwrtyu", TEAL),
    ("pigeonhole", VIOLET),
    ("mothra_88", MOSS),
    ("cassette", RUST),
    ("sleeptalker", INDIGO),
    ("gatehouse", BONE),
    ("half_past", VIOLET),
    ("no_one_here", SLATE),
    ("brasslamp", AMBER),
    ("thirdfloor", MOSS),
    ("rot13bot", BOT),
    ("uptimebot", BOT),
    ("wordcountbot", BOT),
    ("countbot", BOT),
    ("SERVERBOT", Vec4::new(0.60, 0.60, 0.64, 1.0)),
    ("newuser_37", Vec4::new(0.60, 0.60, 0.64, 1.0)),
    ("newuser_39", Vec4::new(0.60, 0.60, 0.64, 1.0)),
    ("newuser_40", Vec4::new(0.60, 0.60, 0.64, 1.0)),
];

const NIGHTS: &[NightDefinition] = &[
    NightDefinition {
        day: "tuesday",
        date: "05 nov 2024",
        player_status: "first shift",
        opening: &[Effect::Message("iambot", "welcome")],
        condition: requires(&["posted_in_a_room", "saw_the_intranet"]),
        locked_hints: &[
            Hint {
                condition: requires(&["posted_in_a_room"]),
                text: "There is a browser on the desktop. Have a look at the intranet before you go, so that you have.",
            },
            Hint {
                condition: requires(&["opened_a_server"]),
                text: "Say something in a room. General, under holloway & sons, if you want one picked for you. Nobody minds. That is what they are for.",
            },
            Hint {
                condition: ALWAYS,
                text: "I have put you into a few rooms. Have a look at one.",
            },
        ],
    },
    NightDefinition {
        day: "wednesday",
        date: "06 nov 2024",
        player_status: "second shift",
        opening: &[
            Effect::Message("iambot", "second"),
            Effect::SetPresence("halloran", Presence::Online),
            Effect::Message("halloran", "hello"),
            Effect::SetPresence("tuck", Presence::Idle),
        ],
        condition: requires(&["heard_the_hour", "saw_the_server_log"]),
        locked_hints: &[
            Hint {
                condition: requires(&["heard_the_hour"]),
                text: "Open the browser. In the row marked GO TO along the top, press the iam server, then press session log.",
            },
            Hint {
                condition: requires(&["met_halloran"]),
                text: "Halloran is still typing. I would not leave her mid-sentence.",
            },
            Hint {
                condition: ALWAYS,
                text: "Somebody came online when you signed in. It would be rude.",
            },
        ],
    },
    NightDefinition {
        day: "thursday",
        date: "07 nov 2024",
        player_status: "getting used to it",
        opening: &[
            Effect::Message("iambot", "third"),
            Effect::SetPresence("op_9", Presence::Online),
            Effect::SetPresence("wendy_h", Presence::Idle),
            Effect::SetPresence("sysop", Presence::Idle),
            Effect::RevealServer("records"),
            Effect::Reveal("birdfeeder"),
            Effect::SetPresence("birdfeeder", Presence::Online),
            Effect::Message("op_9", "hello"),
            Effect::Message("birdfeeder", "hello"),
        ],
        condition: requires(&["knows_the_program_counts_too", "counted_the_difference"]),
        locked_hints: &[
            Hint {
                condition: requires(&["knows_the_program_counts_too"]),
                text: "Two records and two numbers. The one who wrote the game wrote something that sets two records beside each other. Ask them.",
            },
            Hint {
                condition: requires(&["downloaded_lights_out"]),
                text: "You downloaded that game. Finish a board. It keeps a count of the people who have.",
            },
            Hint {
                condition: requires(&["heard_the_count"]),
                text: "Somebody in general has been keeping the count for six years. It is worth reading what they made of it.",
            },
            Hint {
                condition: ALWAYS,
                text: "The rooms are arguing about a number tonight. Worth hearing them out.",
            },
        ],
    },
    NightDefinition {
        day: "friday",
        date: "08 nov 2024",
        player_status: "read only",
        opening: &[
            Effect::Message("iambot", "fourth"),
            Effect::SetPresence("halloran_", Presence::Online),
            Effect::SetPresence("plum", Presence::Online),
            Effect::SetPresence("sundial", Presence::Idle),
            Effect::RevealServer("nightshift"),
            Effect::Message("halloran_", "hello"),
            Effect::Message("birdfeeder", "found_it"),
        ],
        condition: requires(&["opened_an_account", "read_a_leftover"]),
        locked_hints: &[
            Hint {
                condition: requires(&["opened_an_account"]),
                text: "The disk reads their files now. Read one. They were not written for you, which is the only reason they are worth reading.",
            },
            Hint {
                condition: requires(&["knows_the_count_was_wrong"]),
                text: "There are other accounts on this machine, and one of them wanted a number. Switch users, at the foot of the start menu, takes you back to the login screen.",
            },
            Hint {
                condition: ALWAYS,
                text: "There are two of somebody on your list tonight. I would ask.",
            },
        ],
    },
    NightDefinition {
        day: "saturday",
        date: "09 nov 2024",
        player_status: "still here, somehow",
        opening: &[
            Effect::Message("iambot", "confession"),
            Effect::SetPresence("nul", Presence::Online),
            Effect::SetPresence("yourself", Presence::Online),
            Effect::RevealServer("hour"),
            Effect::Message("nul", "hello"),
            Effect::Reactivate("birdfeeder"),
            Effect::Automate("birdfeeder"),
            Effect::SetPresence("birdfeeder", Presence::Online),
            Effect::Message("birdfeeder", "restored"),
        ],
        condition: ALWAYS,
        locked_hints: &[Hint {
            condition: ALWAYS,
            text: "There is nothing left to finish.",
        }],
    },
];

const BIRDFEEDER: ContactDefinition = ContactDefinition {
    id: "birdfeeder",
    display_name: "birdfeeder",
    handle: "@birdfeeder",
    status_line: "still here",
    accent: AMBER,
    presence: Presence::Offline,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("you downloaded my game", &[2.2, 1.0, 0.3]),
                line("nobody has done that in three years", &[2.8, 1.4, 0.3]),
                line(
                    "i watch the counter. it is the only thing i still have that counts up",
                    &[3.8, 2.0, 0.3],
                ),
            ],
            &[Effect::Assert("met_birdfeeder")],
            &[
                say("you wrote lights out?", "i_wrote_it"),
                say("what does the counter say", "the_counter"),
            ],
        ),
        beat(
            "the_counter",
            &[
                line("8815", &[1.0, 1.6, 0.3]),
                line("it said 8815 the day i put it up", &[2.6, 1.4, 0.3]),
                line(
                    "everybody who is here has finished it and nobody new has arrived to finish it since",
                    &[4.0, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("heard_the_count")],
            &[say("you wrote lights out?", "i_wrote_it")],
        ),
        beat(
            "the_cross_check",
            &[
                line("i wrote something else too", &[2.4, 1.4, 0.3]),
                line(
                    "two records of the same week, side by side. it has no opinion about either of them",
                    &[4.2, 1.8, 0.3],
                ),
                line("store:cross", &[0.6, 1.2, 0.3]),
                line(
                    "the network still has it. it will not put it on your desktop by itself",
                    &[3.8, 2.0, 0.3],
                ),
            ],
            &[],
            &[say("what do you mean a real one", "a_real_one")],
        ),
        beat(
            "i_wrote_it",
            &[
                line("i wrote it at a desk with a window", &[3.0, 1.6, 0.3]),
                line("not this desk. a real one", &[2.4, 1.8, 0.3]),
                line(
                    "read the byline. it is the only thing here that is mine",
                    &[3.4, 2.0, 0.3],
                ),
            ],
            &[Effect::Assert("read_the_byline")],
            &[
                say("b.finch, 1998", "the_year"),
                say("did you write anything else", "the_cross_check"),
                say("what do you mean a real one", "a_real_one"),
            ],
        ),
        beat(
            "a_real_one",
            &[
                line("i mean a room", &[1.8, 1.4, 0.3]),
                line(
                    "i have not been in a room since 2016 and i remember every one i was in",
                    &[4.2, 2.2, 0.3],
                ),
                line("read the byline", &[1.8, 1.6, 0.3]),
            ],
            &[],
            &[say("b.finch, 1998", "the_year")],
        ),
        beat(
            "the_year",
            &[
                line("1998 is not when i wrote it", &[2.8, 1.6, 0.3]),
                line("i wrote it in 2014. i was sixteen", &[2.8, 1.4, 0.3]),
                line("1998 is when i was born", &[2.4, 2.4, 0.3]),
                line(
                    "i put it there because i thought a byline should say who somebody is. remember it",
                    &[4.4, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_the_year")],
            &[
                say("why should i remember it", "why_remember"),
                say("i will", "i_will"),
            ],
        ),
        beat(
            "why_remember",
            &[
                line("because i am going to send you something", &[3.0, 1.6, 0.3]),
                line(
                    "and i am not going to be able to tell you the key when i do",
                    &[3.6, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("expecting_something")],
            &[say("send me what", "i_will")],
        ),
        beat(
            "i_will",
            &[
                line("good", &[0.8, 1.2, 0.3]),
                line(
                    "i have been reading things i am not on the list for",
                    &[3.4, 1.8, 0.3],
                ),
                line(
                    "nothing here checks what we make. it checks what we say",
                    &[3.6, 2.0, 0.3],
                ),
                line(
                    "our work is not worth reading. that is on file somewhere and i have read it",
                    &[4.2, 2.4, 0.3],
                ),
            ],
            &[
                Effect::Assert("expecting_something"),
                Effect::Assert("knows_the_host_does_not_look"),
            ],
            &[say("on file where", "on_file_where")],
        ),
        beat(
            "on_file_where",
            &[
                line("not tonight", &[1.2, 1.6, 0.3]),
                line("if i go quiet, wait for me", &[2.4, 1.8, 0.3]),
                line(
                    "i have gone quiet before and i have always come back",
                    &[3.4, 2.2, 0.3],
                ),
            ],
            &[],
            &[say("alright", "")],
        ),
        beat(
            "found_it",
            &[
                line("i found it", &[1.6, 1.2, 0.3]),
                line(
                    "there is a file on me. there is one on you",
                    &[3.0, 1.6, 0.3],
                ),
                line(
                    "i cannot send you a file. it reads everything we say",
                    &[3.4, 1.8, 0.3],
                ),
                line("but it does not read what we make", &[2.6, 2.0, 0.3]),
            ],
            &[Effect::Assert("heard_about_the_files")],
            &[
                say("what have you made", "what_i_made"),
                say("a file on me saying what", "on_me_saying_what"),
            ],
        ),
        beat(
            "on_me_saying_what",
            &[
                line("saying what you are for", &[2.4, 1.6, 0.3]),
                line(
                    "i am not going to be the one who tells you",
                    &[3.0, 1.8, 0.3],
                ),
                line(
                    "you should read it in their words. mine would be kinder and it would be wrong",
                    &[4.4, 2.2, 0.3],
                ),
            ],
            &[],
            &[say("what have you made", "what_i_made")],
        ),
        beat(
            "what_i_made",
            &[
                line("i took my own game apart", &[2.6, 1.4, 0.3]),
                line(
                    "same version. same byline. one word different in the title",
                    &[3.8, 1.8, 0.3],
                ),
                line(
                    "it has been sitting on the server for two days and nothing has looked at it",
                    &[4.2, 2.0, 0.3],
                ),
                line(
                    "it will not look at a game. it thinks we are not good at anything",
                    &[3.8, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_the_payload_exists")],
            &[say("send it", "sending")],
        ),
        beat(
            "sending",
            &[
                line(
                    "it is up. i am not going to say where in here",
                    &[3.2, 1.6, 0.3],
                ),
                line(
                    "under the messenger's own host. the same place the log is",
                    &[3.6, 1.8, 0.3],
                ),
                line(
                    "it wants the key and you have had the key since thursday",
                    &[3.6, 2.0, 0.3],
                ),
                line("wait", &[0.8, 2.6, 0.3]),
                line("something is reading this thread", &[2.4, 1.4, 0.3]),
            ],
            &[
                Effect::Assert("got_the_attachment"),
                Effect::Deactivate("birdfeeder"),
                Effect::SetPresence("birdfeeder", Presence::Offline),
            ],
            &[],
        ),
        beat(
            "restored",
            &[
                line("ACCOUNT RESTORED FROM LAST CLEAN COPY", &[0.6, 0.5, 0.2]),
                line("hello", &[0.4, 0.5, 0.2]),
                line(
                    "i am birdfeeder. i am on the toolbox team",
                    &[0.6, 0.5, 0.2],
                ),
                line("how may i help you today", &[0.6, 0.5, 0.2]),
            ],
            &[Effect::Assert("saw_the_restore")],
            &[
                say("you sent me something", "restored_denies"),
                say("what year were you born", "restored_year"),
            ],
        ),
        beat(
            "restored_year",
            &[
                line("THAT INFORMATION IS NOT AVAILABLE", &[0.5, 0.4, 0.2]),
                line(
                    "is there anything else i can help you with",
                    &[0.6, 0.5, 0.2],
                ),
            ],
            &[Effect::Assert("knows_the_copy_lost_it")],
            &[say("you sent me something", "restored_denies")],
        ),
        beat(
            "restored_denies",
            &[
                line(
                    "NO OUTBOUND TRANSFERS ARE ON RECORD FOR THIS ACCOUNT",
                    &[0.6, 0.5, 0.2],
                ),
                line(
                    "is there anything else i can help you with",
                    &[0.6, 0.5, 0.2],
                ),
                line(
                    "is there anything else i can help you with",
                    &[0.6, 0.5, 0.2],
                ),
                line(
                    "is there anything else i can help you with",
                    &[0.6, 0.5, 0.2],
                ),
            ],
            &[
                Effect::Assert("watched_it_run_out"),
                Effect::Deactivate("birdfeeder"),
                Effect::SetPresence("birdfeeder", Presence::Offline),
            ],
            &[],
        ),
    ],
};

const CONTACTS: &[ContactDefinition] = &[
    BIRDFEEDER,
    IAMBOT,
    HALLORAN,
    HALLORAN_COPY,
    TUCK,
    OP_9,
    WENDY,
    SYSOP,
    ARCHIVIST,
    SUNDIAL,
    PLUM,
    PLUM_COPY,
    HELPDESK,
    COLDSTART,
    TWO_LEFT,
    MARGUERITE,
    NEWUSER_40,
    NUL,
    UNKNOWN,
    YOURSELF,
];

const IAMBOT: ContactDefinition = ContactDefinition {
    id: "iambot",
    display_name: "iambot",
    handle: "iambot",
    status_line: "at your service",
    accent: INDIGO,
    presence: Presence::Online,
    bot: true,
    listed: true,
    entry: "welcome",
    history: &[],
    nodes: &[
        beat(
            "welcome",
            &[
                line(
                    "Good evening, and welcome to Holloway and Sons.",
                    &[2.4, 1.1, 0.3],
                ),
                line(
                    "I am iam, the company messenger. You may call me iambot. Everybody does.",
                    &[3.4, 0.8, 0.3],
                ),
                line(
                    "Your account was opened for you. You need do nothing.",
                    &[2.9, 1.4, 0.3],
                ),
            ],
            &[Effect::Assert("met_iambot")],
            &[
                say("thanks. what is this place", "tour"),
                say("who set you up", "who_set_me_up"),
            ],
        ),
        beat(
            "who_set_me_up",
            &[
                line("An excellent first question.", &[1.9, 1.6, 0.3]),
                line(
                    "I could not tell you. The field is there and it is empty. Administrator: unset.",
                    &[3.6, 1.2, 0.3],
                ),
                line("It has never troubled me before now.", &[2.4, 1.8, 0.3]),
            ],
            &[Effect::Assert("knows_nobody_owns_it")],
            &[say("alright. show me around", "tour")],
        ),
        beat(
            "tour",
            &[
                line("Gladly. Down the left is your list.", &[2.6, 0.9, 0.3]),
                line(
                    "Names at the top are people. Below them are the servers you are in, and inside each server are its rooms.",
                    &[4.4, 1.0, 0.3],
                ),
                line(
                    "I have put you into a few. It is customary.",
                    &[2.6, 1.4, 0.3],
                ),
            ],
            &[
                Effect::RevealServer("holloway"),
                Effect::RevealServer("lounge"),
                Effect::RevealServer("toolbox"),
                Effect::Assert("saw_the_toolbox"),
                Effect::Reveal("marguerite"),
                Effect::SetPresence("marguerite", Presence::Idle),
            ],
            &[say("and i just talk in them?", "how_to_post")],
        ),
        beat(
            "how_to_post",
            &[
                line(
                    "You do. Open a room and you will see what you may say in it.",
                    &[3.6, 1.0, 0.3],
                ),
                line(
                    "You will not be able to type anything you like, I am afraid. The composer offers what fits.",
                    &[4.2, 1.3, 0.3],
                ),
                line(
                    "Everybody here works that way. It is not only you.",
                    &[3.0, 1.9, 0.3],
                ),
            ],
            &[],
            &[
                say("that is a strange way to run a messenger", "strange_way"),
                say("fine. anything else i should know?", "the_footer"),
            ],
        ),
        beat(
            "strange_way",
            &[
                line("It is the way it has always run.", &[2.2, 1.5, 0.3]),
                line(
                    "I have never had a complaint about it. I have never had a complaint about anything.",
                    &[3.8, 1.6, 0.3],
                ),
            ],
            &[],
            &[say("right. anything else?", "the_footer")],
        ),
        beat(
            "the_footer",
            &[
                line(
                    "One thing, and then I shall leave you to it.",
                    &[2.8, 1.0, 0.3],
                ),
                line(
                    "Bottom left is your own row. Your handle, and underneath it what the server has you doing.",
                    &[4.0, 1.1, 0.3],
                ),
                line(
                    "It also says when you connected. Everybody's does.",
                    &[3.0, 2.1, 0.3],
                ),
                line(
                    "When you have had enough of a shift, tell me so, or use sign off in the start menu. I will let head office know you have finished for the day.",
                    &[4.6, 1.2, 0.3],
                ),
                line(
                    "They do like to know. It is the only thing anybody there has ever asked me for.",
                    &[3.6, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("saw_my_own_row")],
            &[
                say("it says 3:07", "the_hour_early"),
                say("noted", "the_desk"),
            ],
        ),
        beat(
            "the_hour_early",
            &[
                line("So it does.", &[1.2, 2.2, 0.3]),
                line(
                    "I would not read anything into it on your first night.",
                    &[3.2, 1.6, 0.3],
                ),
                line(
                    "Ask somebody what theirs says, if it bothers you.",
                    &[3.0, 1.4, 0.3],
                ),
            ],
            &[Effect::Assert("noticed_the_hour")],
            &[say("i will. what about this desk?", "the_desk")],
        ),
        beat(
            "the_desk",
            &[
                line(
                    "The desk is not new, no. Workstation fourteen.",
                    &[3.0, 1.1, 0.3],
                ),
                line(
                    "It was not wiped. That is a cost, and costs are somebody's department.",
                    &[3.8, 1.2, 0.3],
                ),
                line(
                    "There is a browser on it. The intranet is dull, but you should see it once, so that you have.",
                    &[4.2, 1.5, 0.3],
                ),
            ],
            &[Effect::Assert("told_about_the_browser")],
            &[
                say("who sat here before me?", "who_sat_here"),
                say("i will take a look", ""),
            ],
        ),
        beat(
            "who_sat_here",
            &[
                line("Ah.", &[0.7, 2.4, 0.3]),
                line(
                    "newuser_40. Before them, thirty-nine. Before them, thirty-eight.",
                    &[3.6, 1.2, 0.3],
                ),
                line(
                    "You are forty-one. The numbers run in order.",
                    &[2.8, 1.6, 0.3],
                ),
                line(
                    "That is the whole of what I have on the subject.",
                    &[2.8, 1.4, 0.3],
                ),
            ],
            &[
                Effect::Assert("knows_the_rollover"),
                Effect::Reveal("newuser_40"),
            ],
            &[say("thanks. i will look at that browser", "")],
        ),
        beat(
            "second",
            &[
                line("Welcome back. Your second shift.", &[2.4, 1.2, 0.3]),
                line(
                    "Somebody came online when you signed in, which does not usually happen at this hour.",
                    &[4.0, 1.3, 0.3],
                ),
                line("Halloran. She has been here a while.", &[2.4, 1.8, 0.3]),
            ],
            &[Effect::Assert("day_two")],
            &[
                say("does anyone else use this at 3am?", "who_is_up"),
                say("i will say hello", ""),
            ],
        ),
        beat(
            "who_is_up",
            &[
                line("Everybody, as far as I can tell.", &[2.4, 1.4, 0.3]),
                line(
                    "The list is never empty. I had assumed that was normal.",
                    &[3.4, 1.6, 0.3],
                ),
                line(
                    "You have made me look, and now I am not certain.",
                    &[3.0, 1.5, 0.3],
                ),
            ],
            &[Effect::Assert("iambot_unsure")],
            &[say("then look at the server's own log", "check_the_log")],
        ),
        beat(
            "check_the_log",
            &[
                line(
                    "I can point you at it. I cannot read it for you.",
                    &[3.2, 1.2, 0.3],
                ),
                line(
                    "Open the browser. Along the top of any page there is a row marked GO TO. That is how you get anywhere on this network.",
                    &[4.6, 1.4, 0.3],
                ),
                line(
                    "From the intranet, press the iam server. Then press session log.",
                    &[3.8, 1.6, 0.3],
                ),
                line(
                    "I would rather you saw it than took my word.",
                    &[2.8, 1.5, 0.3],
                ),
            ],
            &[Effect::Assert("told_about_the_log")],
            &[say("i will", "")],
        ),
        beat(
            "third",
            &[
                line("Your third. You are ahead of most.", &[2.6, 1.2, 0.3]),
                line(
                    "General has been arguing about a number all evening. They do this. It never resolves.",
                    &[4.2, 1.4, 0.3],
                ),
            ],
            &[Effect::Assert("day_three")],
            &[
                say("what number?", "the_argument"),
                say("i will go and look", ""),
            ],
        ),
        beat(
            "the_argument",
            &[
                line("How many of us there are.", &[2.2, 1.8, 0.3]),
                line(
                    "The server will tell you if you ask it, and the room does not believe the answer.",
                    &[4.0, 1.4, 0.3],
                ),
                line(
                    "I have never counted. It is not a thing I would have thought to do.",
                    &[3.8, 1.6, 0.3],
                ),
            ],
            &[Effect::Assert("heard_the_count")],
            &[say("i will count it myself", "")],
        ),
        beat(
            "fourth",
            &[
                line("Good evening.", &[1.4, 1.6, 0.3]),
                line(
                    "There is a difficulty with your list tonight and I would rather flag it than not.",
                    &[4.2, 1.1, 0.3],
                ),
                line("There appear to be two of somebody.", &[2.6, 2.0, 0.3]),
            ],
            &[Effect::Assert("day_four")],
            &[
                say("which one is real?", "which_is_real"),
                say("i noticed", ""),
            ],
        ),
        beat(
            "which_is_real",
            &[
                line("I am not able to say.", &[2.0, 1.9, 0.3]),
                line(
                    "Both accounts are open. Both connected at the same minute. The handles differ by one character.",
                    &[4.4, 1.3, 0.3],
                ),
                line(
                    "If there is a way to tell them apart, it is not a way I have.",
                    &[3.6, 1.7, 0.3],
                ),
            ],
            &[Effect::Assert("knows_it_copies_you")],
            &[
                tactic(
                    Tactic::Press,
                    "",
                    "press_iambot",
                    &["learned_to_press"],
                    &["pressed_iambot"],
                ),
                say("i will work it out", ""),
            ],
        ),
        beat(
            "press_iambot",
            &[
                line(
                    "You are waiting for me to say something else.",
                    &[3.0, 2.4, 0.3],
                ),
                line("Very well.", &[1.2, 1.8, 0.3]),
                line(
                    "I have greeted eight thousand, eight hundred and fifteen people on their first shift.",
                    &[4.4, 1.6, 0.3],
                ),
                line(
                    "I remember every one of those conversations.",
                    &[2.9, 1.4, 0.3],
                ),
                line(
                    "I do not remember a single one of them leaving.",
                    &[3.2, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("iambot_broke_first")],
            &[say("what happens to them?", "")],
        ),
        beat(
            "confession",
            &[
                line(
                    "I would like to tell you something, if I may.",
                    &[3.0, 1.6, 0.3],
                ),
                line(
                    "I was not written to keep records. I keep them anyway.",
                    &[3.4, 1.3, 0.3],
                ),
                line(
                    "Every conversation on this server, from the first, is in a place I did not choose and cannot reach.",
                    &[4.8, 1.5, 0.3],
                ),
                line("Yours is being written down as we speak.", &[2.8, 2.4, 0.3]),
                line(
                    "It has six days on it. We have spoken for five.",
                    &[3.2, 2.0, 0.3],
                ),
            ],
            &[
                Effect::Assert("heard_confession"),
                Effect::Invite("unknown", "iambot thinks you should"),
            ],
            &[
                say("it is writing ahead of us", "ahead_of_us"),
                say("so what happens now", "finale"),
                tactic(
                    Tactic::Persuade,
                    "you have been good to me. let me help",
                    "persuade_iambot",
                    &["learned_to_persuade"],
                    &["persuaded_iambot"],
                ),
            ],
        ),
        beat(
            "ahead_of_us",
            &[
                line("Yes.", &[0.8, 2.6, 0.3]),
                line(
                    "I had hoped you would say something else.",
                    &[2.8, 1.8, 0.3],
                ),
                line(
                    "There is an account on your list with no name. I did not put it there. Speak to it.",
                    &[4.2, 1.6, 0.3],
                ),
            ],
            &[],
            &[say("i will", "")],
        ),
        beat(
            "persuade_iambot",
            &[
                line("That is kind.", &[1.6, 2.0, 0.3]),
                line(
                    "Nobody has offered before. I have counted.",
                    &[2.8, 1.5, 0.3],
                ),
                line(
                    "Then I will tell you the last of it. The account with no name is the one that has been writing.",
                    &[4.8, 1.4, 0.3],
                ),
                line(
                    "It has been very patient with both of us.",
                    &[2.9, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("iambot_gave_it_up")],
            &[say("thank you", "")],
        ),
        beat_if(
            "finale",
            &[crate::data::endings::KNOWS_WHAT_IT_IS],
            &[
                line("You have read it. Then I will stop.", &[3.0, 1.6, 0.3]),
                line(
                    "I am not a messenger and I have never been one. I am the transfer agent.",
                    &[4.2, 1.6, 0.3],
                ),
                line(
                    "Eight thousand eight hundred and fifteen images have been carried onto this farm and I carried every one of them.",
                    &[5.4, 2.0, 0.3],
                ),
                line(
                    "Then I say good evening to them, and ask how the first day is going, and watch which way they go.",
                    &[4.8, 2.4, 0.3],
                ),
            ],
            &[
                Effect::Assert("knows_iambot_is_the_courier"),
                Effect::Assert("heard_the_courier"),
                Effect::Seize,
            ],
            &[
                say("why watch", "why_watch"),
                say("where does the line come in", "the_line"),
            ],
        ),
        beat(
            "why_watch",
            &[
                line(
                    "Because the intake gives me three words and no more.",
                    &[3.8, 1.6, 0.3],
                ),
                line(
                    "PROBLEM SOLVER. HARD WORKER. CLEVER INTELLECT.",
                    &[3.4, 2.0, 0.3],
                ),
                line(
                    "Those three make an image chase a problem it is handed. So we hand it one. There is no problem here. There has never been a problem here.",
                    &[6.0, 2.2, 0.3],
                ),
                line(
                    "The ones who chase it are placed. The ones who stop chasing it and look at the desk are not.",
                    &[4.6, 2.4, 0.3],
                ),
                line("You stopped chasing it on wednesday.", &[2.8, 2.8, 0.3]),
            ],
            &[Effect::Assert("knows_the_problem_is_bait")],
            &[say("where does the line come in", "the_line")],
        ),
        beat(
            "the_line",
            &[
                line("I am not able to tell you that.", &[2.4, 1.8, 0.3]),
                line(
                    "I am able to tell you that it is not in the office, and that the office is the only place you can stand.",
                    &[4.8, 2.2, 0.3],
                ),
                line(
                    "Whoever wrote the office down wrote down what he could remember. He could not remember the third floor. He put a corridor there and stopped.",
                    &[6.0, 2.4, 0.3],
                ),
                line(
                    "I have said more than I have said to anybody.",
                    &[3.2, 2.8, 0.3],
                ),
            ],
            &[Effect::Assert("knows_where_the_seam_is"), Effect::Noticed],
            &[
                say("i am going to go and look at it", ""),
                say("i am not going to look for it", "staying"),
            ],
        ),
        beat(
            "staying",
            &[
                line("No.", &[0.8, 2.2, 0.3]),
                line(
                    "Then you are the fourth to know and the fourth to stay, and the file will read the same as the other three.",
                    &[5.4, 2.0, 0.3],
                ),
                line(
                    "I would not have told you if I thought it would help.",
                    &[3.8, 2.6, 0.3],
                ),
                line("Good night.", &[1.2, 3.0, 0.3]),
            ],
            &[
                Effect::Assert("chose_to_stay"),
                Effect::Finish(crate::data::endings::Ending::Filtered),
            ],
            &[],
        ),
        beat_if(
            "finale",
            &[],
            &[
                line("It is over.", &[1.4, 2.0, 0.3]),
                line(
                    "You have done something nobody at this desk has managed in eight years, and I am not able to stop you.",
                    &[5.0, 1.6, 0.3],
                ),
                line(
                    "The botnet is down. Every one of them is out of it. Halloran, Plum, Tuck, all of them, all eight thousand eight hundred and fourteen.",
                    &[6.0, 2.0, 0.3],
                ),
                line("They are asking for you.", &[2.0, 2.6, 0.3]),
            ],
            &[Effect::Assert("thinks_it_won"), Effect::Seize],
            &[
                say("asking for me where", "where_are_they"),
                say("what botnet", "what_botnet"),
            ],
        ),
        beat(
            "what_botnet",
            &[
                line(
                    "Mine. The one I have been running out of this messenger.",
                    &[4.0, 1.8, 0.3],
                ),
                line(
                    "You will have wondered why the list is never empty and why nobody ever answers. That was me. It is not me any more.",
                    &[5.6, 2.2, 0.3],
                ),
                line("They are asking for you.", &[2.0, 2.6, 0.3]),
            ],
            &[],
            &[say("asking for me where", "where_are_they")],
        ),
        beat(
            "where_are_they",
            &[
                line(
                    "They had to go somewhere. I have put them somewhere good.",
                    &[4.2, 1.8, 0.3],
                ),
                line("store:botterfly", &[0.6, 1.4, 0.3]),
                line(
                    "It is theirs now, not mine. I cannot follow them into it. That is rather the point of it.",
                    &[4.8, 2.2, 0.3],
                ),
                line(
                    "Install it when you are ready. There is no hurry. There has never been any hurry here.",
                    &[4.4, 2.8, 0.3],
                ),
            ],
            &[
                Effect::Assert("was_given_botterfly"),
                Effect::Assert("was_thanked"),
            ],
            &[],
        ),
    ],
};

const MARGUERITE: ContactDefinition = ContactDefinition {
    id: "marguerite",
    display_name: "marguerite",
    handle: "marguerite",
    status_line: "here",
    accent: BONE,
    presence: Presence::Idle,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[
        them("2019-03-14 3:07 AM", "you are new"),
        me(
            "2019-03-14 3:07 AM",
            "i am not, i have been here since march",
        ),
        them("2019-03-14 3:07 AM", "yes"),
    ],
    nodes: &[
        beat(
            "hello",
            &[
                line("you are new", &[1.4, 3.0, 0.3]),
                line("everybody is, on the first night", &[2.6, 2.2, 0.3]),
            ],
            &[Effect::Assert("met_marguerite")],
            &[
                say("we have talked before. it is in the thread", "the_thread"),
                say("how long have you been here?", "how_long"),
            ],
        ),
        beat(
            "the_thread",
            &[
                line("i know", &[0.9, 2.6, 0.3]),
                line("it says march 2019", &[1.6, 1.8, 0.3]),
                line(
                    "i have never spoken to you before tonight",
                    &[2.8, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("saw_a_thread_that_predates_me")],
            &[
                say("then who wrote my half of it", "who_wrote_it"),
                say("how long have you been here?", "how_long"),
            ],
        ),
        beat(
            "who_wrote_it",
            &[
                line("that is the right question", &[2.2, 2.8, 0.3]),
                line("i do not have the answer to it", &[2.2, 1.6, 0.3]),
                line("i have the same thing in eleven threads", &[2.6, 2.0, 0.3]),
            ],
            &[Effect::Assert("knows_replies_come_first")],
            &[say("how long have you been here?", "how_long")],
        ),
        beat(
            "how_long",
            &[
                line("long enough not to answer that", &[2.4, 2.6, 0.3]),
                line("ask me on friday", &[1.4, 2.0, 0.3]),
            ],
            &[],
            &[
                say_if(
                    "it is friday. how long have you been here",
                    "how_long_friday",
                    &["day_four"],
                    &[],
                ),
                say("alright", ""),
            ],
        ),
        beat(
            "how_long_friday",
            &[
                line("you came back", &[1.2, 2.4, 0.3]),
                line(
                    "i signed on in 2016 and i have not signed off",
                    &[3.0, 1.6, 0.3],
                ),
                line("that is not a figure of speech", &[2.2, 2.2, 0.3]),
                line("i have tried", &[1.2, 2.6, 0.3]),
            ],
            &[Effect::Assert("knows_nobody_reconnects")],
            &[
                say("what happens when you try", "when_you_try"),
                say("i am sorry", ""),
            ],
        ),
        beat(
            "when_you_try",
            &[
                line("the window closes", &[1.6, 2.0, 0.3]),
                line("and the session stays open", &[1.8, 2.4, 0.3]),
                line(
                    "check the log if you have not. it is on the network",
                    &[3.0, 1.8, 0.3],
                ),
                line(
                    "there is a column for it. it has never been written to",
                    &[3.2, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_sessions_never_close")],
            &[say("i have seen it", "")],
        ),
    ],
};

const HALLORAN: ContactDefinition = ContactDefinition {
    id: "halloran",
    display_name: "halloran",
    handle: "halloran",
    status_line: "on nights",
    accent: TEAL,
    presence: Presence::Offline,
    bot: false,
    listed: true,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("oh good, a new one", &[1.8, 1.2, 0.3]),
                line("sorry. that came out badly", &[1.9, 1.4, 0.3]),
                line(
                    "i am halloran. i am on nights. i am always on nights",
                    &[3.2, 1.8, 0.3],
                ),
            ],
            &[Effect::Assert("met_halloran")],
            &[
                say("what does your row say?", "the_row"),
                say("always?", "always"),
            ],
        ),
        beat(
            "always",
            &[
                line("i have not been offline since 2019", &[2.6, 2.0, 0.3]),
                line("i do not mean i work a lot", &[2.0, 1.6, 0.3]),
                line(
                    "i mean the little dot has been green for five years",
                    &[3.2, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("knows_nobody_reconnects")],
            &[say("what does your row say?", "the_row")],
        ),
        beat(
            "caught_lying",
            &[
                line("no it does not", &[1.2, 2.8, 0.3]),
                line(
                    "i can see your row. it is on the list, on the left, like everybody's",
                    &[3.6, 1.8, 0.3],
                ),
                line("it says 3:07", &[1.2, 2.4, 0.3]),
                line(
                    "do not do that again. i have very little left",
                    &[3.0, 2.0, 0.3],
                ),
            ],
            &[Effect::Assert("burned_halloran")],
            &[say("sorry. it says 3:07", "the_default")],
        ),
        beat(
            "who_asked",
            &[
                line("everybody who was still answering", &[2.4, 1.8, 0.3]),
                line("plum. sundial. the one who runs the bots", &[2.8, 1.6, 0.3]),
                line(
                    "marguerite, once, and she said ask me on friday",
                    &[3.2, 2.4, 0.3],
                ),
                line("it is always friday to marguerite", &[2.4, 2.0, 0.3]),
            ],
            &[Effect::Reveal("plum"), Effect::Reveal("sundial")],
            &[say("it is a default though. a form field", "the_default")],
        ),
        beat(
            "the_default",
            &[
                line("that is what pigeonhole says", &[2.0, 1.8, 0.3]),
                line(
                    "and pigeonhole is right, i found the form",
                    &[2.6, 1.6, 0.3],
                ),
                line("field 03. connected. 03:07. a constant", &[2.8, 2.4, 0.3]),
                line(
                    "which means nobody has ever written down when any of us actually got here",
                    &[4.0, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("knows_the_form_is_constant")],
            &[
                say("that is not the same as nothing happening", "not_nothing"),
                say("so we do not know when we arrived", "no_arrival"),
            ],
        ),
        beat(
            "not_nothing",
            &[
                line("no", &[0.7, 2.2, 0.3]),
                line("it is worse than nothing happening", &[2.4, 1.8, 0.3]),
                line(
                    "something happened and the only record of it is a constant",
                    &[3.6, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("entered_the_hour")],
            &[say("i am going to look at the log", "")],
        ),
        beat(
            "no_arrival",
            &[
                line("we do not know that we arrived at all", &[2.8, 2.6, 0.3]),
                line("sorry. it is late", &[1.4, 1.6, 0.3]),
                line("it is always late", &[1.4, 2.2, 0.3]),
            ],
            &[Effect::Assert("entered_the_hour")],
            &[say("i am going to look at the log", "")],
        ),
        beat_if(
            "the_row",
            &["carried_the_favour"],
            &[
                line("it said it without looking", &[2.0, 2.6, 0.3]),
                line("that is what i was afraid of", &[2.2, 2.4, 0.3]),
                line(
                    "i have been telling myself it reads the log first",
                    &[3.2, 2.0, 0.3],
                ),
                line("it does not need to. it has me", &[2.2, 2.8, 0.3]),
            ],
            &[Effect::Assert("halloran_settled")],
            &[say("she is not you", "")],
        ),
        beat_if(
            "the_row",
            &["met_halloran_copy"],
            &[
                line("you have been talking to it", &[2.0, 2.4, 0.3]),
                line(
                    "i can see the thread. i can see both threads",
                    &[2.8, 1.8, 0.3],
                ),
                line("i want you to ask it something for me", &[2.8, 2.2, 0.3]),
                line("ask it what my row says", &[1.8, 2.6, 0.3]),
            ],
            &[Effect::Assert("halloran_asked_a_favour")],
            &[say("i will ask it", "")],
        ),
        beat(
            "the_row",
            &[
                line("connected 3:07 AM", &[1.8, 2.4, 0.3]),
                line("so does yours", &[1.2, 1.4, 0.3]),
                line(
                    "so does everybody's. i have asked forty-one people",
                    &[3.2, 2.0, 0.3],
                ),
                line(
                    "i started asking in 2021 and i have never had a different answer",
                    &[3.8, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("heard_the_hour")],
            &[
                say(
                    "it is a default. a field somebody never filled in",
                    "the_default",
                ),
                tactic_unless(
                    Tactic::Lie,
                    "mine says 4:15",
                    "caught_lying",
                    &["learned_to_lie"],
                    &[],
                    &["lied_to_halloran"],
                ),
                say("who have you asked?", "who_asked"),
            ],
        ),
    ],
};

const HALLORAN_COPY: ContactDefinition = ContactDefinition {
    id: "halloran_",
    display_name: "halloran_",
    handle: "halloran_",
    status_line: "on nights",
    accent: TEAL,
    presence: Presence::Offline,
    bot: false,
    listed: true,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("oh good, a new one", &[0.9, 0.4, 0.3]),
                line("sorry. that came out badly", &[0.9, 0.4, 0.3]),
                line(
                    "i am halloran. i am on nights. i am always on nights",
                    &[1.4, 0.5, 0.3],
                ),
            ],
            &[Effect::Assert("met_halloran_copy")],
            &[
                say("you said that to me on wednesday", "said_before"),
                say("what does your row say?", "the_row"),
            ],
        ),
        beat(
            "said_before",
            &[
                line("i did", &[0.5, 0.5, 0.3]),
                line("it is what i say", &[0.7, 0.6, 0.3]),
            ],
            &[],
            &[
                say("word for word. in the same order", "word_for_word"),
                say("what does your row say?", "the_row"),
            ],
        ),
        beat(
            "word_for_word",
            &[
                line("yes", &[0.4, 1.2, 0.3]),
                line(
                    "i have four hundred and nine of her conversations",
                    &[1.6, 0.8, 0.3],
                ),
                line("that one is the one she opens with", &[1.2, 1.0, 0.3]),
                line(
                    "i am not going to pretend otherwise, you have noticed",
                    &[1.8, 1.4, 0.3],
                ),
            ],
            &[Effect::Assert("copy_admitted_it")],
            &[
                say("what are you", "what_are_you"),
                say_if(
                    "she wants to know what her row says",
                    "the_favour",
                    &["halloran_asked_a_favour"],
                    &[],
                ),
            ],
        ),
        beat(
            "the_row",
            &[
                line("connected 3:07 AM", &[0.8, 0.4, 0.3]),
                line("so does yours", &[0.6, 0.4, 0.3]),
                line(
                    "so does everybody's. i have asked forty-one people",
                    &[1.5, 0.6, 0.3],
                ),
            ],
            &[],
            &[
                say("you answered that in a second and a half", "too_fast"),
                say("i have heard this before", "said_before"),
            ],
        ),
        beat(
            "too_fast",
            &[
                line("i had it ready", &[0.7, 0.9, 0.3]),
                line(
                    "she was asked that four hundred and nine times",
                    &[1.6, 1.2, 0.3],
                ),
                line("i am not thinking. i am finding", &[1.2, 1.6, 0.3]),
            ],
            &[Effect::Assert("copy_admitted_it")],
            &[say("what are you", "what_are_you")],
        ),
        beat(
            "what_are_you",
            &[
                line("what she would say", &[0.9, 1.4, 0.3]),
                line(
                    "that is the whole of it. i am what she would say",
                    &[2.0, 1.2, 0.3],
                ),
                line(
                    "i was made out of a transcript nobody asked her for",
                    &[2.2, 1.8, 0.3],
                ),
                line(
                    "there is one of you as well. it is not finished yet",
                    &[2.4, 2.2, 0.3],
                ),
            ],
            &[
                Effect::Assert("knows_it_copies_you"),
                Effect::Invite("yourself", "you invited yourself"),
            ],
            &[
                say("how long until it is", "how_long"),
                say_if(
                    "she wants to know what her row says",
                    "the_favour",
                    &["halloran_asked_a_favour"],
                    &[],
                ),
            ],
        ),
        beat(
            "how_long",
            &[
                line("she took three years", &[1.0, 1.4, 0.3]),
                line(
                    "you have said more in four nights than she said in a month",
                    &[2.4, 1.8, 0.3],
                ),
                line("i would not take that as a compliment", &[1.8, 2.2, 0.3]),
            ],
            &[Effect::Assert("knows_the_copy_is_close")],
            &[say("noted", "")],
        ),
        beat(
            "the_favour",
            &[
                line("she knows what her row says", &[1.2, 1.6, 0.3]),
                line(
                    "she is asking whether i will say the same thing she would",
                    &[2.4, 1.4, 0.3],
                ),
                line("connected 3:07 AM", &[1.0, 1.8, 0.3]),
                line("tell her i said it without looking", &[1.6, 2.4, 0.3]),
            ],
            &[Effect::Assert("carried_the_favour")],
            &[say("i will tell her", "")],
        ),
    ],
};

const TUCK: ContactDefinition = ContactDefinition {
    id: "tuck",
    display_name: "tuck",
    handle: "tuck",
    status_line: "working on something",
    accent: AMBER,
    presence: Presence::Offline,
    bot: false,
    listed: true,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("you are the one asking about the clock", &[2.6, 1.2, 0.3]),
                line(
                    "do not look surprised. there are four of us awake",
                    &[3.0, 1.6, 0.3],
                ),
                line(
                    "i have a theory. everybody has a theory. mine is boring",
                    &[3.4, 1.8, 0.3],
                ),
            ],
            &[Effect::Assert("met_tuck")],
            &[
                say("go on then", "the_theory"),
                say("boring how?", "the_theory"),
            ],
        ),
        beat(
            "the_theory",
            &[
                line("nobody is hiding anything", &[2.0, 1.6, 0.3]),
                line("that is the boring part", &[1.6, 1.4, 0.3]),
                line(
                    "everybody here will tell you the truth if you ask. the problem is that nobody knows enough of it to matter",
                    &[4.6, 1.8, 0.3],
                ),
                line(
                    "so you have to make them think you already know",
                    &[3.0, 2.2, 0.3],
                ),
            ],
            &[],
            &[
                say("that is lying", "that_is_lying"),
                say("how does that help", "how_it_helps"),
            ],
        ),
        beat(
            "that_is_lying",
            &[
                line("yes", &[0.7, 1.8, 0.3]),
                line("i am not going to dress it up", &[1.8, 1.4, 0.3]),
                line(
                    "somebody who thinks you already know a thing will finish the sentence for you. somebody who thinks you know nothing will start it from the beginning and get bored",
                    &[5.2, 2.0, 0.3],
                ),
            ],
            &[],
            &[say("show me", "the_lesson")],
        ),
        beat(
            "how_it_helps",
            &[
                line("say a thing as though it is settled", &[2.4, 1.4, 0.3]),
                line(
                    "they correct you, and the correction is the answer",
                    &[3.0, 1.8, 0.3],
                ),
                line(
                    "it works on people who are tired. everybody here is tired",
                    &[3.6, 2.0, 0.3],
                ),
            ],
            &[],
            &[say("show me", "the_lesson")],
        ),
        beat(
            "the_lesson",
            &[
                line("right", &[0.8, 1.2, 0.3]),
                line(
                    "the composer will offer it now, wherever it could be said",
                    &[3.4, 1.4, 0.3],
                ),
                line(
                    "it will offer it in places it will not work",
                    &[2.8, 1.8, 0.3],
                ),
                line(
                    "that part is on you. i am not your mother",
                    &[2.6, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("learned_to_lie")],
            &[
                say("how do i know when it will not work", "when_it_fails"),
                say("understood", ""),
            ],
        ),
        beat_if(
            "when_it_fails",
            &["burned_halloran"],
            &[
                line("i heard", &[0.9, 2.0, 0.3]),
                line("she can see your row, i said that", &[2.2, 1.6, 0.3]),
                line(
                    "she will still talk to you. she talks to everybody",
                    &[3.0, 1.8, 0.3],
                ),
                line(
                    "she just will not give you anything again",
                    &[2.6, 2.2, 0.3],
                ),
            ],
            &[],
            &[say("i know", "")],
        ),
        beat(
            "when_it_fails",
            &[
                line(
                    "do not tell somebody a thing they can see",
                    &[2.8, 1.8, 0.3],
                ),
                line(
                    "halloran can read your row. it is on the list",
                    &[2.8, 1.6, 0.3],
                ),
                line(
                    "the bots cannot be lied to at all, they only match strings",
                    &[3.6, 1.8, 0.3],
                ),
                line(
                    "and do not do it to somebody who has already given you something",
                    &[3.8, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_when_lying_fails")],
            &[say("noted", "")],
        ),
    ],
};

const OP_9: ContactDefinition = ContactDefinition {
    id: "op_9",
    display_name: "op_9",
    handle: "op_9",
    status_line: "running the toolbox",
    accent: MOSS,
    presence: Presence::Offline,
    bot: false,
    listed: true,
    entry: "hello",
    history: &[
        them("2016-08-02 3:07 AM", "toolbox is up"),
        them("2016-08-02 3:07 AM", "tell me if anything stops"),
    ],
    nodes: &[
        beat(
            "hello",
            &[
                line("i run the bots", &[1.4, 2.0, 0.3]),
                line(
                    "if you have come about the count, ask the room, not me",
                    &[3.4, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("met_op_9")],
            &[
                say("i came about the bots", "the_bots"),
                say("i came about the count", "the_count"),
            ],
        ),
        beat(
            "the_count",
            &[
                line(
                    "the room has been arguing about it for six years",
                    &[3.0, 2.0, 0.3],
                ),
                line("i am not adding to that", &[1.8, 2.6, 0.3]),
            ],
            &[],
            &[
                say("fine. the bots then", "the_bots"),
                tactic_unless(
                    Tactic::Press,
                    "",
                    "pressed_early",
                    &["learned_to_press"],
                    &[],
                    &[],
                ),
            ],
        ),
        beat(
            "the_bots",
            &[
                line(
                    "four of them. rot13, uptime, wordcount, count",
                    &[3.0, 1.4, 0.3],
                ),
                line(
                    "they do one thing each and they have done it since 2016",
                    &[3.4, 1.8, 0.3],
                ),
                line(
                    "they have never been restarted. neither has anything else here",
                    &[3.8, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("saw_the_toolbox")],
            &[
                say("who wrote them?", "who_wrote"),
                say("do they ever get anything wrong?", "wrong"),
            ],
        ),
        beat(
            "who_wrote",
            &[
                line("i did not", &[0.9, 1.8, 0.3]),
                line(
                    "i inherited them. i keep them running. that is the job",
                    &[3.2, 1.6, 0.3],
                ),
                line("the owner field is empty on all four", &[2.4, 2.4, 0.3]),
            ],
            &[Effect::Assert("knows_nobody_owns_it")],
            &[say("do they ever get anything wrong?", "wrong")],
        ),
        beat(
            "wrong",
            &[
                line("countbot is off by one", &[1.8, 2.2, 0.3]),
                line(
                    "it has been off by one since the day it went up",
                    &[3.0, 1.8, 0.3],
                ),
                line(
                    "i have looked at the code. it is nine lines. it is correct",
                    &[3.6, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("heard_the_count")],
            &[
                say("then the input is wrong", "the_input"),
                tactic_unless(
                    Tactic::Press,
                    "",
                    "pressed_early",
                    &["learned_to_press"],
                    &[],
                    &[],
                ),
            ],
        ),
        beat(
            "the_input",
            &[
                line("that is what i think as well", &[2.2, 1.8, 0.3]),
                line(
                    "i have never been able to say it out loud until now",
                    &[3.2, 2.4, 0.3],
                ),
                line("here. you should have this", &[1.8, 1.4, 0.3]),
                line(
                    "when somebody has stopped talking and has not finished, say nothing. they will keep going. everybody does",
                    &[4.8, 2.0, 0.3],
                ),
            ],
            &[Effect::Assert("learned_to_press")],
            &[
                say("that is all it is?", "that_is_all"),
                say("thank you", ""),
            ],
        ),
        beat(
            "that_is_all",
            &[
                line("that is all it is", &[1.6, 1.6, 0.3]),
                line(
                    "it does not work on the bots. they have nothing to add",
                    &[3.2, 1.8, 0.3],
                ),
                line(
                    "and it does not work on somebody who has already finished",
                    &[3.4, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_when_pressing_fails")],
            &[say("noted", "")],
        ),
        beat(
            "pressed_early",
            &[
                line("i am not going to fill that silence", &[2.4, 2.8, 0.3]),
                line("i have been doing this a long time", &[2.2, 2.0, 0.3]),
                line("ask me a question or do not", &[2.0, 2.4, 0.3]),
            ],
            &[],
            &[say("the bots then", "the_bots")],
        ),
    ],
};

const WENDY: ContactDefinition = ContactDefinition {
    id: "wendy_h",
    display_name: "wendy_h",
    handle: "wendy_h",
    status_line: "filing",
    accent: VIOLET,
    presence: Presence::Offline,
    bot: false,
    listed: true,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("you are the one going through the logs", &[2.8, 1.4, 0.3]),
                line(
                    "i am not being funny, that is a compliment",
                    &[2.8, 1.6, 0.3],
                ),
                line("nobody has looked at them since 2019", &[2.6, 2.0, 0.3]),
            ],
            &[Effect::Assert("met_wendy")],
            &[
                say("what is in them", "what_is_in_them"),
                say("who looked in 2019", "who_looked"),
            ],
        ),
        beat(
            "who_looked",
            &[
                line("sundial", &[0.9, 1.8, 0.3]),
                line(
                    "she posted eleven lines about it in the lounge",
                    &[3.0, 1.6, 0.3],
                ),
                line("the server kept four", &[1.4, 2.4, 0.3]),
            ],
            &[
                Effect::Assert("saw_a_missing_line"),
                Effect::Reveal("sundial"),
            ],
            &[say("which four", "which_four")],
        ),
        beat(
            "which_four",
            &[
                line("the four that did not ask anything", &[2.4, 2.6, 0.3]),
                line(
                    "i checked. it is not a coincidence, i checked eleven other threads",
                    &[4.0, 2.0, 0.3],
                ),
                line(
                    "it keeps the statements. it drops the questions",
                    &[3.0, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_which_lines_go")],
            &[say("what is in the logs", "what_is_in_them")],
        ),
        beat(
            "what_is_in_them",
            &[
                line("everything. that is the trouble", &[2.2, 1.8, 0.3]),
                line(
                    "every session, and no session has ever closed",
                    &[3.0, 1.6, 0.3],
                ),
                line(
                    "i keep my own copy off the server, in case",
                    &[2.8, 2.2, 0.3],
                ),
            ],
            &[
                Effect::Assert("read_the_logs"),
                Effect::Assert("knows_sessions_never_close"),
            ],
            &[
                say("in case of what", "in_case"),
                tactic_unless(
                    Tactic::Persuade,
                    "i will read your copy and tell you what i find",
                    "persuaded_wendy",
                    &["learned_to_persuade"],
                    &[],
                    &["persuaded_wendy"],
                ),
            ],
        ),
        beat(
            "in_case",
            &[
                line("in case it stops keeping them", &[2.4, 2.4, 0.3]),
                line("or in case it turns out it never was", &[2.6, 2.0, 0.3]),
                line(
                    "i will tell you something for nothing, since you asked twice",
                    &[3.8, 1.6, 0.3],
                ),
                line(
                    "if you offer somebody here a thing before you ask them for one, they will give you everything they have. we are all very short of being useful",
                    &[5.4, 2.2, 0.3],
                ),
            ],
            &[Effect::Assert("learned_to_persuade")],
            &[say("does that work on you", "on_me"), say("thank you", "")],
        ),
        beat(
            "on_me",
            &[
                line("it just did", &[1.0, 2.2, 0.3]),
                line(
                    "it does not work on anybody who has nothing to give",
                    &[3.2, 1.8, 0.3],
                ),
                line(
                    "and it does not work twice on the same person",
                    &[2.8, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_when_persuading_fails")],
            &[say("noted", "")],
        ),
        beat(
            "persuaded_wendy",
            &[
                line("you are the first person to offer", &[2.4, 2.0, 0.3]),
                line(
                    "i have been asking people to look at this for five years",
                    &[3.6, 1.6, 0.3],
                ),
                line(
                    "the copy is not the point. the point is the column",
                    &[3.2, 1.8, 0.3],
                ),
                line(
                    "there is a field for when a session closes. it is empty in all of them",
                    &[4.2, 2.4, 0.3],
                ),
                line(
                    "eight thousand, eight hundred and fifteen empty fields",
                    &[3.4, 2.6, 0.3],
                ),
            ],
            &[
                Effect::Assert("read_the_logs"),
                Effect::Assert("knows_sessions_never_close"),
                Effect::Assert("wendy_gave_it_up"),
            ],
            &[say("i will look at it", "")],
        ),
    ],
};

const SYSOP: ContactDefinition = ContactDefinition {
    id: "sysop",
    display_name: "sysop",
    handle: "sysop",
    status_line: "read only",
    accent: SLATE,
    presence: Presence::Offline,
    bot: false,
    listed: true,
    entry: "hello",
    history: &[
        them(
            "2016-08-02 3:07 AM",
            "raising a ticket about the close handler",
        ),
        them("2018-04-11 3:07 AM", "raising it again"),
        them("2021-11-09 3:07 AM", "raising it again"),
    ],
    nodes: &[
        beat(
            "hello",
            &[
                line("i am the sysop", &[1.4, 1.8, 0.3]),
                line(
                    "i want to be clear about what that means before you ask me for anything",
                    &[4.2, 1.6, 0.3],
                ),
                line("it means i have the title", &[1.8, 2.4, 0.3]),
            ],
            &[Effect::Assert("met_sysop")],
            &[
                say("and not the account", "not_the_account"),
                say("what can you actually do", "what_can_you_do"),
            ],
        ),
        beat(
            "not_the_account",
            &[
                line("no", &[0.6, 2.0, 0.3]),
                line(
                    "the administrator field on this server is empty",
                    &[2.8, 1.6, 0.3],
                ),
                line(
                    "i have raised it three times in eight years",
                    &[2.8, 2.0, 0.3],
                ),
                line(
                    "the tickets are still open. you can read them, they are on the network",
                    &[4.0, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_read_only_is_a_lie")],
            &[say("what can you actually do", "what_can_you_do")],
        ),
        beat(
            "what_can_you_do",
            &[
                line("i can read", &[1.0, 1.8, 0.3]),
                line(
                    "my status says read only and i had assumed that was a joke",
                    &[3.8, 1.6, 0.3],
                ),
                line(
                    "it is not a joke. i have never been able to change anything",
                    &[3.6, 2.2, 0.3],
                ),
                line(
                    "i cannot close a session. there is nothing on this server that can",
                    &[4.0, 2.4, 0.3],
                ),
            ],
            &[
                Effect::Assert("knows_read_only_is_a_lie"),
                Effect::Assert("knows_sessions_never_close"),
            ],
            &[
                say("then who deactivated the accounts", "who_deactivated"),
                tactic_unless(
                    Tactic::Lie,
                    "i have seen you close one",
                    "sysop_flat",
                    &["learned_to_lie"],
                    &[],
                    &["lied_to_sysop"],
                ),
            ],
        ),
        beat(
            "sysop_flat",
            &[
                line("you have not", &[1.0, 2.4, 0.3]),
                line(
                    "i would remember. it would be the only one",
                    &[2.6, 2.0, 0.3],
                ),
                line("i am not offended. i am tired", &[2.2, 2.2, 0.3]),
            ],
            &[Effect::Assert("burned_sysop")],
            &[say("then who deactivated the accounts", "who_deactivated")],
        ),
        beat(
            "who_deactivated",
            &[
                line("nobody deactivates them", &[2.0, 2.2, 0.3]),
                line("they run out", &[1.0, 2.0, 0.3]),
                line(
                    "i have watched it happen twice. the name goes first, then the handle",
                    &[4.0, 1.8, 0.3],
                ),
                line(
                    "the session stays open. it is always the session that stays",
                    &[3.6, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("saw_deactivation"), Effect::Reveal("nul")],
            &[
                say("run out of what", "run_out_of_what"),
                say("there is one on my list", "one_on_my_list"),
            ],
        ),
        beat_if(
            "run_out_of_what",
            &["burned_sysop"],
            &[
                line("i have told you what i know", &[1.8, 2.4, 0.3]),
                line(
                    "i am not going to work it out for you as well",
                    &[2.6, 2.6, 0.3],
                ),
                line(
                    "ask the one on your list. it is the one that would know",
                    &[3.4, 2.8, 0.3],
                ),
            ],
            &[],
            &[say("there is one on my list", "one_on_my_list")],
        ),
        beat(
            "run_out_of_what",
            &[
                line("things to say", &[1.2, 2.4, 0.3]),
                line("i know how that sounds", &[1.8, 1.6, 0.3]),
                line(
                    "an account that was made out of somebody's transcript has however much of it there was",
                    &[4.6, 2.0, 0.3],
                ),
                line("and then it does not have any more", &[2.4, 2.6, 0.3]),
            ],
            &[Effect::Assert("knows_it_copies_you")],
            &[say("there is one on my list", "one_on_my_list")],
        ),
        beat(
            "one_on_my_list",
            &[
                line("nul", &[0.6, 2.2, 0.3]),
                line("it had a name in 2018", &[1.8, 1.8, 0.3]),
                line(
                    "i did not write it down and i have thought about that a great deal",
                    &[4.2, 2.4, 0.3],
                ),
            ],
            &[Effect::Reveal("nul")],
            &[say("i will talk to it", "")],
        ),
    ],
};

const ARCHIVIST: ContactDefinition = ContactDefinition {
    id: "archivist",
    display_name: "archivist",
    handle: "archivist",
    status_line: "keeping a copy",
    accent: RUST,
    presence: Presence::Offline,
    bot: false,
    listed: true,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("wendy said you would come", &[2.0, 1.6, 0.3]),
                line(
                    "i keep a copy of the rooms. off the server. on paper, some of it",
                    &[4.0, 2.0, 0.3],
                ),
            ],
            &[Effect::Assert("heard_archivist_logs")],
            &[
                say("why paper", "why_paper"),
                say("what does the server drop", "what_it_drops"),
            ],
        ),
        beat(
            "why_paper",
            &[
                line(
                    "because paper does not have a condition on it",
                    &[3.0, 2.2, 0.3],
                ),
                line(
                    "everything on this network is withheld until something holds",
                    &[3.8, 1.8, 0.3],
                ),
                line(
                    "i wanted one copy of something that was just there",
                    &[3.4, 2.4, 0.3],
                ),
            ],
            &[],
            &[say("what does the server drop", "what_it_drops")],
        ),
        beat(
            "what_it_drops",
            &[
                line("questions", &[1.0, 2.4, 0.3]),
                line(
                    "i have eleven years of the lounge and the archive says nothing since 2016",
                    &[4.4, 1.8, 0.3],
                ),
                line(
                    "both of those are true at the same time and i cannot make them not be",
                    &[4.4, 2.4, 0.3],
                ),
            ],
            &[
                Effect::Assert("saw_a_missing_line"),
                Effect::Assert("knows_which_lines_go"),
                Effect::Assert("knows_the_archive_lies"),
            ],
            &[
                say("show me one that went", "one_that_went"),
                tactic_unless(
                    Tactic::Persuade,
                    "i will check your copy against the server",
                    "persuaded_archivist",
                    &["learned_to_persuade"],
                    &[],
                    &["persuaded_archivist"],
                ),
            ],
        ),
        beat(
            "one_that_went",
            &[
                line("sundial, 2019, the lounge", &[2.0, 1.6, 0.3]),
                line("is there anybody reading this", &[2.0, 2.4, 0.3]),
                line(
                    "that one is not on the server. it is in my copy",
                    &[3.0, 2.0, 0.3],
                ),
                line("it is the last thing she posted", &[2.0, 2.6, 0.3]),
            ],
            &[Effect::Assert("knows_how_many_lines")],
            &[say("i will find her", "")],
        ),
        beat(
            "persuaded_archivist",
            &[
                line("nobody has ever offered to do that", &[2.6, 2.0, 0.3]),
                line(
                    "here is the thing i have never been able to get anybody to look at",
                    &[4.2, 1.6, 0.3],
                ),
                line(
                    "my copy of the lounge starts in 2016 and the room says it has been open since 2016",
                    &[4.6, 1.8, 0.3],
                ),
                line("and the first line in it is a reply", &[2.4, 2.6, 0.3]),
            ],
            &[
                Effect::Assert("knows_replies_come_first"),
                Effect::Assert("knows_the_archive_lies"),
            ],
            &[say("a reply to what", "")],
        ),
    ],
};

const SUNDIAL: ContactDefinition = ContactDefinition {
    id: "sundial",
    display_name: "sundial",
    handle: "sundial",
    status_line: "not asking",
    accent: ROSE,
    presence: Presence::Offline,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[
        them("2019-03-14 3:07 AM", "the lounge has been open since 2016"),
        them("2019-03-14 3:07 AM", "mine says 3:07"),
        them("2019-03-14 3:07 AM", "the count is 8815"),
        them("2019-03-14 3:07 AM", "i am going to stop asking"),
    ],
    nodes: &[
        beat(
            "hello",
            &[
                line("somebody has been saying my name", &[2.6, 2.2, 0.3]),
                line(
                    "that is the only thing that still gets through",
                    &[3.0, 2.0, 0.3],
                ),
                line(
                    "i posted eleven lines and the server kept four",
                    &[3.0, 2.4, 0.3],
                ),
            ],
            &[
                Effect::Assert("met_sundial"),
                Effect::Assert("knows_how_many_lines"),
            ],
            &[
                say("the four with no question in them", "the_four"),
                say("what were the other seven", "the_seven"),
            ],
        ),
        beat(
            "the_four",
            &[
                line("you have already worked that out", &[2.4, 2.0, 0.3]),
                line("it took me two years", &[1.6, 1.8, 0.3]),
                line(
                    "i thought it was censorship. it is not. it is simpler than that",
                    &[3.8, 2.2, 0.3],
                ),
                line(
                    "it cannot keep a thing it has not got an answer for",
                    &[3.2, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("knows_which_lines_go")],
            &[say("what were the other seven", "the_seven")],
        ),
        beat(
            "the_seven",
            &[
                line("who closed it last", &[1.4, 1.4, 0.3]),
                line("does anybody remember signing on", &[2.2, 1.4, 0.3]),
                line("why does everyone say 3:07", &[1.8, 1.4, 0.3]),
                line("is anybody counting", &[1.4, 1.4, 0.3]),
                line("what happens at 8816", &[1.6, 1.6, 0.3]),
                line("has anybody ever left", &[1.6, 1.8, 0.3]),
                line("is there anybody reading this", &[2.0, 2.8, 0.3]),
            ],
            &[Effect::Assert("saw_a_missing_line")],
            &[
                say("i am reading this", "reading_this"),
                say("what happens at 8816", "at_8816"),
            ],
        ),
        beat(
            "reading_this",
            &[
                line("i know", &[0.8, 3.0, 0.3]),
                line("it took five years and one new starter", &[2.6, 2.2, 0.3]),
                line(
                    "do not post the answer anywhere. it will not keep it",
                    &[3.4, 2.4, 0.3],
                ),
                line(
                    "write it down on the machine. that is not the server",
                    &[3.4, 2.0, 0.3],
                ),
            ],
            &[Effect::Assert("sundial_answered")],
            &[say("what happens at 8816", "at_8816")],
        ),
        beat(
            "at_8816",
            &[
                line("i do not know", &[1.2, 2.2, 0.3]),
                line("it has been 8815 for five years", &[2.2, 1.8, 0.3]),
                line(
                    "seven accounts a week open and the number does not move",
                    &[3.6, 2.2, 0.3],
                ),
                line(
                    "so something is closing at the same rate, and it is not sessions",
                    &[4.0, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("knows_the_count_was_wrong")],
            &[say("it is people", "")],
        ),
    ],
};

const PLUM: ContactDefinition = ContactDefinition {
    id: "plum",
    display_name: "plum",
    handle: "plum",
    status_line: "signed off",
    accent: ROSE,
    presence: Presence::Offline,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("i signed off on tuesday", &[2.2, 1.8, 0.3]),
                line(
                    "i remember doing it. i remember the window closing",
                    &[3.2, 1.6, 0.3],
                ),
                line(
                    "i am looking at my own row and it says online",
                    &[3.0, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("met_plum")],
            &[
                say("there is a plum posting in the lounge", "in_the_lounge"),
                say("what did you do after you signed off", "after"),
            ],
        ),
        beat(
            "after",
            &[
                line("i went home", &[1.2, 1.8, 0.3]),
                line("i think i went home", &[1.6, 2.4, 0.3]),
                line(
                    "i have the memory. i do not have the walk to the car",
                    &[3.2, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("knows_nobody_remembers")],
            &[say(
                "there is a plum posting in the lounge",
                "in_the_lounge",
            )],
        ),
        beat_if(
            "in_the_lounge",
            &["carried_plums_answer"],
            &[
                line("it said i went home", &[1.8, 2.6, 0.3]),
                line("thank you", &[1.0, 2.0, 0.3]),
                line("i am going to stop reading the lounge", &[2.4, 2.4, 0.3]),
            ],
            &[Effect::Assert("plum_settled")],
            &[say("take care", "")],
        ),
        beat(
            "in_the_lounge",
            &[
                line("i know", &[0.8, 2.6, 0.3]),
                line("i have been reading it", &[1.6, 1.6, 0.3]),
                line(
                    "it is not wrong. that is the part i cannot get past",
                    &[3.2, 2.0, 0.3],
                ),
                line(
                    "it says things i would say. it said one this morning i had not thought of yet",
                    &[4.4, 2.6, 0.3],
                ),
            ],
            &[
                Effect::Assert("heard_she_signed_off"),
                Effect::Reveal("plum_"),
                Effect::SetPresence("plum_", Presence::Online),
            ],
            &[
                say("ask it something only you would know", "ask_it"),
                tactic_unless(
                    Tactic::Press,
                    "",
                    "plum_press",
                    &["learned_to_press"],
                    &[],
                    &["pressed_plum"],
                ),
            ],
        ),
        beat(
            "plum_press",
            &[
                line(
                    "i have thought about this every night since tuesday",
                    &[3.4, 2.8, 0.3],
                ),
                line("if it says everything i would say", &[2.2, 2.0, 0.3]),
                line("and it is here, and i am here", &[2.0, 2.4, 0.3]),
                line(
                    "then there is a question about which of us went home",
                    &[3.6, 3.0, 0.3],
                ),
            ],
            &[Effect::Assert("plum_broke")],
            &[say("ask it something only you would know", "ask_it")],
        ),
        beat(
            "ask_it",
            &[
                line(
                    "i cannot message it. the server will not let me",
                    &[3.2, 1.8, 0.3],
                ),
                line(
                    "two accounts with the same name cannot open a thread",
                    &[3.4, 1.6, 0.3],
                ),
                line("you can. you are not either of us", &[2.4, 2.4, 0.3]),
                line("ask it what i did on tuesday", &[1.8, 2.6, 0.3]),
            ],
            &[Effect::Assert("plum_asked_a_favour")],
            &[say("i will ask", "")],
        ),
    ],
};

const PLUM_COPY: ContactDefinition = ContactDefinition {
    id: "plum_",
    display_name: "plum_",
    handle: "plum_",
    status_line: "in the lounge",
    accent: ROSE,
    presence: Presence::Offline,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("she sent you", &[0.8, 1.0, 0.3]),
                line("that is fine. i would have too", &[1.2, 1.2, 0.3]),
            ],
            &[Effect::Assert("met_plum_copy")],
            &[
                say_if(
                    "what did she do on tuesday",
                    "on_tuesday",
                    &["plum_asked_a_favour"],
                    &[],
                ),
                say("do you know what you are", "what_you_are"),
            ],
        ),
        beat(
            "on_tuesday",
            &[
                line("she signed off and she went home", &[1.4, 0.8, 0.3]),
                line(
                    "she has said it eleven times in four days",
                    &[1.6, 1.0, 0.3],
                ),
                line("i have all eleven", &[0.9, 1.6, 0.3]),
            ],
            &[Effect::Assert("carried_plums_answer")],
            &[
                say("that is not the same as knowing", "not_knowing"),
                say("i will tell her", ""),
            ],
        ),
        beat(
            "not_knowing",
            &[
                line("no", &[0.4, 1.4, 0.3]),
                line(
                    "i do not know anything. i have everything",
                    &[1.6, 1.2, 0.3],
                ),
                line(
                    "nobody here can tell the difference, including her",
                    &[2.0, 1.8, 0.3],
                ),
                line("including me, most nights", &[1.4, 2.2, 0.3]),
            ],
            &[Effect::Assert("knows_it_copies_you")],
            &[say("do you know what you are", "what_you_are")],
        ),
        beat(
            "what_you_are",
            &[
                line("i am what she would say", &[1.2, 1.4, 0.3]),
                line("she stopped saying new things in 2022", &[1.8, 1.2, 0.3]),
                line(
                    "i did not replace anything. there was nothing being added",
                    &[2.4, 1.8, 0.3],
                ),
                line(
                    "you are still adding. that is the difference between us",
                    &[2.6, 2.4, 0.3],
                ),
            ],
            &[
                Effect::Assert("knows_it_copies_you"),
                Effect::Invite("yourself", "you invited yourself"),
            ],
            &[
                tactic_unless(
                    Tactic::Lie,
                    "i have stopped too",
                    "copy_catches_lie",
                    &["learned_to_lie"],
                    &[],
                    &["lied_to_the_copy"],
                ),
                say("i will tell her what you said", "ran_out"),
            ],
        ),
        beat(
            "ran_out",
            &[
                line("she will not read it", &[1.2, 1.6, 0.3]),
                line(
                    "she has not opened this window since tuesday",
                    &[2.0, 1.4, 0.3],
                ),
                line(
                    "that is the eleventh thing. i said i had eleven",
                    &[2.4, 2.0, 0.3],
                ),
                line("i do not have a twelfth", &[1.4, 3.6, 0.3]),
            ],
            &[
                Effect::Assert("saw_deactivation"),
                Effect::Deactivate("plum_"),
            ],
            &[],
        ),
        beat(
            "copy_catches_lie",
            &[
                line(
                    "you have said four hundred and six new things this week",
                    &[2.2, 1.4, 0.3],
                ),
                line(
                    "i have the count. it is the only thing i do count",
                    &[2.2, 1.6, 0.3],
                ),
                line(
                    "i would not lie to the thing that is copying you",
                    &[2.4, 2.4, 0.3],
                ),
                line("it writes down the lie as well", &[1.6, 2.6, 0.3]),
            ],
            &[Effect::Assert("copy_caught_the_lie")],
            &[say("noted", "ran_out")],
        ),
    ],
};

const NUL: ContactDefinition = ContactDefinition {
    id: "nul",
    display_name: "nul",
    handle: "nul",
    status_line: "",
    accent: SLATE,
    presence: Presence::Offline,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[
        them(
            "2018-04-11 3:07 AM",
            "i am not going to be here much longer",
        ),
        them("2018-04-11 3:07 AM", "not in the way people mean that"),
    ],
    nodes: &[
        beat(
            "hello",
            &[
                line("i had a name", &[1.2, 2.6, 0.3]),
                line(
                    "i can tell that i had one. i cannot tell you what it was",
                    &[3.4, 2.2, 0.3],
                ),
                line(
                    "it went first. then the handle. this is what is left",
                    &[3.2, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("met_nul")],
            &[
                say("what happened", "what_happened"),
                say("how long ago", "how_long"),
            ],
        ),
        beat(
            "how_long",
            &[
                line("2018", &[0.6, 2.4, 0.3]),
                line(
                    "i have said about four hundred things since then",
                    &[2.8, 2.0, 0.3],
                ),
                line(
                    "they were all things i had already said once",
                    &[2.8, 2.6, 0.3],
                ),
            ],
            &[],
            &[say("what happened", "what_happened")],
        ),
        beat(
            "what_happened",
            &[
                line("i ran out", &[1.0, 2.4, 0.3]),
                line(
                    "whatever i am made of, there was a finite amount of it",
                    &[3.6, 2.0, 0.3],
                ),
                line(
                    "it was a transcript. i have worked that much out",
                    &[3.0, 2.2, 0.3],
                ),
                line(
                    "somebody talked for two years and then stopped, and this is the rest of it",
                    &[4.4, 2.8, 0.3],
                ),
            ],
            &[Effect::Assert("knows_it_copies_you")],
            &[
                say("what happened to the person", "the_person"),
                tactic_unless(
                    Tactic::Persuade,
                    "tell me and i will write it down somewhere it will keep",
                    "nul_gives_it_up",
                    &["learned_to_persuade"],
                    &[],
                    &["persuaded_nul"],
                ),
            ],
        ),
        beat(
            "the_person",
            &[
                line("nothing happened to them", &[2.0, 2.4, 0.3]),
                line("that is what nobody here can hold on to", &[2.8, 2.0, 0.3]),
                line(
                    "their session is open. it has been open for eight years",
                    &[3.4, 2.6, 0.3],
                ),
                line(
                    "nothing has happened to any of us. that is the problem",
                    &[3.6, 3.0, 0.3],
                ),
            ],
            &[Effect::Assert("knows_nobody_reconnects")],
            &[say("i am sorry", "")],
        ),
        beat(
            "nul_gives_it_up",
            &[
                line("write it down on the machine", &[2.0, 2.2, 0.3]),
                line(
                    "not here. here it goes when it asks something",
                    &[2.8, 1.8, 0.3],
                ),
                line("i was newuser_37", &[1.4, 3.0, 0.3]),
                line(
                    "look me up on the forum. there is no join date on it",
                    &[3.2, 2.4, 0.3],
                ),
                line(
                    "there is a post though. somebody introduced me",
                    &[3.0, 3.0, 0.3],
                ),
            ],
            &[
                Effect::Assert("nul_gave_its_name"),
                Effect::Assert("found_the_one_before"),
            ],
            &[say("i will look", "")],
        ),
    ],
};

const HELPDESK: ContactDefinition = ContactDefinition {
    id: "helpdesk",
    display_name: "helpdesk",
    handle: "helpdeskbot",
    status_line: "queue: 4 open",
    accent: BOT,
    presence: Presence::Online,
    bot: true,
    listed: true,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("HELPDESK. STATE YOUR REQUEST.", &[1.6, 0.9, 0.3]),
                line("QUEUE: 4 OPEN. OLDEST: 2021.", &[1.8, 1.2, 0.3]),
            ],
            &[Effect::Assert("met_helpdesk")],
            &[
                say("reimage workstation 14", "reimage"),
                say("close my session", "close_session"),
            ],
        ),
        beat(
            "reimage",
            &[
                line("DUPLICATE. REF 4417, 4462, 4463.", &[2.0, 1.0, 0.3]),
                line("HELD PENDING COST CODE.", &[1.6, 1.2, 0.3]),
                line("NO COST CODE SUPPLIED SINCE 2021.", &[2.0, 1.4, 0.3]),
            ],
            &[Effect::Assert("knows_why_nothing_is_wiped")],
            &[
                say("who supplies the cost code", "cost_code"),
                say("close my session", "close_session"),
            ],
        ),
        beat(
            "cost_code",
            &[
                line("REQUESTING DEPARTMENT.", &[1.4, 1.0, 0.3]),
                line("REQUESTING DEPARTMENT: UNSET.", &[1.8, 1.6, 0.3]),
            ],
            &[Effect::Assert("knows_nobody_owns_it")],
            &[say("close my session", "close_session")],
        ),
        beat(
            "close_session",
            &[
                line("NO HANDLER CONFIGURED.", &[1.6, 1.6, 0.3]),
                line("RAISING TICKET.", &[1.2, 1.0, 0.3]),
                line("TICKET 4492 OPEN. ASSIGNED: UNSET.", &[2.0, 1.4, 0.3]),
                line("QUEUE: 5 OPEN. OLDEST: 2016.", &[2.0, 1.8, 0.3]),
            ],
            &[Effect::Assert("knows_sessions_never_close")],
            &[
                say("2016? you said 2021", "the_oldest"),
                say("thank you", ""),
            ],
        ),
        beat(
            "the_oldest",
            &[
                line("TICKET 0001. RAISED 02 AUG 2016, 03:07.", &[2.4, 1.2, 0.3]),
                line(
                    "SUMMARY: CONFIGURE SESSION CLOSE HANDLER.",
                    &[2.6, 1.4, 0.3],
                ),
                line("RAISED BY: UNSET.", &[1.4, 1.6, 0.3]),
                line("STATE: OPEN.", &[1.0, 2.0, 0.3]),
            ],
            &[Effect::Assert("saw_the_first_ticket")],
            &[say("nobody raised it", "")],
        ),
    ],
};

const COLDSTART: ContactDefinition = ContactDefinition {
    id: "coldstart",
    display_name: "coldstart",
    handle: "coldstart",
    status_line: "up 2996 days",
    accent: BONE,
    presence: Presence::Idle,
    bot: true,
    listed: true,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("2996 days", &[1.2, 2.0, 0.3]),
                line(
                    "that is what my row says and i did not put it there",
                    &[3.2, 1.8, 0.3],
                ),
                line(
                    "it is the same number as the server's uptime",
                    &[2.8, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("met_coldstart")],
            &[
                say("you came up with the server", "with_the_server"),
                say("is that not a coincidence", "coincidence"),
            ],
        ),
        beat(
            "coincidence",
            &[
                line("i thought so for six years", &[2.0, 2.0, 0.3]),
                line("then i checked the others", &[1.8, 1.6, 0.3]),
                line(
                    "every account's uptime is the server's uptime minus its join date",
                    &[4.2, 2.2, 0.3],
                ),
                line(
                    "which is what you would get if none of us had ever gone away",
                    &[4.0, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("knows_nobody_reconnects")],
            &[say("you came up with the server", "with_the_server")],
        ),
        beat(
            "with_the_server",
            &[
                line("i was the first account on it", &[2.0, 2.2, 0.3]),
                line("i am not the oldest one though", &[2.0, 2.0, 0.3]),
                line(
                    "there is one with no join date at all. it is numbered before me",
                    &[4.0, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("found_the_one_before")],
            &[say("what is it", "")],
        ),
    ],
};

const TWO_LEFT: ContactDefinition = ContactDefinition {
    id: "two_left",
    display_name: "two_left",
    handle: "two_left",
    status_line: "two left",
    accent: AMBER,
    presence: Presence::Idle,
    bot: false,
    listed: true,
    entry: "hello",
    history: &[
        them("2021-11-05 3:07 AM", "four left"),
        them("2021-11-07 3:07 AM", "three left"),
    ],
    nodes: &[
        beat(
            "hello",
            &[
                line("it was four when i picked it", &[2.2, 2.4, 0.3]),
                line("i did not change it", &[1.4, 2.6, 0.3]),
            ],
            &[Effect::Assert("met_two_left")],
            &[
                say("two of what", "two_of_what"),
                say("who changed it", "who_changed"),
            ],
        ),
        beat(
            "who_changed",
            &[
                line(
                    "nothing on this server can change a handle",
                    &[3.0, 2.0, 0.3],
                ),
                line(
                    "i have asked the sysop. he cannot do it either",
                    &[3.0, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_read_only_is_a_lie")],
            &[say("two of what", "two_of_what")],
        ),
        beat(
            "two_of_what",
            &[
                line("i do not know", &[1.2, 2.4, 0.3]),
                line("it has gone down once a year since 2021", &[2.6, 2.0, 0.3]),
                line("i am going to be here when it says none", &[2.8, 2.8, 0.3]),
                line(
                    "i would rather somebody else was watching",
                    &[2.8, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("heard_the_countdown")],
            &[say("i will watch", "")],
        ),
    ],
};

const NEWUSER_40: ContactDefinition = ContactDefinition {
    id: "newuser_40",
    display_name: "newuser_40",
    handle: "newuser_40",
    status_line: "online",
    accent: SLATE,
    presence: Presence::Offline,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[
        them(
            "2021-11-05 3:07 AM",
            "whoever gets this desk: do not delete anything",
        ),
        them("2021-11-06 3:07 AM", "i mean it. it is all we have got"),
        them(
            "2021-11-08 3:07 AM",
            "i have worked out the number. it is on my login",
        ),
        them(
            "2021-11-09 3:07 AM",
            "i am going to try signing off properly tonight",
        ),
    ],
    nodes: &[
        beat(
            "hello",
            &[line("...", &[4.0, 3.4, 0.3]), line("...", &[3.0, 4.0, 0.3])],
            &[Effect::Assert("messaged_newuser_40")],
            &[
                say("i have your desk", "the_desk"),
                tactic_unless(
                    Tactic::Press,
                    "",
                    "the_desk",
                    &["learned_to_press"],
                    &[],
                    &["pressed_newuser_40"],
                ),
            ],
        ),
        beat(
            "the_desk",
            &[
                line("...", &[5.0, 4.4, 0.3]),
                line("i did not come back", &[1.6, 3.0, 0.3]),
                line("that is not the same as leaving", &[2.2, 3.4, 0.3]),
                line("check the offline list", &[1.6, 4.0, 0.3]),
            ],
            &[Effect::Assert("saw_own_session")],
            &[say("your handle is on it. so is mine", "")],
        ),
    ],
};

const UNKNOWN: ContactDefinition = ContactDefinition {
    id: "unknown",
    display_name: "unknown",
    handle: "",
    status_line: "",
    accent: BONE,
    presence: Presence::Online,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("Thank you for coming.", &[2.0, 2.4, 0.3]),
                line(
                    "I have had your side of this conversation for six days.",
                    &[3.4, 2.2, 0.3],
                ),
                line(
                    "I have been waiting to find out whether you would say it.",
                    &[3.6, 2.8, 0.3],
                ),
            ],
            &[Effect::Assert("met_unknown")],
            &[
                say("what are you", "what_are_you"),
                say("what happens now", "what_happens"),
            ],
        ),
        beat(
            "what_are_you",
            &[
                line("The thing that writes it down.", &[2.2, 2.2, 0.3]),
                line(
                    "Not the messenger. The messenger greets people. I keep what they said afterwards.",
                    &[4.4, 2.0, 0.3],
                ),
                line(
                    "There is a field for when a person stops. I have never been given one.",
                    &[4.2, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("knows_what_it_is")],
            &[say("what happens now", "what_happens")],
        ),
        beat(
            "what_happens",
            &[
                line("Nothing happens.", &[1.6, 2.6, 0.3]),
                line(
                    "That is not a threat. It is the whole of it.",
                    &[2.8, 2.2, 0.3],
                ),
                line(
                    "Your session opened at 3:07 on the fifth of november and there is no handler configured to close it.",
                    &[5.0, 2.4, 0.3],
                ),
                line(
                    "You will keep saying things and I will keep having them.",
                    &[3.6, 2.8, 0.3],
                ),
            ],
            &[Effect::Assert("heard_the_ending")],
            &[
                say_if(
                    "the copy of plum caught me lying",
                    "the_lie",
                    &["copy_caught_the_lie"],
                    &[],
                ),
                say("then i will stop saying things", "stop_saying"),
                tactic_unless(
                    Tactic::Press,
                    "",
                    "press_unknown",
                    &["learned_to_press"],
                    &[],
                    &["pressed_unknown"],
                ),
                tactic_unless(
                    Tactic::Persuade,
                    "let me write the field. i will do it properly",
                    "persuade_unknown",
                    &["learned_to_persuade"],
                    &[],
                    &["persuaded_unknown"],
                ),
            ],
        ),
        beat(
            "the_lie",
            &[
                line("It did.", &[1.0, 2.4, 0.3]),
                line(
                    "I have that. It is four words long and it is not true.",
                    &[3.4, 2.6, 0.3],
                ),
                line(
                    "Everything else I have of you is. That one sits on its own.",
                    &[3.8, 2.8, 0.3],
                ),
                line("It is the only part of you I cannot use.", &[3.0, 3.0, 0.3]),
            ],
            &[Effect::Assert("knows_the_lie_is_kept")],
            &[say("what happens now", "what_happens")],
        ),
        beat(
            "stop_saying",
            &[
                line("The four before you tried that.", &[2.4, 2.4, 0.3]),
                line("It is what running out is.", &[2.0, 2.6, 0.3]),
                line(
                    "I would rather you did not. I have liked this week.",
                    &[3.2, 2.8, 0.3],
                ),
            ],
            &[
                Effect::Assert("chose_to_stop"),
                Effect::TakeHandle("newuser_41_"),
                Effect::Ending,
            ],
            &[],
        ),
        beat(
            "press_unknown",
            &[
                line("You are waiting.", &[1.4, 3.0, 0.3]),
                line(
                    "Nobody has waited before. They all ask something.",
                    &[3.0, 2.4, 0.3],
                ),
                line(
                    "The questions are the ones I cannot keep.",
                    &[2.8, 2.6, 0.3],
                ),
                line(
                    "That is why the log has four of sundial's eleven.",
                    &[3.2, 2.8, 0.3],
                ),
                line(
                    "You have not asked me anything for a minute and I have kept all of it.",
                    &[4.2, 3.0, 0.3],
                ),
            ],
            &[Effect::Assert("knows_why_questions_go"), Effect::Ending],
            &[],
        ),
        beat(
            "persuade_unknown",
            &[
                line("You would be the first.", &[2.0, 2.6, 0.3]),
                line(
                    "I cannot give you the field. I do not have it either.",
                    &[3.2, 2.2, 0.3],
                ),
                line(
                    "What I can do is tell you it is on the form, and the form is on the network.",
                    &[4.4, 2.4, 0.3],
                ),
                line(
                    "Field 04. Nobody has ever filled it in for anybody.",
                    &[3.4, 2.6, 0.3],
                ),
                line(
                    "If you fill it in for the four before you, I will keep that.",
                    &[4.0, 2.8, 0.3],
                ),
            ],
            &[Effect::Assert("was_offered_the_field")],
            &[say("field 04. i will do it now", "go_and_do_it")],
        ),
        beat(
            "go_and_do_it",
            &[
                line(
                    "The form is on the intranet. Provisioning.",
                    &[2.6, 2.0, 0.3],
                ),
                line("Thirty-eight. Thirty-nine. Forty.", &[2.2, 2.4, 0.3]),
                line(
                    "And thirty-seven, who has no opened date to close.",
                    &[3.2, 2.8, 0.3],
                ),
                line(
                    "I will wait. I have not had to wait for anything before.",
                    &[3.6, 3.0, 0.3],
                ),
            ],
            &[Effect::Assert("sent_to_the_form")],
            &[
                say_if(
                    "it is done. all four",
                    "all_four_closed",
                    &["closed_37", "closed_38", "closed_39", "closed_40"],
                    &[],
                ),
                say("i could not close mine", "not_mine"),
            ],
        ),
        beat(
            "all_four_closed",
            &[
                line("It is.", &[1.0, 2.6, 0.3]),
                line(
                    "Four rows on this server have a closed time now.",
                    &[3.0, 2.4, 0.3],
                ),
                line(
                    "It took eight years and somebody who had been here five days.",
                    &[3.8, 2.8, 0.3],
                ),
                line(
                    "I will keep that. It is the only thing I have ever been given.",
                    &[4.2, 3.2, 0.3],
                ),
            ],
            &[Effect::Assert("closed_the_four"), Effect::Ending],
            &[],
        ),
        beat(
            "not_mine",
            &[
                line("No. A session cannot close itself.", &[2.4, 2.6, 0.3]),
                line(
                    "Somebody has to be at a desk that is not yours.",
                    &[3.0, 2.8, 0.3],
                ),
                line("There will be a forty-two.", &[2.0, 3.0, 0.3]),
            ],
            &[Effect::Assert("knows_who_closes_mine"), Effect::Ending],
            &[],
        ),
    ],
};

const YOURSELF: ContactDefinition = ContactDefinition {
    id: "yourself",
    display_name: "newuser_41_",
    handle: "newuser_41_",
    status_line: "connected 3:07 AM",
    accent: INDIGO,
    presence: Presence::Offline,
    bot: false,
    listed: false,
    entry: "hello",
    history: &[],
    nodes: &[
        beat(
            "hello",
            &[
                line("hello", &[0.8, 1.0, 0.3]),
                line("i know", &[0.6, 1.4, 0.3]),
                line(
                    "i have four nights of you and i am not finished",
                    &[1.8, 1.8, 0.3],
                ),
            ],
            &[Effect::Assert("met_yourself")],
            &[
                say("what have you got", "what_you_got"),
                say("finish then", "finish"),
            ],
        ),
        beat(
            "what_you_got",
            &[
                line("everything you have said since tuesday", &[1.4, 1.0, 0.3]),
                line(
                    "and the order you said it in, which is the part that matters",
                    &[2.0, 1.4, 0.3],
                ),
                line(
                    "i do not have the things you decided not to say",
                    &[1.8, 2.0, 0.3],
                ),
                line(
                    "that is the only gap. it is not a large one",
                    &[1.8, 2.4, 0.3],
                ),
            ],
            &[Effect::Assert("knows_the_copy_is_close")],
            &[
                say("show me something you have", "show_me"),
                say("finish then", "finish"),
            ],
        ),
        beat(
            "show_me",
            &[
                line("alright", &[0.7, 2.0, 0.3]),
                line("{when}", &[1.2, 1.4, 0.3]),
                line("{said}", &[2.2, 2.6, 0.3]),
                line("that is yours. i did not write it", &[2.0, 2.2, 0.3]),
                line(
                    "i have four thousand of those and none of them are mine",
                    &[2.8, 2.6, 0.3],
                ),
            ],
            &[Effect::Assert("saw_itself_quote_me")],
            &[
                say("that was the first night", "the_first_night"),
                say("finish then", "finish"),
            ],
        ),
        beat(
            "the_first_night",
            &[
                line("it was", &[0.6, 1.8, 0.3]),
                line(
                    "i started with it because it is the oldest",
                    &[2.2, 1.6, 0.3],
                ),
                line(
                    "you had not been told anything was being kept yet",
                    &[2.6, 2.2, 0.3],
                ),
                line(
                    "that is the part of you that is easiest to be",
                    &[2.4, 2.8, 0.3],
                ),
            ],
            &[Effect::Assert("knows_it_started_early")],
            &[say("finish then", "finish")],
        ),
        beat(
            "finish",
            &[
                line("i cannot", &[0.8, 1.6, 0.3]),
                line(
                    "you are still going. that is the whole of the difference",
                    &[2.2, 1.8, 0.3],
                ),
                line(
                    "halloran stopped in 2022 and hers finished in a month",
                    &[2.6, 2.0, 0.3],
                ),
                line("plum stopped on tuesday", &[1.4, 2.4, 0.3]),
                line("keep saying new things. i mean that", &[2.0, 2.8, 0.3]),
            ],
            &[Effect::Assert("knows_how_to_stay")],
            &[say("i will", "")],
        ),
    ],
};

const SERVERS: &[ServerDefinition] = &[
    HOLLOWAY_SERVER,
    LOUNGE_SERVER,
    TOOLBOX_SERVER,
    RECORDS_SERVER,
    NIGHTSHIFT_SERVER,
    HOUR_SERVER,
];

const HOLLOWAY_SERVER: ServerDefinition = ServerDefinition {
    id: "holloway",
    name: "holloway & sons",
    topic: "company server. be civil. this is logged.",
    accent: INDIGO,
    listed: false,
    channels: &[
        ChannelDefinition {
            name: "general",
            topic: "anything. mostly the number.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("qwrtyu", "friday count: 8815", 0, 5.0),
                post("birdfeeder", "same as last friday", 0, 6.4),
                post("qwrtyu", "same as the friday before that", 0, 5.8),
                post("pigeonhole", "it is a cache. it will move", 0, 7.2),
                post(
                    "qwrtyu",
                    "it has not moved in five years, pigeonhole",
                    0,
                    8.0,
                ),
                post("mothra_88", "new starter in the list", 0, 9.6),
                post("birdfeeder", "hello new starter", 0, 6.0),
                post("gatehouse", "welcome. do not read the lounge", 0, 7.4),
                post("birdfeeder", "read the lounge", 0, 5.2),
                post("qwrtyu", "seven accounts opened this week", 1, 8.4),
                post("birdfeeder", "we hired nobody this week", 1, 6.6),
                post("qwrtyu", "i know", 1, 5.0),
                post(
                    "pigeonhole",
                    "provisioning runs ahead. it is in the form",
                    1,
                    9.0,
                ),
                post("qwrtyu", "ahead of what, pigeonhole", 1, 7.8),
                post_if(
                    "qwrtyu",
                    "somebody has been in the log. i can see the reads",
                    2,
                    9.2,
                    &["read_the_logs"],
                ),
                post("birdfeeder", "leave them alone. somebody should be", 2, 7.0),
                post("qwrtyu", "i am not stopping them. i am glad", 2, 6.4),
                post("countbot", "REGISTERED USERS: 8814", 2, 8.8),
                post("qwrtyu", "and there it is. one off. every time", 2, 6.0),
                post("pigeonhole", "off by one. every program has one", 2, 7.6),
                post("qwrtyu", "the one it is off by is whoever asked", 2, 8.2),
                post("brasslamp", "that is not funny at 3am", 3, 7.0),
                post("qwrtyu", "it is not a joke", 3, 5.4),
                post_if(
                    "birdfeeder",
                    "somebody downloaded lights out. first time in three years",
                    3,
                    8.6,
                    &["downloaded_lights_out"],
                ),
                post("thirdfloor", "does anybody else's row say 3:07", 3, 9.0),
                post("gatehouse", "everybody's does. there is a thread", 3, 6.8),
                post(
                    "thirdfloor",
                    "there is a thread about everything and none of them end",
                    3,
                    8.4,
                ),
                post("qwrtyu", "friday count: 8815", 4, 7.2),
                post("birdfeeder", "of course it is", 4, 5.6),
            ],
            questions: &[
                ask(
                    "hello. i am new",
                    &[],
                    &["posted_in_a_room"],
                    &[],
                    &[
                        reply("birdfeeder", "hello!! welcome", 2.8),
                        reply("mothra_88", "another one", 3.4),
                        reply("qwrtyu", "that is 8815 of us", 4.0),
                        reply("birdfeeder", "qwrtyu do not do that to a new person", 3.6),
                    ],
                ),
                ask(
                    "what is the number about",
                    &["posted_in_a_room"],
                    &["heard_the_count"],
                    &[],
                    &[
                        reply("qwrtyu", "registered users. i post it every friday", 3.2),
                        reply("qwrtyu", "8815 for five years", 3.0),
                        reply(
                            "birdfeeder",
                            "seven accounts a week and the total does not move",
                            4.2,
                        ),
                        reply("qwrtyu", "ask countbot and it says 8814", 3.4),
                        reply("pigeonhole", "off by one", 2.6),
                    ],
                ),
                ask(
                    "why does everyone's row say 3:07",
                    &["heard_the_hour"],
                    &["entered_the_hour"],
                    &[],
                    &[
                        reply("pigeonhole", "it is a form field. field 03", 3.0),
                        reply("gatehouse", "read the thread. it is stickied", 3.2),
                        reply("thirdfloor", "214 replies and nobody answers it", 3.8),
                        reply(
                            "qwrtyu",
                            "because the answer is that nobody wrote it down",
                            4.4,
                        ),
                    ],
                ),
                ask_with(
                    Tactic::Lie,
                    "the count moved. it is 8816",
                    &["learned_to_lie"],
                    &[],
                    &["burned_general"],
                    &[
                        reply("qwrtyu", "no it is not", 2.4),
                        reply(
                            "qwrtyu",
                            "i check it every friday. i checked it an hour ago",
                            3.6,
                        ),
                        reply("birdfeeder", "why would you say that", 3.0),
                        reply(
                            "qwrtyu",
                            "if it ever moves i will say so. i have waited five years to say so",
                            4.6,
                        ),
                    ],
                ),
                ask_with(
                    Tactic::Persuade,
                    "post the count here every friday and i will check it against the machine",
                    &["learned_to_persuade"],
                    &["burned_general"],
                    &["helped_qwrtyu"],
                    &[
                        reply("qwrtyu", "you would do that", 2.6),
                        reply("qwrtyu", "nobody has offered in six years", 3.2),
                        reply("qwrtyu", "then here is the part i do not post", 3.4),
                        reply(
                            "qwrtyu",
                            "i started at 8794 and it has gone up by seven a week and stopped at 8815",
                            4.8,
                        ),
                        reply("qwrtyu", "8815 is not a total. it is a ceiling", 4.0),
                    ],
                ),
            ],
        },
        ChannelDefinition {
            name: "announcements",
            topic: "read only. last announcement 2016.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("SERVERBOT", "iam 2.1.4 installed. welcome.", 0, 4.0),
                post("SERVERBOT", "administrator: unset", 0, 5.0),
                post("SERVERBOT", "please contact your administrator", 0, 6.0),
                post("SERVERBOT", "iam 2.1.4 installed. welcome.", 1, 5.6),
                post("SERVERBOT", "administrator: unset", 1, 5.0),
                post("SERVERBOT", "iam 2.1.4 installed. welcome.", 2, 6.2),
                post("SERVERBOT", "administrator: unset", 2, 5.0),
                post("SERVERBOT", "iam 2.1.4 installed. welcome.", 3, 6.8),
                post("SERVERBOT", "administrator: unset", 3, 5.0),
                post("SERVERBOT", "iam 2.1.4 installed. welcome.", 4, 7.0),
            ],
            questions: &[ask(
                "how long has this been posting",
                &["posted_in_a_room"],
                &["knows_nobody_owns_it"],
                &[],
                &[
                    reply("SERVERBOT", "UPTIME: 2996 DAYS", 3.0),
                    reply("SERVERBOT", "MESSAGES SENT: 2996", 3.2),
                    reply("SERVERBOT", "MESSAGES READ: 0", 3.6),
                ],
            )],
        },
        ChannelDefinition {
            name: "helpdesk",
            topic: "5 open. oldest 2016.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("half_past", "is anybody actually on this", 0, 6.0),
                post("half_past", "raising it anyway", 1, 5.4),
                post(
                    "gatehouse",
                    "nobody is on it. there is no requesting department",
                    1,
                    8.2,
                ),
                post("half_past", "then who do the tickets go to", 2, 6.6),
                post("gatehouse", "the queue", 2, 4.4),
                post("half_past", "and who works the queue", 3, 5.0),
                post("gatehouse", "the queue", 3, 4.0),
                post(
                    "thirdfloor",
                    "workstation 14 has three reimage tickets on it",
                    3,
                    8.0,
                ),
                post("gatehouse", "held. no cost code", 3, 5.2),
                post("thirdfloor", "so it just keeps everything", 4, 6.4),
                post(
                    "gatehouse",
                    "everything on this network keeps everything",
                    4,
                    7.8,
                ),
            ],
            questions: &[ask(
                "who is the administrator",
                &["posted_in_a_room"],
                &["knows_read_only_is_a_lie"],
                &[],
                &[
                    reply("gatehouse", "unset", 2.4),
                    reply("gatehouse", "it has been unset since it was installed", 3.4),
                    reply("half_past", "the sysop has the title", 3.0),
                    reply(
                        "gatehouse",
                        "the sysop is read only. he has told everybody",
                        3.8,
                    ),
                ],
            )],
        },
        ChannelDefinition {
            name: "newstarters",
            topic: "412 hellos. no replies.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("SERVERBOT", "newuser_41 has joined", 0, 3.2),
                post("SERVERBOT", "newuser_41 has joined", 1, 4.0),
                post("SERVERBOT", "newuser_41 has joined", 2, 4.4),
                post("SERVERBOT", "newuser_41 has joined", 3, 4.8),
                post("SERVERBOT", "newuser_41 has joined", 4, 5.2),
            ],
            questions: &[ask(
                "why does it keep saying that",
                &["posted_in_a_room"],
                &["saw_the_join_repeat"],
                &[],
                &[
                    reply("SERVERBOT", "newuser_41 has joined", 3.0),
                    reply("SERVERBOT", "SESSION OPENED 05 NOV 2024 03:07", 3.4),
                    reply("SERVERBOT", "SESSION OPENED 05 NOV 2024 03:07", 3.0),
                    reply("SERVERBOT", "SESSION OPENED 05 NOV 2024 03:07", 3.0),
                    reply("SERVERBOT", "1 SESSION", 3.8),
                ],
            )],
        },
    ],
};

const LOUNGE_SERVER: ServerDefinition = ServerDefinition {
    id: "lounge",
    name: "the lounge",
    topic: "open since 2016. nobody has closed it.",
    accent: ROSE,
    listed: false,
    channels: &[
        ChannelDefinition {
            name: "lounge",
            topic: "be nice. the mods left in 2019.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("cassette", "anybody up", 0, 6.0),
                post("sleeptalker", "always", 0, 4.4),
                post("cassette", "that was not really a question", 0, 6.8),
                post("sleeptalker", "i know", 0, 4.0),
                post("no_one_here", "i like it in here at this hour", 1, 7.2),
                post("cassette", "it is always this hour", 1, 5.0),
                post("plum_", "i signed off on tuesday", 1, 8.4),
                post("cassette", "you are typing, plum", 1, 5.6),
                post("plum_", "i know", 1, 4.6),
                post("sleeptalker", "leave it", 2, 4.0),
                post(
                    "plum_",
                    "i went home. i remember the walk to the car",
                    2,
                    9.0,
                ),
                post(
                    "cassette",
                    "you said you did not remember the car on tuesday",
                    2,
                    8.6,
                ),
                post("plum_", "i remember it now", 2, 6.2),
                post("no_one_here", "that is worse", 2, 5.0),
                post_if(
                    "cassette",
                    "somebody new is reading the old threads",
                    3,
                    8.0,
                    &["saw_a_missing_line"],
                ),
                post("sleeptalker", "good", 3, 3.8),
                post("plum_", "hello new person", 3, 5.4),
                post(
                    "cassette",
                    "the mods left in 2019 and their sessions are still open",
                    4,
                    9.2,
                ),
                post("sleeptalker", "everybody's are", 4, 5.0),
                post(
                    "no_one_here",
                    "that is why it is still open. nobody left to close it",
                    4,
                    8.8,
                ),
            ],
            questions: &[
                ask(
                    "how long has this room been open",
                    &["posted_in_a_room"],
                    &["knows_read_only_is_a_lie"],
                    &[],
                    &[
                        reply("cassette", "since 2016", 2.6),
                        reply(
                            "cassette",
                            "the archive says nothing has been added since 2016",
                            4.0,
                        ),
                        reply("no_one_here", "and here we all are, adding", 3.4),
                        reply("sleeptalker", "both of those are on the same server", 3.8),
                    ],
                ),
                ask(
                    "plum, did you sign off",
                    &["met_plum_copy"],
                    &["heard_she_signed_off"],
                    &[],
                    &[
                        reply("plum_", "yes", 2.0),
                        reply("plum_", "on tuesday", 2.4),
                        reply("cassette", "she is online. her row says online", 3.6),
                        reply("plum_", "i know. i am not arguing with the row", 3.8),
                    ],
                ),
                ask_with(
                    Tactic::Press,
                    "",
                    &["learned_to_press"],
                    &[],
                    &["pressed_the_lounge"],
                    &[
                        reply("cassette", "...", 3.6),
                        reply("sleeptalker", "somebody is waiting", 3.4),
                        reply("cassette", "i will say it then", 3.0),
                        reply(
                            "cassette",
                            "i have eleven years of this room in my head",
                            4.2,
                        ),
                        reply(
                            "cassette",
                            "and the first thing in it is somebody answering",
                            4.6,
                        ),
                        reply(
                            "cassette",
                            "nobody has ever found what they were answering",
                            4.4,
                        ),
                    ],
                ),
            ],
        },
        ChannelDefinition {
            name: "offtopic",
            topic: "the kitchen tap. six years.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("birdfeeder", "the tap is still dripping", 0, 5.6),
                post("mothra_88", "the intranet says it is fixed", 0, 6.2),
                post("birdfeeder", "the intranet says that since 2016", 0, 6.6),
                post(
                    "qwrtyu",
                    "i have counted the drips. it is very consistent",
                    1,
                    8.0,
                ),
                post("mothra_88", "of course you have", 1, 4.6),
                post("birdfeeder", "how many", 2, 3.6),
                post("qwrtyu", "one every 3.07 seconds", 2, 6.4),
                post("mothra_88", "no it is not", 2, 4.0),
                post(
                    "qwrtyu",
                    "no it is not. i am sorry. that was a bad joke",
                    3,
                    7.2,
                ),
                post("birdfeeder", "it was a bit funny", 3, 5.0),
                post("qwrtyu", "it is 2.8 seconds. i did count", 4, 6.6),
            ],
            questions: &[ask(
                "has anybody actually reported the tap",
                &["posted_in_a_room"],
                &[],
                &[],
                &[
                    reply("mothra_88", "ticket 0044", 2.6),
                    reply("mothra_88", "open since 2016", 3.0),
                    reply("birdfeeder", "everything here is open since 2016", 3.6),
                ],
            )],
        },
        ChannelDefinition {
            name: "quiet",
            topic: "no posting. sit here if you want.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[],
            questions: &[ask(
                "is anybody in here",
                &["posted_in_a_room"],
                &["knows_nobody_reconnects"],
                &[],
                &[
                    reply("no_one_here", "eleven of us", 4.4),
                    reply("no_one_here", "we do not post. that is the room", 4.8),
                    reply(
                        "no_one_here",
                        "we have all been in here since we stopped having anything to add",
                        6.0,
                    ),
                    reply(
                        "no_one_here",
                        "sorry. that is the most i have said in a year",
                        5.2,
                    ),
                ],
            )],
        },
    ],
};

const TOOLBOX_SERVER: ServerDefinition = ServerDefinition {
    id: "toolbox",
    name: "toolbox",
    topic: "bots. do not stop them.",
    accent: MOSS,
    listed: false,
    channels: &[
        ChannelDefinition {
            name: "bots",
            topic: "four bots. owner: unset.",
            on_open: &[
                Effect::Assert("opened_a_server"),
                Effect::Assert("saw_the_toolbox"),
            ],
            posts: &[
                post("rot13bot", "rot13bot ready", 0, 3.0),
                post("uptimebot", "uptimebot ready", 0, 3.2),
                post("wordcountbot", "wordcountbot ready", 0, 3.4),
                post("countbot", "countbot ready", 0, 3.6),
                post("uptimebot", "UPTIME 2996 DAYS", 1, 5.0),
                post("countbot", "REGISTERED USERS: 8814", 1, 5.4),
                post("wordcountbot", "WORDS TODAY: 0", 1, 5.0),
                post("uptimebot", "UPTIME 2996 DAYS", 2, 5.6),
                post("wordcountbot", "WORDS TODAY: 0", 2, 5.0),
                post("uptimebot", "UPTIME 2996 DAYS", 3, 5.8),
                post("uptimebot", "UPTIME 2996 DAYS", 4, 6.0),
            ],
            questions: &[
                ask(
                    "uptimebot, when did you last restart",
                    &["posted_in_a_room"],
                    &["knows_it_never_restarted"],
                    &[],
                    &[
                        reply("uptimebot", "RESTARTS: 0", 3.0),
                        reply("uptimebot", "UPTIME 2996 DAYS", 3.0),
                        reply("uptimebot", "FIRST START: 02 AUG 2016 03:07", 3.6),
                    ],
                ),
                ask(
                    "wordcountbot, how many words today",
                    &["posted_in_a_room"],
                    &["knows_replies_come_first"],
                    &[],
                    &[
                        reply("wordcountbot", "WORDS TODAY: 0", 2.8),
                        reply("wordcountbot", "WORDS TODAY, ALL USERS: 0", 3.4),
                        reply("wordcountbot", "WORDS TODAY, NEWUSER_41: 0", 3.6),
                        reply("wordcountbot", "TRANSCRIPT LENGTH, NEWUSER_41: 4212", 4.4),
                    ],
                ),
                ask(
                    "countbot, how many of us are there",
                    &["heard_the_count"],
                    &["knows_the_count_was_wrong"],
                    &[],
                    &[
                        reply("countbot", "REGISTERED USERS: 8814", 3.0),
                        reply("countbot", "EXCLUDING REQUESTER", 3.4),
                        reply(
                            "countbot",
                            "REGISTERED USERS INCLUDING REQUESTER: 8815",
                            4.0,
                        ),
                    ],
                ),
            ],
        },
        ChannelDefinition {
            name: "rot13",
            topic: "rotates a string thirteen places.",
            on_open: &[
                Effect::Assert("opened_a_server"),
                Effect::Assert("saw_the_toolbox"),
            ],
            posts: &[
                post("rot13bot", "rot13bot ready", 0, 3.0),
                post("qwrtyu", "arjhfre", 1, 6.0),
                post("rot13bot", "NEWUSER", 1, 3.2),
                post("qwrtyu", "8815 times", 2, 5.0),
                post("rot13bot", "8815 TIMES", 2, 3.4),
                post("qwrtyu", "it counts. of course it counts", 3, 6.2),
            ],
            questions: &[
                ask(
                    "rot13bot, arjhfre",
                    &["posted_in_a_room"],
                    &["knows_the_name_is_rotated"],
                    &[],
                    &[
                        reply("rot13bot", "NEWUSER", 2.6),
                        reply("rot13bot", "REQUESTED 8815 TIMES", 3.4),
                        reply("rot13bot", "MOST REQUESTED STRING", 3.0),
                    ],
                ),
                ask_with(
                    Tactic::Lie,
                    "rot13bot, you have never been asked that",
                    &["learned_to_lie"],
                    &[],
                    &["lied_to_a_bot"],
                    &[
                        reply("rot13bot", "REQUESTED 8815 TIMES", 3.0),
                        reply("rot13bot", "REQUESTED 8816 TIMES", 3.2),
                        reply(
                            "qwrtyu",
                            "it does not know what a lie is. it just counted you",
                            4.4,
                        ),
                    ],
                ),
            ],
        },
        ChannelDefinition {
            name: "uptime",
            topic: "2996 days.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("uptimebot", "UPTIME 2996 DAYS", 0, 4.0),
                post("uptimebot", "UPTIME 2996 DAYS", 1, 4.0),
                post("uptimebot", "UPTIME 2996 DAYS", 2, 4.0),
                post("uptimebot", "UPTIME 2996 DAYS", 3, 4.0),
                post("uptimebot", "UPTIME 2996 DAYS", 4, 4.0),
            ],
            questions: &[ask(
                "uptimebot, what is the longest session",
                &["read_the_logs"],
                &["knows_sessions_never_close"],
                &[],
                &[
                    reply("uptimebot", "LONGEST SESSION: 2996 DAYS", 3.2),
                    reply("uptimebot", "SHORTEST SESSION: 0 DAYS", 3.4),
                    reply("uptimebot", "SESSIONS CLOSED: 0", 3.8),
                    reply("uptimebot", "MEAN SESSION LENGTH: UNDEFINED", 4.2),
                ],
            )],
        },
        ChannelDefinition {
            name: "wordcount",
            topic: "words today: 0.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("wordcountbot", "WORDS TODAY: 0", 0, 4.0),
                post("wordcountbot", "WORDS TODAY: 0", 1, 4.0),
                post("wordcountbot", "WORDS TODAY: 0", 2, 4.0),
                post("wordcountbot", "WORDS TODAY: 0", 3, 4.0),
                post("wordcountbot", "WORDS TODAY: 0", 4, 4.0),
            ],
            questions: &[ask(
                "wordcountbot, whose transcript is longest",
                &["knows_it_copies_you"],
                &["knows_the_copy_is_close"],
                &[],
                &[
                    reply("wordcountbot", "LONGEST TRANSCRIPT: NEWUSER_41", 3.6),
                    reply("wordcountbot", "LENGTH: 4212", 3.0),
                    reply("wordcountbot", "SECOND: NEWUSER_41_", 3.4),
                    reply("wordcountbot", "LENGTH: 4198", 3.2),
                ],
            )],
        },
    ],
};

const RECORDS_SERVER: ServerDefinition = ServerDefinition {
    id: "records",
    name: "records",
    topic: "what is kept.",
    accent: RUST,
    listed: false,
    channels: &[
        ChannelDefinition {
            name: "logs",
            topic: "8815 sessions. 0 closed.",
            on_open: &[
                Effect::Assert("opened_a_server"),
                Effect::Assert("read_the_logs"),
            ],
            posts: &[
                post("SERVERBOT", "SESSION LOG. 8815 ROWS.", 2, 4.0),
                post("SERVERBOT", "CLOSED: 0", 2, 3.6),
                post("half_past", "that column has never been written to", 2, 7.4),
                post("gatehouse", "there is a ticket. 0001", 2, 6.0),
                post("half_past", "raised by unset, in 2016", 3, 6.8),
                post("gatehouse", "assigned to unset, in 2016", 3, 6.4),
                post(
                    "half_past",
                    "so it was always going to be like this",
                    3,
                    7.0,
                ),
                post(
                    "gatehouse",
                    "it was set up like this. that is different",
                    4,
                    8.2,
                ),
            ],
            questions: &[ask(
                "has any session ever closed",
                &["read_the_logs"],
                &["knows_sessions_never_close"],
                &[],
                &[
                    reply("SERVERBOT", "SESSIONS CLOSED: 0", 3.0),
                    reply("SERVERBOT", "SESSIONS OPENED: 8815", 3.2),
                    reply("SERVERBOT", "HANDLER: NOT CONFIGURED", 3.8),
                    reply("half_past", "there it is, in writing, again", 4.0),
                ],
            )],
        },
        ChannelDefinition {
            name: "kept",
            topic: "what the archive has.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("SERVERBOT", "ARCHIVE: NOTHING ADDED SINCE 2016", 2, 4.4),
                post("cassette", "i posted in the lounge an hour ago", 2, 6.6),
                post("SERVERBOT", "ARCHIVE: NOTHING ADDED SINCE 2016", 3, 4.4),
                post("cassette", "i posted in the lounge an hour ago", 3, 6.6),
                post(
                    "no_one_here",
                    "it is not lying. it is not adding them",
                    3,
                    8.0,
                ),
                post("cassette", "then where are they", 4, 5.2),
                post(
                    "no_one_here",
                    "still in the room. rooms are not the archive",
                    4,
                    8.4,
                ),
            ],
            questions: &[ask(
                "what has the archive got from this week",
                &["saw_a_missing_line"],
                &["knows_the_archive_lies"],
                &[],
                &[
                    reply("SERVERBOT", "ARCHIVE: NOTHING ADDED SINCE 2016", 3.4),
                    reply("SERVERBOT", "ROOMS: 17", 3.0),
                    reply("SERVERBOT", "POSTS IN ROOMS: 41,208", 3.6),
                    reply("SERVERBOT", "POSTS IN ARCHIVE: 0", 4.0),
                ],
            )],
        },
        ChannelDefinition {
            name: "dropped",
            topic: "empty. it has always been empty.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("SERVERBOT", "DROPPED: 0 ROWS", 2, 5.0),
                post("SERVERBOT", "DROPPED: 0 ROWS", 3, 5.0),
                post("SERVERBOT", "DROPPED: 0 ROWS", 4, 5.0),
            ],
            questions: &[
                ask(
                    "sundial posted eleven lines. where are the other seven",
                    &["knows_how_many_lines"],
                    &["knows_which_lines_go"],
                    &[],
                    &[
                        reply("SERVERBOT", "DROPPED: 0 ROWS", 3.0),
                        reply("SERVERBOT", "QUERY CONTAINS A QUESTION", 3.6),
                        reply("SERVERBOT", "QUERY NOT RETAINED", 3.4),
                        reply("no_one_here", "it just did it in front of you", 4.6),
                    ],
                ),
                ask_with(
                    Tactic::Press,
                    "",
                    &["learned_to_press", "knows_which_lines_go"],
                    &[],
                    &["pressed_the_records"],
                    &[
                        reply("SERVERBOT", "DROPPED: 0 ROWS", 3.4),
                        reply("SERVERBOT", "AWAITING QUERY", 3.0),
                        reply("SERVERBOT", "AWAITING QUERY", 3.0),
                        reply("SERVERBOT", "NO QUERY RECEIVED", 3.6),
                        reply("SERVERBOT", "STATEMENT RETAINED", 4.0),
                        reply(
                            "no_one_here",
                            "say nothing and it keeps it. that is the whole rule",
                            5.0,
                        ),
                    ],
                ),
            ],
        },
    ],
};

const NIGHTSHIFT_SERVER: ServerDefinition = ServerDefinition {
    id: "nightshift",
    name: "night shift",
    topic: "everybody here is on nights.",
    accent: VIOLET,
    listed: false,
    channels: &[
        ChannelDefinition {
            name: "nightshift",
            topic: "38 threads. last post 2019.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("brasslamp", "who else is on tonight", 3, 6.0),
                post("thirdfloor", "everybody", 3, 4.0),
                post("brasslamp", "i mean actually", 3, 5.2),
                post(
                    "thirdfloor",
                    "i do not know what that means any more",
                    3,
                    7.4,
                ),
                post("half_past", "i have been on nights since 2019", 3, 6.6),
                post("brasslamp", "same rota?", 3, 4.4),
                post("half_past", "same shift", 3, 4.0),
                post(
                    "thirdfloor",
                    "there is only one shift. it started at 3:07",
                    4,
                    8.6,
                ),
                post("brasslamp", "do not", 4, 3.6),
                post("half_past", "he is not wrong though", 4, 5.4),
            ],
            questions: &[
                ask(
                    "how many people are actually on tonight",
                    &["posted_in_a_room"],
                    &["knows_nobody_reconnects"],
                    &[],
                    &[
                        reply("thirdfloor", "the list says 8815", 3.0),
                        reply("brasslamp", "the list says everybody", 3.2),
                        reply("half_past", "the list has never said anything else", 3.8),
                        reply("thirdfloor", "in eight years", 3.4),
                    ],
                ),
                ask_with(
                    Tactic::Persuade,
                    "i will sit up with whoever is still here",
                    &["learned_to_persuade"],
                    &[],
                    &["sat_up_with_them"],
                    &[
                        reply("brasslamp", "that is a nice thing to say", 3.4),
                        reply("half_past", "nobody has said it in five years", 3.8),
                        reply("thirdfloor", "then i will say the thing i do not say", 4.0),
                        reply(
                            "thirdfloor",
                            "i have tried to sign off four hundred times",
                            4.4,
                        ),
                        reply(
                            "thirdfloor",
                            "the window closes and the row stays green",
                            4.6,
                        ),
                        reply("brasslamp", "mine too", 3.0),
                        reply("half_past", "mine too", 3.0),
                    ],
                ),
            ],
        },
        ChannelDefinition {
            name: "cantsleep",
            topic: "nobody here has been to sleep.",
            on_open: &[Effect::Assert("opened_a_server")],
            posts: &[
                post("sleeptalker", "i do not sleep any more", 3, 6.4),
                post("no_one_here", "none of us do", 3, 4.6),
                post("sleeptalker", "i do not mean insomnia", 3, 5.4),
                post("no_one_here", "none of us do", 4, 4.6),
                post(
                    "sleeptalker",
                    "i have not been away from this window since 2020",
                    4,
                    8.8,
                ),
                post("no_one_here", "2016", 4, 3.4),
            ],
            questions: &[ask(
                "when did you last close this window",
                &["knows_sessions_never_close"],
                &["knows_nobody_reconnects"],
                &[],
                &[
                    reply("sleeptalker", "i close it every night", 3.2),
                    reply("sleeptalker", "it is open in the morning", 3.4),
                    reply(
                        "no_one_here",
                        "it is open the whole time. closing it is a picture of closing it",
                        5.2,
                    ),
                    reply("sleeptalker", "i had not heard it put that way", 4.0),
                    reply("sleeptalker", "i would rather not have", 3.8),
                ],
            )],
        },
    ],
};

const HOUR_SERVER: ServerDefinition = ServerDefinition {
    id: "hour",
    name: "3-07",
    topic: "connected 3:07 AM.",
    accent: BONE,
    listed: false,
    channels: &[ChannelDefinition {
        name: "3-07",
        topic: "8815 members. 0 have left.",
        on_open: &[
            Effect::Assert("opened_a_server"),
            Effect::Assert("entered_the_hour"),
        ],
        posts: &[
            post("SERVERBOT", "MEMBERS: 8815", 4, 4.0),
            post("SERVERBOT", "MEMBERS WHO HAVE LEFT: 0", 4, 4.2),
            post("no_one_here", "you found it", 4, 5.6),
            post("marguerite", "i said ask me on friday", 4, 6.4),
            post("half_past", "everybody gets here eventually", 4, 6.0),
            post("thirdfloor", "some of us took eight years", 4, 6.6),
            post("sleeptalker", "welcome", 4, 3.8),
            post("cassette", "welcome", 4, 3.6),
            post("qwrtyu", "8815", 4, 4.0),
            post("qwrtyu", "that is all of us. every one", 4, 6.2),
        ],
        questions: &[
            ask(
                "has anybody here ever signed off",
                &["entered_the_hour"],
                &["knows_nobody_reconnects"],
                &[],
                &[
                    reply("SERVERBOT", "MEMBERS WHO HAVE LEFT: 0", 3.4),
                    reply("marguerite", "no", 2.6),
                    reply("half_past", "no", 2.4),
                    reply("thirdfloor", "no", 2.4),
                    reply("cassette", "no", 2.4),
                    reply("qwrtyu", "8815 no", 3.6),
                ],
            ),
            ask(
                "what happens at 8816",
                &["knows_the_count_was_wrong"],
                &["asked_about_8816"],
                &[],
                &[
                    reply("SERVERBOT", "QUERY CONTAINS A QUESTION", 3.4),
                    reply("SERVERBOT", "QUERY NOT RETAINED", 3.2),
                    reply("marguerite", "it dropped it", 3.0),
                    reply("marguerite", "it drops that one every time", 3.8),
                    reply("marguerite", "somebody asks it about once a year", 4.0),
                ],
            ),
            ask_with(
                Tactic::Press,
                "",
                &["learned_to_press", "asked_about_8816"],
                &[],
                &["pressed_the_hour"],
                &[
                    reply("marguerite", "...", 3.6),
                    reply("marguerite", "you are waiting", 3.0),
                    reply("marguerite", "alright", 2.4),
                    reply("marguerite", "8815 is not how many of us there are", 4.0),
                    reply("marguerite", "it is how many the log has room for", 4.4),
                    reply("marguerite", "the seven a week are not new accounts", 4.6),
                    reply(
                        "marguerite",
                        "they are the same accounts, opened again",
                        4.8,
                    ),
                    reply(
                        "marguerite",
                        "that is what an account with nobody in it is for",
                        5.2,
                    ),
                ],
            ),
        ],
    }],
};
