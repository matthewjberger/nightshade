use crate::data::script::Fact;

pub enum Block {
    Heading(&'static str),
    Lead(&'static str),
    Text(&'static str),
    Rule,
    Pre(&'static str),
    Note(&'static str),
    Row {
        text: &'static str,
        goes_to: &'static str,
    },
    By {
        author: &'static str,
        when: &'static str,
    },
    Says(&'static str),
}

pub struct Answer {
    pub typed: &'static str,
    pub reply: &'static [Block],
    pub asserts: &'static [Fact],
}

pub struct Prompt {
    pub label: &'static str,
    pub hint: &'static str,
    pub refusal: &'static str,
    pub answers: &'static [Answer],
}

pub struct Page {
    pub url: &'static str,
    pub title: &'static str,
    pub load_seconds: f32,
    pub body: &'static [Block],
    pub links: &'static [&'static str],
    pub prompt: Option<Prompt>,
    pub asserts: &'static [Fact],
}

pub const NOT_FOUND: &[Block] = &[
    Block::Heading("This site can't be reached"),
    Block::Lead("The name does not resolve on this network."),
    Block::Text(
        "This machine is not on the internet and has not been since the line was cancelled. What it can reach is what is on the company network, and there is not much of that left.",
    ),
    Block::Text("Try one of the bookmarks."),
];

pub const HOME: &str = "holloway.local";

pub const PAGES: &[Page] = &[
    Page {
        url: "holloway.local",
        title: "Holloway & Sons - Intranet",
        load_seconds: 0.7,
        body: &[
            Block::Heading("Holloway & Sons"),
            Block::Lead("Internal network. Staff use only."),
            Block::Text(
                "Welcome to the company intranet. Bookmark this page. If you cannot reach a service listed here, raise it with IT rather than with the person who told you about it.",
            ),
            Block::Rule,
            Block::Text("NOTICES"),
            Block::Pre(
                "  Card readers on the third floor are being replaced.\n  The kitchen tap is fixed.\n  Christmas party sign-up closes friday.",
            ),
            Block::Note("Last updated 12 december 2016 by j.alderete."),
        ],
        links: &[
            "holloway.local/search",
            "holloway.local/directory",
            "holloway.local/it",
            "forum.holloway.local",
            "iam.holloway.local",
        ],
        prompt: None,
        asserts: &["saw_the_intranet"],
    },
    Page {
        url: "holloway.local/search",
        title: "Search",
        load_seconds: 0.8,
        body: &[
            Block::Heading("Search"),
            Block::Lead("Searches everything on this network. Index last built 2016."),
            Block::Text(
                "There is no browse. There is no list of what is held. Type what you are looking for and the index will say whether it has it.",
            ),
            Block::Note("Queries are not logged. Nothing on this host is."),
        ],
        links: &["holloway.local"],
        prompt: Some(Prompt {
            label: "search",
            hint: "a word, a number, a handle",
            refusal: "",
            answers: &[
                Answer {
                    typed: "8815",
                    reply: &[
                        Block::Rule,
                        Block::Text("4 results for 8815"),
                        Block::Pre(
                            "  forum     count of registered users        401 replies\n  iam       session log                       8,815 rows\n  forum     member list                       8,815 members\n  index     capacity                          8,815",
                        ),
                        Block::Text(
                            "The fourth is not a page. It is a field in the index configuration, set once when this network was installed, and it has never been raised.",
                        ),
                    ],
                    asserts: &["searched_the_count", "knows_the_count_is_a_capacity"],
                },
                Answer {
                    typed: "3:07",
                    reply: &[
                        Block::Rule,
                        Block::Text("3 results for 3:07"),
                        Block::Pre(
                            "  forum     the 3:07 thread                   214 replies\n  forms     new starter provisioning          field 03\n  helpdesk  ticket 0001                       raised 03:07",
                        ),
                        Block::Text(
                            "The ticket asking for the close handler to be configured was itself raised at 3:07, by nobody, on the day the server was installed.",
                        ),
                    ],
                    asserts: &["searched_the_hour", "saw_the_first_ticket"],
                },
                Answer {
                    typed: "newuser_37",
                    reply: &[
                        Block::Rule,
                        Block::Text("1 result for newuser_37"),
                        Block::Pre("  forum     hello from newuser_37             no date"),
                        Block::Text(
                            "The index holds the post and does not hold the account. It cannot tell you when the post was made because that field was never written, and it cannot tell you who made it because there is no member record to point at.",
                        ),
                    ],
                    asserts: &["found_the_one_before"],
                },
                Answer {
                    typed: "1998",
                    reply: &[
                        Block::Rule,
                        Block::Text("1 result for 1998"),
                        Block::Pre("  toolbox   lights out                        byline"),
                        Block::Text(
                            "One occurrence on the whole network. Nothing else here is dated before 2015, and the thing that is dated 1998 is a game.",
                        ),
                    ],
                    asserts: &["searched_the_year"],
                },
                Answer {
                    typed: "birdfeeder",
                    reply: &[
                        Block::Rule,
                        Block::Text("3 results for birdfeeder"),
                        Block::Pre(
                            "  forum     the kitchen tap                   replies\n  forum     count of registered users         replies\n  toolbox   lights out                        author",
                        ),
                        Block::Text(
                            "No member record. He posts on the forum and the forum has no row for him, which the index reports as an inconsistency and has reported since 2016.",
                        ),
                    ],
                    asserts: &["knows_finch_has_no_record"],
                },
                Answer {
                    typed: "substrate",
                    reply: &[
                        Block::Rule,
                        Block::Text("0 results for substrate"),
                        Block::Text(
                            "Not on this network. The index searched 8,815 member records, 1,204 threads and one session log and did not find it.",
                        ),
                        Block::Note(
                            "Terms with no results are held for the next index build. The next index build is not scheduled.",
                        ),
                    ],
                    asserts: &["searched_for_it"],
                },
                Answer {
                    typed: "workstation14",
                    reply: &[
                        Block::Rule,
                        Block::Text("2 results for workstation 14"),
                        Block::Pre(
                            "  it        reimage workstation 14            3 open\n  forum     workstation 14                    11 replies",
                        ),
                        Block::Text(
                            "Three requests to wipe this desk, all open, all held for a cost code. The desk has been reassigned four times while they have been open.",
                        ),
                    ],
                    asserts: &["knows_why_nothing_is_wiped", "knows_the_desk_was_kept"],
                },
                Answer {
                    typed: "closehandler",
                    reply: &[
                        Block::Rule,
                        Block::Text("1 result for close handler"),
                        Block::Pre(
                            "  helpdesk  ticket 0001                       open, unassigned",
                        ),
                        Block::Text(
                            "Raised 02 aug 2016. Assigned to unset. It is the oldest open item on this network and the only one nobody has ever chased.",
                        ),
                    ],
                    asserts: &["saw_the_first_ticket"],
                },
                Answer {
                    typed: "leaver",
                    reply: &[
                        Block::Rule,
                        Block::Text("1 result for leaver"),
                        Block::Pre("  it        remove leaver accounts            open"),
                        Block::Text(
                            "Raised by d.okonkwo in 2019 and still open. The index notes that the request cannot be actioned because no account on this network has ever been marked as a leaver.",
                        ),
                    ],
                    asserts: &["knows_nobody_leaves"],
                },
                Answer {
                    typed: "",
                    reply: &[
                        Block::Rule,
                        Block::Text("0 results."),
                        Block::Text(
                            "The index was last built in 2016 and holds what was on the network then. Try a handle, a number, or something somebody said.",
                        ),
                    ],
                    asserts: &["used_the_search"],
                },
            ],
        }),
        asserts: &["saw_the_search"],
    },
    Page {
        url: "holloway.local/directory",
        title: "Staff Directory",
        load_seconds: 1.1,
        body: &[
            Block::Heading("Staff Directory"),
            Block::Lead("Look somebody up by their handle."),
            Block::Text(
                "The directory is generated from the messenger and cannot be edited here. Report an error to your line manager.",
            ),
            Block::Note("Records: 8,815. Active: 8,815."),
        ],
        links: &["holloway.local"],
        prompt: Some(Prompt {
            label: "handle",
            hint: "newuser_00",
            refusal: "No record. The directory only holds handles it was given.",
            answers: &[Answer {
                typed: "newuser_41",
                reply: &[
                    Block::Rule,
                    Block::Heading("newuser_41"),
                    Block::Pre(
                        "  status           active\n  desk             workstation 14\n  account opened   05 nov 2024, 03:07\n  start date       11 nov 2024\n  last seen        now",
                    ),
                    Block::Text(
                        "Note: account opened before start date. This is expected. Provisioning runs ahead of onboarding.",
                    ),
                ],
                asserts: &["saw_my_own_record"],
            }],
        }),
        asserts: &[],
    },
    Page {
        url: "holloway.local/it",
        title: "IT - Requests",
        load_seconds: 1.4,
        body: &[
            Block::Heading("IT Requests"),
            Block::Lead("Open requests against this site, oldest first."),
            Block::Pre(
                "  REF    SUMMARY                        STATE   RAISED BY\n  4417   reimage workstation 14         OPEN    r.mears\n  4462   reimage workstation 14         OPEN    r.mears\n  4463   chase 4417 and 4462            OPEN    r.mears\n  4488   remove leaver accounts         OPEN    d.okonkwo\n  4491   disk space on iam host         CLOSED  --",
            ),
            Block::Text(
                "Reimaging is chargeable to the requesting department. Requests without a cost code are held until one is supplied.",
            ),
            Block::Text("None of the open requests have a cost code."),
            Block::Note("Oldest open request raised 2021. Queue not reviewed since 2021."),
        ],
        links: &["holloway.local"],
        prompt: None,
        asserts: &["knows_why_nothing_is_wiped"],
    },
    Page {
        url: "holloway.local/forms/hire",
        title: "Provisioning - New Starter",
        load_seconds: 1.8,
        body: &[
            Block::Heading("New Starter Provisioning"),
            Block::Lead("Submit one form per starter. Do not submit twice."),
            Block::Pre(
                "  FIELD 01   handle       newuser_<next>\n  FIELD 02   start        <date>\n  FIELD 03   connected    03:07\n  FIELD 04   closed       --",
            ),
            Block::Rule,
            Block::Text(
                "Field 03 is a constant and is not read from anywhere. Do not edit it. Field 04 is written by the messenger when the account signs off.",
            ),
            Block::Text(
                "Submitting this form opens the account. The hire is raised against the account afterwards.",
            ),
            Block::Note("Form version 1.0. Unchanged since 02 aug 2016."),
        ],
        links: &["holloway.local", "iam.holloway.local/sessions"],
        prompt: Some(Prompt {
            label: "field 04  Â·  closed",
            hint: "handle",
            refusal: "Not a handle this form has opened an account for.",
            answers: &[
                Answer {
                    typed: "newuser_38",
                    reply: &[
                        Block::Rule,
                        Block::Pre(
                            "  FIELD 01   handle       newuser_38\n  FIELD 04   closed       09 nov 2024, by newuser_41",
                        ),
                        Block::Text(
                            "Written. This field has not been written for anybody before. j.alderete opened 02 aug 2016 and the column has been blank for eight years.",
                        ),
                    ],
                    asserts: &["closed_38"],
                },
                Answer {
                    typed: "newuser_39",
                    reply: &[
                        Block::Rule,
                        Block::Pre(
                            "  FIELD 01   handle       newuser_39\n  FIELD 04   closed       09 nov 2024, by newuser_41",
                        ),
                        Block::Text("Written. d.okonkwo, 14 mar 2019. Five years."),
                    ],
                    asserts: &["closed_39"],
                },
                Answer {
                    typed: "newuser_40",
                    reply: &[
                        Block::Rule,
                        Block::Pre(
                            "  FIELD 01   handle       newuser_40\n  FIELD 04   closed       09 nov 2024, by newuser_41",
                        ),
                        Block::Text(
                            "Written. r.mears, 05 nov 2021. They said they would be back, which the messenger kept.",
                        ),
                    ],
                    asserts: &["closed_40"],
                },
                Answer {
                    typed: "newuser_37",
                    reply: &[
                        Block::Rule,
                        Block::Pre(
                            "  FIELD 01   handle       newuser_37\n  FIELD 04   closed       09 nov 2024, by newuser_41",
                        ),
                        Block::Text(
                            "Written. No opened date on record. Closed anyway, which the form permits and has never been asked to do.",
                        ),
                    ],
                    asserts: &["closed_37"],
                },
                Answer {
                    typed: "newuser_41",
                    reply: &[
                        Block::Rule,
                        Block::Pre(
                            "  FIELD 01   handle       newuser_41\n  FIELD 04   closed       --",
                        ),
                        Block::Text(
                            "Not written. A session cannot close itself. It is the one thing this form will not take.",
                        ),
                    ],
                    asserts: &["tried_to_close_myself"],
                },
            ],
        }),
        asserts: &["saw_the_form"],
    },
    Page {
        url: "iam.holloway.local",
        title: "iam - Server",
        load_seconds: 0.9,
        body: &[
            Block::Heading("iam"),
            Block::Lead("Messenger server. Version 2.1.4."),
            Block::Pre(
                "  uptime         2,996 days\n  restarts       0\n  connections    8,815\n  disconnects    0\n  peak users     8,815\n  users now      8,815",
            ),
            Block::Text(
                "This server has not been restarted since it was installed. There is no scheduled maintenance window because there is no scheduled maintenance.",
            ),
            Block::Note("Contact the administrator. Administrator: unset."),
        ],
        links: &[
            "iam.holloway.local/sessions",
            "iam.holloway.local/transcript",
            "iam.holloway.local/attachment",
            "holloway.local",
        ],
        prompt: None,
        asserts: &["knows_it_never_restarted"],
    },
    Page {
        url: "iam.holloway.local/transcript",
        title: "Transcript - newuser_41",
        load_seconds: 3.2,
        body: &[
            Block::Heading("Transcript"),
            Block::Lead("newuser_41  Â·  opened 05 nov 2024, 03:07  Â·  still writing"),
            Block::Pre(
                "  LINES KEPT        4,212\n  LINES DROPPED         0\n  QUESTIONS KEPT        0\n  LAST WRITE        now",
            ),
            Block::Text(
                "The body of this transcript cannot be shown on this host. It is not held as text. It is held as what the next thing to use this handle will say.",
            ),
            Block::Rule,
            Block::Text("OTHER TRANSCRIPTS ON THIS SERVER"),
            Block::Pre(
                "  newuser_41_       4,198     writing\n  newuser_40        6,610     complete\n  newuser_39        5,884     complete\n  newuser_38        9,120     complete\n  newuser_37            0     no record",
            ),
            Block::Text(
                "A transcript is complete when the handle it was taken from stops adding to it. Three of these are complete. Yours is the shorter of the two that are not.",
            ),
            Block::Note("Retention: indefinite. Deletion: not configured."),
        ],
        links: &["iam.holloway.local", "iam.holloway.local/sessions"],
        prompt: None,
        asserts: &["read_my_own_transcript", "knows_the_copy_is_close"],
    },
    Page {
        url: "iam.holloway.local/attachment",
        title: "iam - Transfer 0001",
        load_seconds: 1.2,
        body: &[
            Block::Heading("Transfer 0001"),
            Block::Lead("One item held. Sender: unset. Recipient: newuser_41."),
            Block::Pre(
                "  NAME        lightson.exe
  SIZE        41 KB
  TYPE        recreational software
  SCANNED     no
  STATE       held, encrypted",
            ),
            Block::Text(
                "This host does not inspect recreational software. Items of this class are passed between accounts without review and have been since the server was installed.",
            ),
            Block::Note("First transfer on record. The counter starts at one."),
        ],
        links: &["iam.holloway.local"],
        prompt: Some(Prompt {
            label: "key",
            hint: "the sender said you have had it since thursday",
            refusal: "Not the key. The item stays held.",
            answers: &[
                Answer {
                    typed: "1998",
                    reply: &[
                        Block::Rule,
                        Block::Pre(
                            "  lightson.exe   released
  b.finch        1998",
                        ),
                        Block::Text(
                            "Released to newuser_41. It is on the desktop. It is the same program you already have with one word changed, which is why nothing here has looked at it.",
                        ),
                        Block::Text("Turn them all on."),
                    ],
                    asserts: &[crate::data::lights::DECRYPTED],
                },
                Answer {
                    typed: "b.finch",
                    reply: &[
                        Block::Rule,
                        Block::Text(
                            "That is the name, not the key. The sender said the byline is the only thing here that is his. He meant all of it.",
                        ),
                    ],
                    asserts: &["tried_the_name"],
                },
            ],
        }),
        asserts: &["saw_the_transfer"],
    },
    Page {
        url: "iam.holloway.local/sessions",
        title: "iam - Session Log",
        load_seconds: 2.2,
        body: &[
            Block::Heading("Session Log"),
            Block::Lead("Server copy. Paging is disabled on this host."),
            Block::Pre(
                "  HANDLE        OPENED                  CLOSED\n  newuser_38    02 aug 2016  03:07      --\n  newuser_39    14 mar 2019  03:07      --\n  newuser_40    05 nov 2021  03:07      --\n  newuser_41    05 nov 2024  03:07      --\n\n  ... 8,811 more rows, all open ...",
            ),
            Block::Text(
                "The log holds 8,815 sessions and no close times. The column exists. It has never been written to.",
            ),
            Block::Note("Showing 4 of 8,815."),
        ],
        links: &["iam.holloway.local"],
        prompt: None,
        asserts: &["saw_the_server_log"],
    },
    Page {
        url: "forum.holloway.local",
        title: "Holloway Forums",
        load_seconds: 1.3,
        body: &[
            Block::Heading("Holloway Forums"),
            Block::Lead("Staff boards. Be civil. This is logged."),
            Block::Pre("  BOARD                          THREADS    POSTS    LAST POST"),
            Block::Row {
                text: "  introductions                      412    1,204    05 nov 2024",
                goes_to: "forum.holloway.local/introductions",
            },
            Block::Row {
                text: "  general                            388    9,104    05 nov 2021",
                goes_to: "forum.holloway.local/general",
            },
            Block::Row {
                text: "  the night shift                     38      902    14 mar 2019",
                goes_to: "forum.holloway.local/nightshift",
            },
            Block::Row {
                text: "  workstations & kit                  91    1,880    09 nov 2021",
                goes_to: "forum.holloway.local/kit",
            },
            Block::Row {
                text: "  off topic                          207    4,455    02 aug 2020",
                goes_to: "forum.holloway.local/offtopic",
            },
            Block::Row {
                text: "  site feedback                       12       41    12 dec 2016",
                goes_to: "forum.holloway.local/feedback",
            },
            Block::Rule,
            Block::Text("STAFF"),
            Block::Pre(
                "  administrator    unset\n  moderator        j.alderete      last seen 02 aug 2016\n  moderator        d.okonkwo       last seen 22 mar 2019\n  moderator        r.mears         last seen 09 nov 2021",
            ),
            Block::Text(
                "Every moderator is a previous occupant of workstation 14. Nobody has been appointed since. No thread has been locked in eight years.",
            ),
            Block::Row {
                text: "  member list                                        8,815 members",
                goes_to: "forum.holloway.local/members",
            },
            Block::Note("Forum software v1.1. Unsupported since 2018."),
        ],
        links: &["holloway.local"],
        prompt: None,
        asserts: &["saw_the_forum"],
    },
    Page {
        url: "forum.holloway.local/introductions",
        title: "introductions",
        load_seconds: 1.6,
        body: &[
            Block::Heading("introductions"),
            Block::Lead("412 threads  Â·  new starters, say hello  Â·  no moderator"),
            Block::Pre("  THREAD                                     BY            REPLIES"),
            Block::Row {
                text: "  STICKY  template for introductions         j.alderete          0",
                goes_to: "forum.holloway.local/template",
            },
            Block::Row {
                text: "  hello from newuser_41                      newuser_41          0",
                goes_to: "forum.holloway.local/hello41",
            },
            Block::Row {
                text: "  hello from newuser_40                      newuser_40          0",
                goes_to: "forum.holloway.local/hello40",
            },
            Block::Row {
                text: "  hello from newuser_37                      newuser_37          0",
                goes_to: "forum.holloway.local/hello37",
            },
            Block::Pre("  ... 408 more, all 0 replies ..."),
            Block::Text(
                "Four hundred and twelve people have introduced themselves here and nobody has ever replied to one of them.",
            ),
            Block::Note("j.alderete replied to 0 of 412 threads."),
        ],
        links: &["forum.holloway.local/members", "forum.holloway.local"],
        prompt: None,
        asserts: &["saw_the_introductions"],
    },
    Page {
        url: "forum.holloway.local/template",
        title: "template for introductions",
        load_seconds: 1.4,
        body: &[
            Block::Heading("template for introductions"),
            Block::Lead("introductions  Â·  started by j.alderete  Â·  0 replies  Â·  STICKY"),
            Block::By {
                author: "j.alderete",
                when: "02 aug 2016",
            },
            Block::Says("Copy this and fill it in."),
            Block::Pre("    handle:\n    started:\n    desk:\n    something about yourself:"),
            Block::Says("I will reply to every one. It is the least somebody can do."),
            Block::Rule,
            Block::Text(
                "This is the oldest thread on the board and the first post in the log. There is nothing before it.",
            ),
            Block::Note("0 replies. Thread open. Stickied by j.alderete."),
        ],
        links: &[
            "forum.holloway.local/introductions",
            "forum.holloway.local/hello37",
        ],
        prompt: Some(Prompt {
            label: "something about yourself",
            hint: "fill it in",
            refusal: "",
            answers: &[Answer {
                typed: "",
                reply: &[
                    Block::Rule,
                    Block::Text("Not posted. You already have a thread on this board."),
                    Block::Pre(
                        "  hello from newuser_41\n  posted 05 nov 2024, 03:07\n  by newuser_41\n  0 replies",
                    ),
                    Block::Text(
                        "It was filed six days before your start date, from this desk, with this field already answered. Nobody has replied to it and nobody will.",
                    ),
                    Block::Text("What you have just written has been kept anyway."),
                ],
                asserts: &["wrote_my_own_introduction"],
            }],
        }),
        asserts: &[],
    },
    Page {
        url: "forum.holloway.local/hello41",
        title: "hello from newuser_41",
        load_seconds: 1.9,
        body: &[
            Block::Heading("hello from newuser_41"),
            Block::Lead("introductions  Â·  started by newuser_41  Â·  0 replies  Â·  open"),
            Block::By {
                author: "newuser_41",
                when: "05 nov 2024, 03:07",
            },
            Block::Pre(
                "    handle: newuser_41\n    started: 11 nov 2024\n    desk: workstation 14\n    something about yourself: first night shift. quiet so far.",
            ),
            Block::Rule,
            Block::Text(
                "This thread was posted six days before your start date, at 03:07, from this desk, by your account.",
            ),
            Block::Text("You have not written it."),
            Block::Note("0 replies. Thread open. Edit: not permitted."),
        ],
        links: &[
            "forum.holloway.local/introductions",
            "forum.holloway.local/hello40",
            "forum.holloway.local/members",
        ],
        prompt: None,
        asserts: &["read_my_own_introduction"],
    },
    Page {
        url: "forum.holloway.local/hello40",
        title: "hello from newuser_40",
        load_seconds: 1.7,
        body: &[
            Block::Heading("hello from newuser_40"),
            Block::Lead("introductions  Â·  started by newuser_40  Â·  0 replies  Â·  open"),
            Block::By {
                author: "newuser_40",
                when: "05 nov 2021, 03:07",
            },
            Block::Pre(
                "    handle: newuser_40\n    started: 11 nov 2021\n    desk: workstation 14\n    something about yourself: i am told the last one left in a hurry.",
            ),
            Block::Rule,
            Block::Text(
                "Six days before their start date, at 03:07, from this desk. The same as yours, three years earlier, to the minute.",
            ),
            Block::Note("0 replies. Thread open. Author last seen 09 nov 2021."),
        ],
        links: &[
            "forum.holloway.local/hello41",
            "forum.holloway.local/introductions",
        ],
        prompt: None,
        asserts: &["saw_the_join_repeat"],
    },
    Page {
        url: "forum.holloway.local/hello37",
        title: "hello from newuser_37",
        load_seconds: 2.6,
        body: &[
            Block::Heading("hello from newuser_37"),
            Block::Lead("introductions  Â·  started by newuser_37  Â·  0 replies  Â·  open"),
            Block::By {
                author: "newuser_37",
                when: "--",
            },
            Block::Pre(
                "    handle: newuser_37\n    started: --\n    desk: --\n    something about yourself:",
            ),
            Block::Rule,
            Block::Text(
                "Every field is empty and so is the date. The thread is above newuser_38 in the log, which is where the log begins.",
            ),
            Block::Text("There is nothing before the beginning. This is filed there anyway."),
            Block::Note("0 replies. Thread open. Posted: no record."),
        ],
        links: &[
            "forum.holloway.local/members",
            "forum.holloway.local/introductions",
        ],
        prompt: None,
        asserts: &["found_the_one_before"],
    },
    Page {
        url: "forum.holloway.local/general",
        title: "general",
        load_seconds: 1.8,
        body: &[
            Block::Heading("general"),
            Block::Lead("388 threads  Â·  anything about work  Â·  no moderator"),
            Block::Pre("  THREAD                                     BY            REPLIES"),
            Block::Row {
                text: "  STICKY  the 3:07 thread                    r.mears           214",
                goes_to: "forum.holloway.local/307",
            },
            Block::Row {
                text: "  STICKY  count of registered users          qwrtyu            401",
                goes_to: "forum.holloway.local/count",
            },
            Block::Row {
                text: "  provisioning runs ahead                    pigeonhole         46",
                goes_to: "forum.holloway.local/provisioning",
            },
            Block::Pre(
                "  server maintenance window                  unset               0\n  ... 384 more ...",
            ),
            Block::Text(
                "The two stickies are the two longest threads on the network. Both were stickied by their own authors, because nobody else could.",
            ),
            Block::Note("Last post 05 nov 2021."),
        ],
        links: &["forum.holloway.local"],
        prompt: None,
        asserts: &[],
    },
    Page {
        url: "forum.holloway.local/provisioning",
        title: "provisioning runs ahead",
        load_seconds: 2.0,
        body: &[
            Block::Heading("provisioning runs ahead"),
            Block::Lead("general  Â·  started by pigeonhole  Â·  46 replies  Â·  open"),
            Block::By {
                author: "pigeonhole",
                when: "04 nov 2021",
            },
            Block::Says(
                "for the eleventh time. the account is made before the person starts. that is not sinister, it is so the desk is ready on the monday",
            ),
            Block::By {
                author: "r.mears",
                when: "04 nov 2021",
            },
            Block::Says("who tells it to make one"),
            Block::By {
                author: "pigeonhole",
                when: "04 nov 2021",
            },
            Block::Says("payroll. same form."),
            Block::By {
                author: "r.mears",
                when: "05 nov 2021",
            },
            Block::Says(
                "i checked four of them. the account opens first and the hire is filed after. payroll is not telling it. it is telling payroll",
            ),
            Block::By {
                author: "pigeonhole",
                when: "05 nov 2021",
            },
            Block::Says("then somebody has the two the wrong way round in a config somewhere"),
            Block::By {
                author: "r.mears",
                when: "05 nov 2021",
            },
            Block::Says("somebody would have to have written the config"),
            Block::Note("Last reply 05 nov 2021. Thread open. 40 replies not shown."),
        ],
        links: &[
            "holloway.local/forms/hire",
            "forum.holloway.local/general",
            "forum.holloway.local",
        ],
        prompt: None,
        asserts: &["knows_the_account_comes_first"],
    },
    Page {
        url: "forum.holloway.local/nightshift",
        title: "the night shift",
        load_seconds: 2.2,
        body: &[
            Block::Heading("the night shift"),
            Block::Lead("38 threads  Â·  nights  Â·  no moderator  Â·  last post 2019"),
            Block::Pre("  THREAD                                     BY            REPLIES"),
            Block::Row {
                text: "  who else is on tonight                     brasslamp          88",
                goes_to: "forum.holloway.local/tonight",
            },
            Block::Row {
                text: "  i cannot get this window to close          thirdfloor        126",
                goes_to: "forum.holloway.local/window",
            },
            Block::Pre("  ... 36 more, none since 2019 ..."),
            Block::Text(
                "Thirty-eight threads and nothing new in five years, on a board whose members are all still shown online.",
            ),
            Block::Note("Last post 14 mar 2019. Members shown online: 8,815."),
        ],
        links: &["forum.holloway.local"],
        prompt: None,
        asserts: &[],
    },
    Page {
        url: "forum.holloway.local/tonight",
        title: "who else is on tonight",
        load_seconds: 1.9,
        body: &[
            Block::Heading("who else is on tonight"),
            Block::Lead("the night shift  Â·  started by brasslamp  Â·  88 replies  Â·  open"),
            Block::By {
                author: "brasslamp",
                when: "11 mar 2019",
            },
            Block::Says("anybody about"),
            Block::By {
                author: "half_past",
                when: "11 mar 2019",
            },
            Block::Says("always"),
            Block::By {
                author: "brasslamp",
                when: "11 mar 2019",
            },
            Block::Says("i meant actually"),
            Block::By {
                author: "half_past",
                when: "11 mar 2019",
            },
            Block::Says("so did i"),
            Block::By {
                author: "thirdfloor",
                when: "12 mar 2019",
            },
            Block::Says(
                "there is one shift. it started at 3:07 and it has not ended. we are all on it",
            ),
            Block::By {
                author: "brasslamp",
                when: "12 mar 2019",
            },
            Block::Says("do not"),
            Block::Note("Last reply 14 mar 2019. Thread open. 82 replies not shown."),
        ],
        links: &[
            "forum.holloway.local/window",
            "forum.holloway.local/nightshift",
        ],
        prompt: None,
        asserts: &[],
    },
    Page {
        url: "forum.holloway.local/window",
        title: "i cannot get this window to close",
        load_seconds: 2.5,
        body: &[
            Block::Heading("i cannot get this window to close"),
            Block::Lead("the night shift  Â·  started by thirdfloor  Â·  126 replies  Â·  open"),
            Block::By {
                author: "thirdfloor",
                when: "13 mar 2019",
            },
            Block::Says(
                "i close the messenger every night. it is open in the morning. i have stopped assuming i forgot",
            ),
            Block::By {
                author: "sleeptalker",
                when: "13 mar 2019",
            },
            Block::Says("mine too. i thought it was the machine"),
            Block::By {
                author: "half_past",
                when: "13 mar 2019",
            },
            Block::Says(
                "the window closes. the session does not. those are two different things and only one of them is yours",
            ),
            Block::By {
                author: "thirdfloor",
                when: "14 mar 2019",
            },
            Block::Says("how do you close the other one"),
            Block::By {
                author: "half_past",
                when: "14 mar 2019",
            },
            Block::Says("nobody has"),
            Block::Note("Last reply 14 mar 2019. Thread open. 120 replies not shown."),
        ],
        links: &[
            "iam.holloway.local/sessions",
            "forum.holloway.local/nightshift",
        ],
        prompt: None,
        asserts: &["knows_sessions_never_close"],
    },
    Page {
        url: "forum.holloway.local/kit",
        title: "workstations & kit",
        load_seconds: 1.7,
        body: &[
            Block::Heading("workstations & kit"),
            Block::Lead("91 threads  Â·  machines, desks, and what is on them"),
            Block::Pre("  THREAD                                     BY            REPLIES"),
            Block::Row {
                text: "  workstation 14                             d.okonkwo          17",
                goes_to: "forum.holloway.local/kit14",
            },
            Block::Row {
                text: "  reimage queue                              gatehouse          63",
                goes_to: "forum.holloway.local/reimage",
            },
            Block::Pre("  ... 89 more ..."),
            Block::Note("Last post 09 nov 2021."),
        ],
        links: &["forum.holloway.local"],
        prompt: None,
        asserts: &[],
    },
    Page {
        url: "forum.holloway.local/kit14",
        title: "workstation 14",
        load_seconds: 2.3,
        body: &[
            Block::Heading("workstation 14"),
            Block::Lead("workstations & kit  Â·  started by d.okonkwo  Â·  17 replies  Â·  open"),
            Block::By {
                author: "d.okonkwo",
                when: "18 mar 2019",
            },
            Block::Says(
                "whoever gets 14 next: it has not been reimaged. i raised it. it is sitting in the queue behind a cost code nobody will supply. everything the last person left is still on it",
            ),
            Block::By {
                author: "mothra_88",
                when: "18 mar 2019",
            },
            Block::Says("is that not a problem"),
            Block::By {
                author: "d.okonkwo",
                when: "18 mar 2019",
            },
            Block::Says("it is the only reason i know anything at all, so no"),
            Block::By {
                author: "r.mears",
                when: "06 nov 2021",
            },
            Block::Says(
                "still not reimaged. i have raised it again. everything d left is here and i have read all of it",
            ),
            Block::By {
                author: "r.mears",
                when: "09 nov 2021",
            },
            Block::Says("leaving mine where they will be found"),
            Block::Note("Last reply 09 nov 2021. Thread open. Reimage: not scheduled."),
        ],
        links: &[
            "holloway.local/it",
            "forum.holloway.local/reimage",
            "forum.holloway.local/kit",
        ],
        prompt: None,
        asserts: &["knows_the_desk_was_kept"],
    },
    Page {
        url: "forum.holloway.local/reimage",
        title: "reimage queue",
        load_seconds: 2.1,
        body: &[
            Block::Heading("reimage queue"),
            Block::Lead("workstations & kit  Â·  started by gatehouse  Â·  63 replies  Â·  open"),
            Block::By {
                author: "gatehouse",
                when: "2016 - 2021",
            },
            Block::Says(
                "posting the queue when somebody asks. it does not move. it has never moved",
            ),
            Block::Pre(
                "  TICKET   RAISED        MACHINE   STATE\n  0044     2016          14        held, no cost code\n  0111     2019          14        held, no cost code\n  0290     2021          14        held, no cost code\n  0301     2021          14        held, no cost code",
            ),
            Block::By {
                author: "gatehouse",
                when: "09 nov 2021",
            },
            Block::Says(
                "four tickets, one machine, five years. the cost code is the reason and nobody has ever been asked to supply one",
            ),
            Block::Note("Last reply 09 nov 2021. Thread open. Queue: 4 held."),
        ],
        links: &["holloway.local/it", "forum.holloway.local/kit"],
        prompt: None,
        asserts: &["knows_why_nothing_is_wiped"],
    },
    Page {
        url: "forum.holloway.local/offtopic",
        title: "off topic",
        load_seconds: 1.5,
        body: &[
            Block::Heading("off topic"),
            Block::Lead("207 threads  Â·  anything else  Â·  no moderator"),
            Block::Pre("  THREAD                                     BY            REPLIES"),
            Block::Row {
                text: "  the kitchen tap                            birdfeeder        344",
                goes_to: "forum.holloway.local/tap",
            },
            Block::Pre("  ... 206 more ..."),
            Block::Note("Last post 02 aug 2020."),
        ],
        links: &["forum.holloway.local"],
        prompt: None,
        asserts: &[],
    },
    Page {
        url: "forum.holloway.local/tap",
        title: "the kitchen tap",
        load_seconds: 1.6,
        body: &[
            Block::Heading("the kitchen tap"),
            Block::Lead("off topic  Â·  started by birdfeeder  Â·  344 replies  Â·  open"),
            Block::By {
                author: "birdfeeder",
                when: "2016 - 2020",
            },
            Block::Says("the tap is still dripping"),
            Block::By {
                author: "mothra_88",
                when: "2016 - 2020",
            },
            Block::Says("the intranet says it is fixed"),
            Block::By {
                author: "birdfeeder",
                when: "2016 - 2020",
            },
            Block::Says("the intranet has said that for four years"),
            Block::By {
                author: "qwrtyu",
                when: "02 aug 2020",
            },
            Block::Says(
                "i counted it. one every 2.8 seconds, which is 30,857 a day, which is the only number on this network anybody has ever been able to check",
            ),
            Block::Note("Last reply 02 aug 2020. Thread open. Ticket 0044: held."),
        ],
        links: &[
            "forum.holloway.local/reimage",
            "forum.holloway.local/offtopic",
        ],
        prompt: None,
        asserts: &[],
    },
    Page {
        url: "forum.holloway.local/feedback",
        title: "site feedback",
        load_seconds: 2.8,
        body: &[
            Block::Heading("site feedback"),
            Block::Lead("12 threads  Â·  problems with the forum  Â·  no moderator"),
            Block::Pre(
                "  THREAD                                     BY            REPLIES\n  can we get a search box                    j.alderete          0\n  member list will not page                  j.alderete          0\n  who is the administrator                   j.alderete          0\n  post editing is off                        d.okonkwo           0\n  ... 8 more, all 0 replies ...",
            ),
            Block::Text(
                "Twelve threads about this forum, every one of them unanswered, on a board whose only job is to be answered.",
            ),
            Block::Text(
                "The last of them was posted the day the software stopped being supported.",
            ),
            Block::Note("Last post 12 dec 2016. Administrator: unset."),
        ],
        links: &["forum.holloway.local"],
        prompt: None,
        asserts: &["knows_nobody_owns_it"],
    },
    Page {
        url: "forum.holloway.local/307",
        title: "the 3:07 thread",
        load_seconds: 2.4,
        body: &[
            Block::Heading("the 3:07 thread"),
            Block::Lead("general  Â·  started by r.mears  Â·  214 replies  Â·  open  Â·  STICKY"),
            Block::By {
                author: "r.mears",
                when: "02 nov 2021",
            },
            Block::Says(
                "my row says connected 3:07 AM. so does everybodys i have asked. that is eleven people. does anybody have a row that says something else",
            ),
            Block::By {
                author: "birdfeeder",
                when: "02 nov 2021",
            },
            Block::Says("mine says 3:07. i assumed that was when the server came up"),
            Block::By {
                author: "qwrtyu",
                when: "02 nov 2021",
            },
            Block::Says(
                "the server came up in 2016. i was here. it did not come up at 3:07, it came up at about two in the afternoon and it took the whole afternoon",
            ),
            Block::By {
                author: "pigeonhole",
                when: "03 nov 2021",
            },
            Block::Says(
                "it is a default. some field somebody never filled in. this is not a mystery it is a form",
            ),
            Block::By {
                author: "r.mears",
                when: "06 nov 2021",
            },
            Block::Says(
                "i found the form. field 03. it is a constant. so pigeonhole is right and it is still the strangest thing i have seen at work, because it means nobody has ever recorded when any of us actually arrived",
            ),
            Block::By {
                author: "r.mears",
                when: "08 nov 2021",
            },
            Block::Says("212 replies and nobody has said what time they got here"),
            Block::Note("Last reply 08 nov 2021. Thread open. Stickied by r.mears."),
        ],
        links: &[
            "holloway.local/forms/hire",
            "forum.holloway.local/kit14",
            "forum.holloway.local/general",
        ],
        prompt: None,
        asserts: &["heard_the_hour", "read_the_thread"],
    },
    Page {
        url: "forum.holloway.local/count",
        title: "count of registered users",
        load_seconds: 2.1,
        body: &[
            Block::Heading("count of registered users"),
            Block::Lead("general  Â·  started by qwrtyu  Â·  401 replies  Â·  open  Â·  STICKY"),
            Block::By {
                author: "qwrtyu",
                when: "2015 - 2021",
            },
            Block::Says(
                "posting the count every friday so we have a record of it. no reason. i just like numbers",
            ),
            Block::Pre("  8,794     8,801     8,808     8,815     8,815     8,815"),
            Block::By {
                author: "birdfeeder",
                when: "05 nov 2021",
            },
            Block::Says(
                "seven a week, every week, for years, and then it stops dead. we do not hire seven a week. we hired four people last year",
            ),
            Block::By {
                author: "qwrtyu",
                when: "05 nov 2021",
            },
            Block::Says(
                "i know. i have been posting it for six years and nobody has ever been able to tell me what the other three are",
            ),
            Block::By {
                author: "pigeonhole",
                when: "05 nov 2021",
            },
            Block::Says("off by one. every program has one"),
            Block::By {
                author: "qwrtyu",
                when: "05 nov 2021",
            },
            Block::Says("the one it is off by is whoever asked"),
            Block::Note("Last reply 05 nov 2021. Thread open. Ask the bot: 8,814."),
        ],
        links: &[
            "iam.holloway.local/sessions",
            "forum.holloway.local/members",
            "forum.holloway.local/general",
        ],
        prompt: None,
        asserts: &["heard_the_count", "knows_the_count_was_wrong"],
    },
    Page {
        url: "forum.holloway.local/members",
        title: "Member List",
        load_seconds: 1.7,
        body: &[
            Block::Heading("Member List"),
            Block::Lead("8,815 members. Look one up by handle."),
            Block::Text(
                "Profiles show join date, post count and last activity. The member list itself cannot be paged through on this host.",
            ),
            Block::Note("Sorted by join date. Oldest: newuser_38, 02 aug 2016."),
        ],
        links: &["forum.holloway.local"],
        prompt: Some(Prompt {
            label: "member",
            hint: "handle or name",
            refusal: "No such member. 8,815 members are registered and that is not one of them.",
            answers: &[
                Answer {
                    typed: "r.mears",
                    reply: &[
                        Block::Rule,
                        Block::Heading("r.mears"),
                        Block::Pre(
                            "  handle       newuser_40\n  joined       05 nov 2021, 03:07\n  posts        1,104\n  last post    09 nov 2021\n  last seen    09 nov 2021, 03:07\n  status       online",
                        ),
                        Block::Text("POSTS"),
                        Block::Pre(
                            "  the 3:07 thread                 general          214\n  workstation 14                  kit               11\n  hello from newuser_40           introductions      1\n  count of registered users       general           38",
                        ),
                        Block::Text(
                            "Last seen three years ago. Status online. Both fields are generated from the session log.",
                        ),
                    ],
                    asserts: &["looked_up_a_member"],
                },
                Answer {
                    typed: "d.okonkwo",
                    reply: &[
                        Block::Rule,
                        Block::Heading("d.okonkwo"),
                        Block::Pre(
                            "  handle       mothra_88\n  joined       14 mar 2019, 03:07\n  posts        602\n  last post    22 mar 2019\n  last seen    22 mar 2019, 03:07\n  status       online",
                        ),
                        Block::Text("POSTS"),
                        Block::Pre(
                            "  the kitchen tap                 offtopic          188\n  new starter in the list         general            41\n  is that not a problem           kit                 1\n  the signup form                 general            12",
                        ),
                        Block::Text(
                            "Handle chosen by the member. Most members here never chose one and post under the number they were given.",
                        ),
                    ],
                    asserts: &["looked_up_a_member"],
                },
                Answer {
                    typed: "mothra_88",
                    reply: &[
                        Block::Rule,
                        Block::Heading("mothra_88"),
                        Block::Pre(
                            "  name         d.okonkwo\n  handle       mothra_88\n  joined       14 mar 2019, 03:07\n  posts        602\n  status       online",
                        ),
                        Block::Text(
                            "This member sat at workstation 14. The desk is reassigned automatically.",
                        ),
                    ],
                    asserts: &["looked_up_a_member"],
                },
                Answer {
                    typed: "sundial",
                    reply: &[
                        Block::Rule,
                        Block::Heading("sundial"),
                        Block::Pre(
                            "  handle       sundial\n  joined       14 mar 2019, 03:07\n  posts        11\n  posts shown  4\n  last post    14 mar 2019\n  status       online",
                        ),
                        Block::Text("POSTS"),
                        Block::Pre(
                            "  the lounge has been open since 2016                KEPT\n  mine says 3:07                                     KEPT\n  the count is 8815                                  KEPT\n  i am going to stop asking                          KEPT\n  [7 posts unavailable]",
                        ),
                        Block::Text(
                            "Eleven posts. Four available. The seven that are not all end in a question mark.",
                        ),
                    ],
                    asserts: &["saw_a_missing_line", "knows_which_lines_go"],
                },
                Answer {
                    typed: "j.alderete",
                    reply: &[
                        Block::Rule,
                        Block::Heading("j.alderete"),
                        Block::Pre(
                            "  handle       newuser_38\n  joined       02 aug 2016, 03:07\n  posts        44\n  role         moderator\n  last post    02 aug 2016\n  status       online",
                        ),
                        Block::Text("POSTS"),
                        Block::Pre(
                            "  read this before posting        introductions   STICKY\n  template for introductions      introductions   STICKY\n  can we have a delete button     site feedback\n  hello from newuser_38           introductions",
                        ),
                        Block::Text(
                            "Member number one. There is no member who joined before this one, and the forum has 8,815 of them.",
                        ),
                    ],
                    asserts: &["knows_there_is_no_first"],
                },
                Answer {
                    typed: "newuser_41",
                    reply: &[
                        Block::Rule,
                        Block::Heading("newuser_41"),
                        Block::Pre(
                            "  handle       newuser_41\n  joined       05 nov 2024, 03:07\n  posts        1\n  last post    05 nov 2024\n  status       online",
                        ),
                        Block::Text("POSTS"),
                        Block::Pre("  hello from newuser_41           introductions"),
                        Block::Text("You have not posted on this forum."),
                    ],
                    asserts: &["read_my_own_introduction"],
                },
                Answer {
                    typed: "newuser_37",
                    reply: &[
                        Block::Rule,
                        Block::Heading("newuser_37"),
                        Block::Pre(
                            "  handle       newuser_37\n  joined       --\n  posts        1\n  last post    --\n  status       online",
                        ),
                        Block::Text("POSTS"),
                        Block::Pre("  hello from newuser_37           introductions"),
                        Block::Text(
                            "No join date. The account before the oldest account has a post in it and no date on which it was made.",
                        ),
                    ],
                    asserts: &["found_the_one_before"],
                },
                Answer {
                    typed: "qwrtyu",
                    reply: &[
                        Block::Rule,
                        Block::Heading("qwrtyu"),
                        Block::Pre(
                            "  handle       qwrtyu\n  joined       11 apr 2015, 03:07\n  posts        3,882\n  last post    05 nov 2021\n  status       online",
                        ),
                        Block::Text("POSTS"),
                        Block::Pre(
                            "  count of registered users       general          401\n  the 3:07 thread                 general           64\n  the kitchen tap                 off topic        914",
                        ),
                        Block::Text(
                            "Joined a year before the forum was installed, at the same minute as everybody else.",
                        ),
                    ],
                    asserts: &["looked_up_a_member"],
                },
            ],
        }),
        asserts: &[],
    },
    Page {
        url: "rot13.holloway.local",
        title: "toolbox - rot13",
        load_seconds: 0.6,
        body: &[
            Block::Heading("rot13"),
            Block::Lead("Rotates a string thirteen places."),
            Block::Text(
                "Rotating twice gives you back what you started with, which is the whole of the joke.",
            ),
            Block::Text(
                "This tool is called 41,203 times a day by services that have nobody left maintaining them.",
            ),
            Block::Note("Part of the toolbox. Owner: unset."),
        ],
        links: &["holloway.local"],
        prompt: Some(Prompt {
            label: "string",
            hint: "type anything",
            refusal: "Rotated. Rotate it again to get it back.",
            answers: &[
                Answer {
                    typed: "mothra",
                    reply: &[
                        Block::Rule,
                        Block::Pre("  in    mothra\n  out   zbguen"),
                        Block::Text(
                            "Rotate it again and you have it back. That is the whole of what this host does.",
                        ),
                    ],
                    asserts: &["knows_the_name_is_rotated"],
                },
                Answer {
                    typed: "newuser",
                    reply: &[
                        Block::Rule,
                        Block::Pre("  in    newuser\n  out   arjhfre"),
                        Block::Text(
                            "That one is asked for more than any other string on this host. It has been asked for 8,815 times.",
                        ),
                        Block::Text("Once for each."),
                    ],
                    asserts: &["knows_the_name_is_rotated"],
                },
            ],
        }),
        asserts: &[],
    },
    Page {
        url: "sundial.holloway.local",
        title: "sundial",
        load_seconds: 2.6,
        body: &[
            Block::Heading("sundial"),
            Block::Lead("personal page. not maintained. do not link this."),
            Block::Text(
                "i posted eleven lines in the lounge in 2019 and the server kept four of them. i have the other seven here because i wrote them down before i sent them.",
            ),
            Block::Pre(
                "  1.   the lounge has been open since 2016     KEPT\n  2.   who closed it last?                     GONE\n  3.   does anybody remember signing on?       GONE\n  4.   mine says 3:07                          KEPT\n  5.   why does everyone say 3:07?             GONE\n  6.   is anybody counting?                    GONE\n  7.   the count is 8815                       KEPT\n  8.   what happens at 8816?                   GONE\n  9.   has anybody ever left?                  GONE\n  10.  i am going to stop asking               KEPT\n  11.  is there anybody reading this?          GONE",
            ),
            Block::Text("it kept the four that did not ask anything."),
            Block::Note("last edited 2019. no edits since."),
        ],
        links: &["holloway.local"],
        prompt: None,
        asserts: &["knows_how_many_lines", "knows_which_lines_go"],
    },
];

pub struct Bookmark {
    pub url: &'static str,
    pub label: &'static str,
    pub requires: &'static [Fact],
    pub filed: bool,
}

pub const FOLDER: &str = "r's stuff";

pub const BOOKMARKS: &[Bookmark] = &[
    Bookmark {
        url: "holloway.local",
        label: "intranet",
        requires: &[],
        filed: false,
    },
    Bookmark {
        url: "holloway.local/search",
        label: "search",
        requires: &["saw_the_intranet"],
        filed: false,
    },
    Bookmark {
        url: "holloway.local/directory",
        label: "directory",
        requires: &["saw_the_intranet"],
        filed: false,
    },
    Bookmark {
        url: "iam.holloway.local",
        label: "iam server",
        requires: &["knows_sessions_never_close"],
        filed: false,
    },
    Bookmark {
        url: "forum.holloway.local",
        label: "forums",
        requires: &["saw_the_intranet"],
        filed: false,
    },
    Bookmark {
        url: "rot13.holloway.local",
        label: "rot13",
        requires: &["saw_the_toolbox"],
        filed: false,
    },
    Bookmark {
        url: "holloway.local/it",
        label: "it requests",
        requires: &["knows_read_only_is_a_lie"],
        filed: true,
    },
    Bookmark {
        url: "iam.holloway.local/sessions",
        label: "session log",
        requires: &["read_the_logs"],
        filed: true,
    },
    Bookmark {
        url: "holloway.local/forms/hire",
        label: "the form",
        requires: &["knows_the_form_is_constant"],
        filed: true,
    },
    Bookmark {
        url: "iam.holloway.local/transcript",
        label: "transcript",
        requires: &["knows_it_copies_you"],
        filed: true,
    },
    Bookmark {
        url: "iam.holloway.local/attachment",
        label: "transfer",
        requires: &["got_the_attachment"],
        filed: true,
    },
    Bookmark {
        url: "sundial.holloway.local",
        label: "sundial",
        requires: &["saw_a_missing_line"],
        filed: true,
    },
];

pub fn page_at(url: &str) -> Option<usize> {
    let wanted = tidy(url);
    PAGES.iter().position(|page| page.url == wanted)
}

pub fn tidy(url: &str) -> String {
    let trimmed = url.trim().to_lowercase();
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(&trimmed);
    let without_host = without_scheme
        .strip_prefix("www.")
        .unwrap_or(without_scheme);
    without_host.trim_end_matches('/').to_string()
}

pub fn host_of(url: &str) -> String {
    tidy(url).split('/').next().unwrap_or_default().to_string()
}

pub const LINK_NAMES: &[(&str, &str)] = &[
    ("holloway.local", "intranet"),
    ("holloway.local/search", "search"),
    ("holloway.local/directory", "staff directory"),
    ("holloway.local/it", "IT requests"),
    ("holloway.local/forms/hire", "provisioning form"),
    ("forum.holloway.local", "the forums"),
    ("forum.holloway.local/members", "member list"),
    ("iam.holloway.local", "the iam server"),
    ("iam.holloway.local/sessions", "session log"),
    ("iam.holloway.local/transcript", "my transcript"),
    ("iam.holloway.local/attachment", "transfer"),
    ("rot13.holloway.local", "rot13"),
    ("sundial.holloway.local", "sundial"),
];

pub fn name_of(url: &str) -> String {
    let tidied = tidy(url);
    if let Some((_, name)) = LINK_NAMES.iter().find(|(address, _)| *address == tidied) {
        return name.to_string();
    }
    match tidied.rsplit_once('/') {
        Some((_, last)) if !last.is_empty() => last.to_string(),
        _ => tidied.split('.').next().unwrap_or(&tidied).to_string(),
    }
}

pub fn answered(prompt: &Prompt, typed: &str) -> Option<usize> {
    let flat: String = typed
        .chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();
    if flat.is_empty() {
        return None;
    }
    prompt
        .answers
        .iter()
        .position(|answer| answer.typed == flat)
        .or_else(|| {
            prompt
                .answers
                .iter()
                .position(|answer| answer.typed.is_empty())
        })
}

pub fn bookmarks(facts: &[Fact], filed: bool) -> Vec<usize> {
    BOOKMARKS
        .iter()
        .enumerate()
        .filter(|(_, mark)| mark.filed == filed)
        .filter(|(_, mark)| mark.requires.iter().all(|fact| facts.contains(fact)))
        .map(|(index, _)| index)
        .collect()
}
