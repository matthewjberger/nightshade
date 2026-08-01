use blocker_player::data::ask::Ask;
use blocker_screen::data::screen::Screen;

const _: () = assert!(
    blocker_crew::data::people::persona_named(blocker_chat::data::chat::GATE_CALLER)
        == blocker_crew::data::people::GATE_PERSONAS[0]
);

pub struct Toggle {
    pub ask: Ask,
    pub screen: Screen,
}

pub const TOGGLES: [Toggle; 4] = [
    Toggle {
        ask: Ask::Pocket,
        screen: Screen::Inventory,
    },
    Toggle {
        ask: Ask::Map,
        screen: Screen::Map,
    },
    Toggle {
        ask: Ask::Character,
        screen: Screen::Character,
    },
    Toggle {
        ask: Ask::Journal,
        screen: Screen::Journal,
    },
];
