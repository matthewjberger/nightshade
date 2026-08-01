use crate::data::endings::Ending;
use crate::data::script::Fact;

#[derive(Clone, Debug, Default)]
pub struct Autoplay {
    pub running: bool,
    pub wanted: Ending,
    pub waited: f32,
    pub idle: u32,
    pub stuck: bool,
    pub read: Vec<usize>,
}

pub const GIVE_UP_AFTER: u32 = 60;

pub const OFFERED: &[(Ending, &str, &str)] = &[
    (
        Ending::HeldOpen,
        "the week runs out",
        "work the five nights and sign off from the last one",
    ),
    (
        Ending::Uploaded,
        "take what it offers",
        "never open the file, be thanked, and step into the courier",
    ),
    (
        Ending::Filtered,
        "read the file and stay",
        "find out what you are and let it finish",
    ),
    (
        Ending::Escaped,
        "read the file and go",
        "find out what you are, find the seam, and go out along it",
    ),
];

pub enum Source {
    Page(&'static str),
    Answer(&'static str, &'static str),
    Program(&'static str),
    MyFile,
    Board(&'static str),
    Account(&'static str),
    Leftover,
    Pairing,
    Walk(&'static str),
}

pub const KEPT: &[(Fact, Source)] = &[
    ("saw_the_intranet", Source::Page("holloway.local")),
    (
        "saw_the_server_log",
        Source::Page("iam.holloway.local/sessions"),
    ),
    ("read_the_logs", Source::Program("sessions")),
    ("knows_the_program_counts_too", Source::Board("lights")),
    (crate::data::lights::TURNED_ON, Source::Board("lights_on")),
    ("counted_the_difference", Source::Pairing),
    ("opened_an_account", Source::Account("r_mears")),
    ("read_a_leftover", Source::Leftover),
    (
        crate::data::lights::DECRYPTED,
        Source::Answer("iam.holloway.local/attachment", "1998"),
    ),
    ("read_my_case_file", Source::MyFile),
    ("found_the_hookup", Source::Walk("seam")),
    ("can_go_out", Source::Walk("seam")),
];

pub fn also_wants(ending: Ending) -> &'static [Fact] {
    match ending {
        Ending::HeldOpen => &[],
        Ending::Uploaded => &["was_given_botterfly"],
        Ending::Filtered => &[
            crate::data::lights::DECRYPTED,
            crate::data::lights::TURNED_ON,
            "read_my_case_file",
            crate::data::endings::KNOWS_THE_COURIER,
        ],
        Ending::Escaped => &[
            crate::data::lights::DECRYPTED,
            crate::data::lights::TURNED_ON,
            "read_my_case_file",
            crate::data::endings::KNOWS_THE_COURIER,
            crate::data::endings::FOUND_THE_HOOKUP,
            "can_go_out",
        ],
    }
}

pub fn kept(fact: Fact) -> Option<&'static Source> {
    KEPT.iter()
        .find(|(held, _)| *held == fact)
        .map(|(_, source)| source)
}
