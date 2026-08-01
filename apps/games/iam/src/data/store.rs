use crate::data::script::Fact;

pub struct Listing {
    pub program: Option<&'static str>,
    pub title: &'static str,
    pub by: &'static str,
    pub size: &'static str,
    pub blurb: &'static str,
    pub note: &'static str,
    pub requires: &'static [Fact],
}

pub const LISTINGS: &[Listing] = &[
    Listing {
        program: Some("sessions"),
        title: "Session Reader",
        by: "network services",
        size: "104 KB",
        blurb: "reads the log this machine keeps of what it has connected to",
        note: "supplied with the desk. reinstall if the icon is missing.",
        requires: &[],
    },
    Listing {
        program: Some("files"),
        title: "Disk Browser",
        by: "network services",
        size: "212 KB",
        blurb: "opens the disk. permissions were never set, so it opens all of it",
        note: "no maintainer. last build 2016.",
        requires: &[],
    },
    Listing {
        program: None,
        title: "Timesheet Client",
        by: "network services",
        size: "1.1 MB",
        blurb: "record your hours against a project code",
        note: "unavailable. no project codes have been issued since 2016.",
        requires: &[],
    },
    Listing {
        program: None,
        title: "Print Queue",
        by: "network services",
        size: "88 KB",
        blurb: "see what is waiting at the printer on your floor",
        note: "unavailable. no printer is registered to this floor.",
        requires: &[],
    },
    Listing {
        program: None,
        title: "Room Booking",
        by: "facilities",
        size: "340 KB",
        blurb: "book a meeting room, a desk, or a parking space",
        note: "unavailable. one booking on file. it has no name against it.",
        requires: &[],
    },
    Listing {
        program: None,
        title: "Holloway Handbook",
        by: "people team",
        size: "6.4 MB",
        blurb: "everything a new starter needs in their first week",
        note: "unavailable. the people team has no members.",
        requires: &[],
    },
    Listing {
        program: None,
        title: "Leavers Checklist",
        by: "people team",
        size: "72 KB",
        blurb: "what to do when somebody moves on",
        note: "unavailable. this form has never been opened.",
        requires: &[],
    },
    Listing {
        program: None,
        title: "Backup Agent",
        by: "network services",
        size: "2.8 MB",
        blurb: "keeps a copy of your work somewhere that is not this machine",
        note: "unavailable. no destination is configured and none can be.",
        requires: &[],
    },
    Listing {
        program: Some("lights"),
        title: "Lights Out",
        by: "b.finch",
        size: "38 KB",
        blurb: "turn them all off. it is harder than it looks",
        note: "pushed to this list by the author in 2016. still the only thing here anybody made.",
        requires: &[crate::data::lights::DOWNLOADED],
    },
    Listing {
        program: Some("office"),
        title: "The Building",
        by: "mothra_88",
        size: "61 KB",
        blurb: "somebody wrote down the office from memory and it is nearly all there",
        note: "pushed to this list from offtopic. the third floor is missing.",
        requires: &["downloaded_the_walk"],
    },
    Listing {
        program: Some("cross"),
        title: "Cross-Check",
        by: "b.finch",
        size: "17 KB",
        blurb: "two records of the same week, and a way to set them beside each other",
        note: "it has no opinion about any of them. it only puts them side by side.",
        requires: &["saw_a_missing_line"],
    },
    Listing {
        program: Some("botterfly"),
        title: "Botterfly",
        by: "iam network services",
        size: "4 KB",
        blurb: "stay in touch with your colleagues, wherever they end up",
        note: "recommended for you. verified publisher. 8,814 people you know are here.",
        requires: &["was_given_botterfly"],
    },
];

pub const MOST_LISTINGS: usize = LISTINGS.len();

pub fn listed(facts: &[Fact]) -> Vec<usize> {
    LISTINGS
        .iter()
        .enumerate()
        .filter(|(_, listing)| listing.requires.iter().all(|fact| facts.contains(fact)))
        .map(|(index, _)| index)
        .collect()
}

pub fn matching(facts: &[Fact], typed: &str) -> Vec<usize> {
    let wanted = typed.trim().to_lowercase();
    listed(facts)
        .into_iter()
        .filter(|index| {
            if wanted.is_empty() {
                return true;
            }
            let listing = &LISTINGS[*index];
            [listing.title, listing.by, listing.blurb]
                .iter()
                .any(|field| field.to_lowercase().contains(&wanted))
        })
        .collect()
}

pub fn for_program(program: &str) -> Option<usize> {
    LISTINGS
        .iter()
        .position(|listing| listing.program == Some(program))
}
