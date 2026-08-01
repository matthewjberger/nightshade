use nightshade_input::{Action, Binding};
use nightshade_platform::KeyCode;

pub const LINE_SLOTS: usize = 24;
pub const REPLY_SLOTS: usize = 4;
pub const CARD_SLOTS: usize = 3;
pub const SUMMARY_SLOTS: usize = 6;

pub const SHIFT_START_MINUTES: u32 = 23 * 60;
pub const MINUTES_PER_REPLY: u32 = 2;
pub const MINUTES_PER_DECISION: u32 = 4;
pub const SECONDS_PER_LETTER: f32 = 0.016;
pub const SECONDS_BETWEEN_LINES: f32 = 0.22;

pub const CARD_NAMES: [&str; CARD_SLOTS] = ["the reservoir", "the empty truck", "the second board"];

pub struct Reply {
    pub prompt: &'static str,
    pub lines: &'static [&'static str],
    pub gives_card: Option<usize>,
    pub closes_call: bool,
}

pub struct CardAnswer {
    pub call: usize,
    pub card: usize,
    pub lines: &'static [&'static str],
    pub gives_card: Option<usize>,
}

pub struct Call {
    pub caller: &'static str,
    pub opening: &'static [&'static str],
    pub replies: &'static [Reply],
    pub deflection: &'static [&'static str],
    pub can_dispatch: bool,
    pub dispatch_lines: &'static [&'static str],
    pub busy_lines: &'static [&'static str],
    pub hold_lines: &'static [&'static str],
}

pub const SENT_HIM_OUT: &[&str] = &[
    "CALLER: You sent the deputy up the ridge. The station is empty.",
    "CALLER: It is only you and the board now. That is usual here.",
];

pub const KEPT_HIM_IN: &[&str] = &[
    "CALLER: You kept the deputy at the station. That was careful.",
    "CALLER: Careful is not the same as correct. You will see which.",
];

pub const CALLS: [Call; 3] = [
    Call {
        caller: "MARION ASH   Ridge Road turnout",
        opening: &[
            "The jack drops. There is wind, and under the wind a radio.",
            "MARION ASH: Hale, it is Marion. I am out on the turnout.",
            "MARION ASH: The truck quit. Headlights on, engine gone cold.",
            "MARION ASH: I would walk it down. Tonight I would rather not.",
        ],
        replies: &[
            Reply {
                prompt: "Are you hurt?",
                lines: &[
                    "MARION ASH: No. Cold. I have the blanket from the back.",
                    "MARION ASH: You sound a long way off. Are you in the basement?",
                    "MARION ASH: You always sound like the basement.",
                ],
                gives_card: None,
                closes_call: false,
            },
            Reply {
                prompt: "Where exactly are you parked?",
                lines: &[
                    "MARION ASH: On the wide shoulder, past the gate.",
                    "MARION ASH: Above the reservoir. I can see the water.",
                    "MARION ASH: It is very still. It has not moved once.",
                ],
                gives_card: Some(0),
                closes_call: false,
            },
            Reply {
                prompt: "What is on your radio?",
                lines: &[
                    "MARION ASH: Four seconds of something. Then it starts over.",
                    "MARION ASH: It has done that since the engine quit.",
                    "MARION ASH: I counted. Two hundred and eleven times.",
                ],
                gives_card: None,
                closes_call: false,
            },
            Reply {
                prompt: "Stay in the truck. I will see what I can send.",
                lines: &["MARION ASH: Thank you. I will keep the doors shut."],
                gives_card: None,
                closes_call: true,
            },
        ],
        deflection: &["MARION ASH: I do not know anything about that, Hale."],
        can_dispatch: true,
        dispatch_lines: &[
            "You ring the deputy. He takes the Ridge Road at speed.",
            "His line drops off the board. The station is yours alone.",
        ],
        busy_lines: &["There is nobody at the station to send."],
        hold_lines: &[
            "You tell Marion to sit tight. The wind takes the rest.",
            "The jack goes dark with the radio still under it.",
        ],
    },
    Call {
        caller: "DEL ROURKE   reservoir house",
        opening: &[
            "The jack drops on water, and on nothing else for a while.",
            "DEL ROURKE: Switchboard. Rourke, down at the reservoir house.",
            "DEL ROURKE: There are lights on the water. Under it, I mean.",
            "DEL ROURKE: Nine years I have worked here. That is new.",
        ],
        replies: &[
            Reply {
                prompt: "How many lights?",
                lines: &[
                    "DEL ROURKE: Four. They come up, they hold, they go.",
                    "DEL ROURKE: Then four again. Same order. Same spacing.",
                ],
                gives_card: None,
                closes_call: false,
            },
            Reply {
                prompt: "Is anyone out there with you?",
                lines: &[
                    "DEL ROURKE: No. The gate is chained. I chained it at eight.",
                    "DEL ROURKE: There is a truck up on the shoulder, though.",
                    "DEL ROURKE: Been there a while. Headlights on.",
                ],
                gives_card: None,
                closes_call: false,
            },
            Reply {
                prompt: "Have you left the house tonight?",
                lines: &[
                    "DEL ROURKE: Once. To chain the gate. That is all.",
                    "DEL ROURKE: My boots are by the door and they are wet.",
                    "DEL ROURKE: I do not remember them being wet, Hale.",
                ],
                gives_card: None,
                closes_call: false,
            },
            Reply {
                prompt: "Stay inside. Chain the door as well.",
                lines: &["DEL ROURKE: Chained. I will keep the lamp on."],
                gives_card: None,
                closes_call: true,
            },
        ],
        deflection: &["DEL ROURKE: That is nothing to do with the water."],
        can_dispatch: true,
        dispatch_lines: &[
            "You ring the deputy. He starts down the service road.",
            "The line to the water goes quiet, and stays quiet.",
        ],
        busy_lines: &[
            "You reach for the deputy line. It is dead.",
            "He is on the ridge. There is nobody to send.",
        ],
        hold_lines: &[
            "You keep the station manned. Rourke says he understands.",
            "He does not sound as though he understands.",
        ],
    },
    Call {
        caller: "NO LINE NUMBER",
        opening: &[
            "One jack lights. The board gives it no line number.",
            "CALLER: Is the lamp still on the left of the log book?",
            "CALLER: It was on the left. I want to be sure it is still there.",
        ],
        replies: &[
            Reply {
                prompt: "Who is this?",
                lines: &[
                    "CALLER: Somebody who sat where you are sitting.",
                    "CALLER: The chair leans. You have noticed that it leans.",
                ],
                gives_card: None,
                closes_call: false,
            },
            Reply {
                prompt: "Where are you calling from?",
                lines: &[
                    "CALLER: The basement of the post office.",
                    "CALLER: Under the stairs there is a second board.",
                    "CALLER: Older than yours. If it rings, do not answer it.",
                ],
                gives_card: Some(2),
                closes_call: false,
            },
            Reply {
                prompt: "What do you want?",
                lines: &[
                    "CALLER: The same as you. To get to six in the morning.",
                    "CALLER: I did not, in the end. You still can.",
                ],
                gives_card: None,
                closes_call: false,
            },
            Reply {
                prompt: "I am ending this call.",
                lines: &[
                    "CALLER: Of course. Sit straight. Keep the lamp on the left.",
                    "The jack goes dark. The board says no line was open.",
                ],
                gives_card: None,
                closes_call: true,
            },
        ],
        deflection: &["CALLER: You are asking the board. The board is old."],
        can_dispatch: false,
        dispatch_lines: &[],
        busy_lines: &[],
        hold_lines: &[],
    },
];

pub const CARD_ANSWERS: [CardAnswer; 3] = [
    CardAnswer {
        call: 1,
        card: 0,
        lines: &[
            "DEL ROURKE: Marion Ash? That is the truck I told you about.",
            "DEL ROURKE: Hold on. I have the glasses on it now.",
            "DEL ROURKE: Hale, there is nobody in that truck.",
            "DEL ROURKE: The blanket is folded on the seat. Folded.",
        ],
        gives_card: Some(1),
    },
    CardAnswer {
        call: 2,
        card: 1,
        lines: &[
            "CALLER: Marion went up to see the water. Everyone does, once.",
            "CALLER: They come back up the bank by morning. Most of them.",
            "CALLER: Not all of them come back the same.",
        ],
        gives_card: None,
    },
    CardAnswer {
        call: 2,
        card: 0,
        lines: &[
            "CALLER: There was a road under that water, and houses on it.",
            "CALLER: The switchboard served them. This switchboard.",
            "CALLER: Nobody took the old board out. They only stopped using it.",
        ],
        gives_card: None,
    },
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Glass,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Glass]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Glass => "glass",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Glass => &[Binding::Key(KeyCode::F1)],
        }
    }
}
