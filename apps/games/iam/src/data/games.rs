use crate::data::script::Fact;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Board {
    Rotate,
    Count,
}

pub struct Posted {
    pub id: &'static str,
    pub author: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    pub room: &'static str,
    pub board: Board,
    pub start: &'static str,
    pub goal: &'static str,
    pub requires: &'static [Fact],
    pub asserts: &'static [Fact],
    pub installs: Option<&'static str>,
    pub prize: &'static str,
}

pub const POSTED: &[Posted] = &[
    Posted {
        id: "lamps",
        author: "birdfeeder",
        title: "download lights out",
        blurb: "made this at work. pushed it to the software list so it is easier to get",
        room: "general",
        board: Board::Rotate,
        start: "",
        goal: "",
        requires: &[],
        asserts: &[crate::data::lights::DOWNLOADED],
        prize: "",
        installs: Some("lights"),
    },
    Posted {
        id: "walk",
        author: "mothra_88",
        title: "download the building",
        blurb: "wrote down the office from memory. it is nearly all there",
        room: "offtopic",
        board: Board::Rotate,
        start: "",
        goal: "",
        requires: &[],
        asserts: &["downloaded_the_walk"],
        prize: "",
        installs: Some("office"),
    },
    Posted {
        id: "rotate",
        author: "rot13bot",
        title: "turn it until it reads",
        blurb: "i do this all day. you try it",
        room: "rot13",
        board: Board::Rotate,
        start: "arjhfre",
        goal: "newuser",
        requires: &["saw_the_toolbox"],
        asserts: &["knows_the_name_is_rotated"],
        prize: "rot13bot: that is the one i am asked for most",
        installs: None,
    },
    Posted {
        id: "count",
        author: "pigeonhole",
        title: "count the room",
        blurb: "serverbot and i do not agree. see who you believe",
        room: "3-07",
        board: Board::Count,
        start: "8814",
        goal: "8815",
        requires: &["entered_the_hour"],
        asserts: &["knows_the_count_was_wrong"],
        prize: "pigeonhole: it takes one off every time. i stopped arguing",
        installs: None,
    },
];

pub fn in_room(room: &str, facts: &[Fact]) -> Vec<usize> {
    POSTED
        .iter()
        .enumerate()
        .filter(|(_, posted)| posted.room == room)
        .filter(|(_, posted)| posted.requires.iter().all(|fact| facts.contains(fact)))
        .map(|(index, _)| index)
        .collect()
}

pub fn solved(posted: &Posted, state: &str) -> bool {
    state == posted.goal
}

pub fn turned(board: Board, face: char) -> char {
    match board {
        Board::Rotate => match face {
            'a'..='y' => ((face as u8) + 1) as char,
            'z' => 'a',
            other => other,
        },
        Board::Count => match face.to_digit(10) {
            Some(value) => char::from_digit((value + 1) % 10, 10).unwrap_or(face),
            None => face,
        },
    }
}
