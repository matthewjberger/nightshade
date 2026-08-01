use crate::data::script::Fact;
use nightshade_ui::data::icons::{IconGlyph, material};

pub struct Program {
    pub id: &'static str,
    pub label: &'static str,
    pub glyph: IconGlyph,
    pub opens_full: bool,
    pub requires: &'static [Fact],
    pub from_store: bool,
    pub installed_note: &'static str,
}

pub const PROGRAMS: &[Program] = &[
    Program {
        id: "iam",
        glyph: material::TOKEN,
        opens_full: true,
        label: "iam",
        requires: &[],
        from_store: false,
        installed_note: "",
    },
    Program {
        id: "board",
        glyph: material::ACCOUNT_TREE,
        opens_full: false,
        label: "board",
        requires: &[],
        from_store: false,
        installed_note: "",
    },
    Program {
        id: "notes",
        glyph: material::TEXT_FIELDS,
        opens_full: false,
        label: "notes",
        requires: &[],
        from_store: false,
        installed_note: "",
    },
    Program {
        id: "sessions",
        glyph: material::TABLE_CHART,
        opens_full: false,
        label: "sessions",
        requires: &["read_the_logs"],
        from_store: true,
        installed_note: "the machine keeps its own copy of what it connected to",
    },
    Program {
        id: "browser",
        glyph: material::LANGUAGE,
        opens_full: false,
        label: "browser",
        requires: &[],
        from_store: false,
        installed_note: "",
    },
    Program {
        id: "files",
        glyph: material::FOLDER,
        opens_full: false,
        label: "files",
        requires: &["opened_an_account"],
        from_store: true,
        installed_note: "nobody set the permissions, so the whole disk reads",
    },
    Program {
        id: "lights",
        glyph: material::LIGHTBULB,
        opens_full: false,
        label: "lights out",
        requires: &[crate::data::lights::DOWNLOADED],
        from_store: true,
        installed_note: "somebody wrote this at work and it is still going round",
    },
    Program {
        id: "office",
        glyph: material::ACCOUNT_TREE,
        opens_full: false,
        label: "the building",
        requires: &["downloaded_the_walk"],
        from_store: true,
        installed_note: "somebody wrote down the building from memory and it is nearly all there",
    },
    Program {
        id: "lights_on",
        glyph: material::LIGHTBULB,
        opens_full: false,
        label: "lights on",
        requires: &[crate::data::lights::DECRYPTED],
        from_store: false,
        installed_note: "the same program with one word changed, and nothing looked at it",
    },
    Program {
        id: "case",
        glyph: material::FOLDER,
        opens_full: true,
        label: "case files",
        requires: &[crate::data::lights::TURNED_ON],
        from_store: false,
        installed_note: "",
    },
    Program {
        id: "store",
        glyph: material::GRID_ON,
        opens_full: false,
        label: "software",
        requires: &[],
        from_store: false,
        installed_note: "",
    },
    Program {
        id: "botterfly",
        glyph: material::AUTO_AWESOME,
        opens_full: false,
        label: "Botterfly",
        requires: &["was_given_botterfly"],
        from_store: true,
        installed_note: "everybody you freed is already on it",
    },
    Program {
        id: "cross",
        glyph: material::SHUFFLE,
        opens_full: false,
        label: "cross-check",
        requires: &["saw_a_missing_line"],
        from_store: true,
        installed_note: "two records of the same week, and a way to set them beside each other",
    },
];

pub fn known(program: &Program, facts: &[Fact]) -> bool {
    program.requires.iter().all(|fact| facts.contains(fact))
}

pub fn comes_with_the_desk(program: &Program, facts: &[Fact]) -> bool {
    !program.from_store && known(program, facts)
}

pub fn index_of(id: &str) -> Option<usize> {
    PROGRAMS.iter().position(|program| program.id == id)
}
