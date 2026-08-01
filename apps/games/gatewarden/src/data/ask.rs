use nightshade_gamepad::contract::GamepadButton;
use nightshade_input::{Action, Binding, Stick};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Pause,
    Confirm,
    Alternate,
    Advance,
    TopLane,
    MiddleLane,
    BottomLane,
    EndTurn,
    Power,
    Cancel,
    Pick,
    FirstCard,
    SecondCard,
    ThirdCard,
    FourthCard,
    FifthCard,
    SixthCard,
    SeventhCard,
    EighthCard,
    CursorLeft,
    CursorRight,
    ZoneUp,
    ZoneDown,
    CursorSlide,
}

pub const LANES: [Ask; 3] = [Ask::TopLane, Ask::MiddleLane, Ask::BottomLane];

pub const HAND_CARDS: [Ask; 8] = [
    Ask::FirstCard,
    Ask::SecondCard,
    Ask::ThirdCard,
    Ask::FourthCard,
    Ask::FifthCard,
    Ask::SixthCard,
    Ask::SeventhCard,
    Ask::EighthCard,
];

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Pause,
            Self::Confirm,
            Self::Alternate,
            Self::Advance,
            Self::TopLane,
            Self::MiddleLane,
            Self::BottomLane,
            Self::EndTurn,
            Self::Power,
            Self::Cancel,
            Self::Pick,
            Self::FirstCard,
            Self::SecondCard,
            Self::ThirdCard,
            Self::FourthCard,
            Self::FifthCard,
            Self::SixthCard,
            Self::SeventhCard,
            Self::EighthCard,
            Self::CursorLeft,
            Self::CursorRight,
            Self::ZoneUp,
            Self::ZoneDown,
            Self::CursorSlide,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Confirm => "confirm",
            Self::Alternate => "alternate",
            Self::Advance => "advance",
            Self::TopLane => "top lane",
            Self::MiddleLane => "middle lane",
            Self::BottomLane => "bottom lane",
            Self::EndTurn => "end turn",
            Self::Power => "power",
            Self::Cancel => "cancel",
            Self::Pick => "pick",
            Self::FirstCard => "first card",
            Self::SecondCard => "second card",
            Self::ThirdCard => "third card",
            Self::FourthCard => "fourth card",
            Self::FifthCard => "fifth card",
            Self::SixthCard => "sixth card",
            Self::SeventhCard => "seventh card",
            Self::EighthCard => "eighth card",
            Self::CursorLeft => "cursor left",
            Self::CursorRight => "cursor right",
            Self::ZoneUp => "zone up",
            Self::ZoneDown => "zone down",
            Self::CursorSlide => "cursor slide",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Confirm => &[
                Binding::Key(KeyCode::Enter),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Alternate => &[Binding::Pad(GamepadButton::North)],
            Self::Advance => &[
                Binding::Key(KeyCode::Enter),
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::TopLane => &[
                Binding::Key(KeyCode::Digit1),
                Binding::Pad(GamepadButton::DPadUp),
            ],
            Self::MiddleLane => &[
                Binding::Key(KeyCode::Digit2),
                Binding::Pad(GamepadButton::South),
            ],
            Self::BottomLane => &[
                Binding::Key(KeyCode::Digit3),
                Binding::Pad(GamepadButton::DPadDown),
            ],
            Self::EndTurn => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Power => &[
                Binding::Key(KeyCode::KeyQ),
                Binding::Pad(GamepadButton::North),
            ],
            Self::Cancel => &[
                Binding::Key(KeyCode::Backspace),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Pick => &[Binding::Pad(GamepadButton::South)],
            Self::FirstCard => &[Binding::Key(KeyCode::Digit1)],
            Self::SecondCard => &[Binding::Key(KeyCode::Digit2)],
            Self::ThirdCard => &[Binding::Key(KeyCode::Digit3)],
            Self::FourthCard => &[Binding::Key(KeyCode::Digit4)],
            Self::FifthCard => &[Binding::Key(KeyCode::Digit5)],
            Self::SixthCard => &[Binding::Key(KeyCode::Digit6)],
            Self::SeventhCard => &[Binding::Key(KeyCode::Digit7)],
            Self::EighthCard => &[Binding::Key(KeyCode::Digit8)],
            Self::CursorLeft => &[Binding::Pad(GamepadButton::DPadLeft)],
            Self::CursorRight => &[Binding::Pad(GamepadButton::DPadRight)],
            Self::ZoneUp => &[
                Binding::Pad(GamepadButton::DPadUp),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::ZoneDown => &[
                Binding::Pad(GamepadButton::DPadDown),
                Binding::Pad(GamepadButton::RightBumper),
            ],
            Self::CursorSlide => &[Binding::Stick(Stick::Left)],
        }
    }
}
