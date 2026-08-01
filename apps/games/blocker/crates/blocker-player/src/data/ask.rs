#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Move,
    Look,
    Jump,
    Sink,
    Sprint,
    Fly,
    Watch,
    Slot1,
    Slot2,
    Slot3,
    Slot4,
    Slot5,
    Slot6,
    Slot7,
    Slot8,
    Slot9,
    PreviousSlot,
    NextSlot,
    Mine,
    Place,
    Drop,
    Act,
    Lock,
    Unlock,
    Back,
    Pocket,
    Chat,
    Cursor,
    Map,
    Character,
    Journal,
    Target,
    Nameplates,
}

impl Ask {
    pub const SLOTS: [Ask; 9] = [
        Ask::Slot1,
        Ask::Slot2,
        Ask::Slot3,
        Ask::Slot4,
        Ask::Slot5,
        Ask::Slot6,
        Ask::Slot7,
        Ask::Slot8,
        Ask::Slot9,
    ];
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Move,
            Self::Look,
            Self::Jump,
            Self::Sink,
            Self::Sprint,
            Self::Fly,
            Self::Watch,
            Self::Slot1,
            Self::Slot2,
            Self::Slot3,
            Self::Slot4,
            Self::Slot5,
            Self::Slot6,
            Self::Slot7,
            Self::Slot8,
            Self::Slot9,
            Self::PreviousSlot,
            Self::NextSlot,
            Self::Mine,
            Self::Place,
            Self::Drop,
            Self::Act,
            Self::Lock,
            Self::Unlock,
            Self::Back,
            Self::Pocket,
            Self::Chat,
            Self::Cursor,
            Self::Map,
            Self::Character,
            Self::Journal,
            Self::Target,
            Self::Nameplates,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Look => "look",
            Self::Jump => "jump",
            Self::Sink => "sink",
            Self::Sprint => "sprint",
            Self::Fly => "fly",
            Self::Watch => "watch",
            Self::Slot1 => "slot 1",
            Self::Slot2 => "slot 2",
            Self::Slot3 => "slot 3",
            Self::Slot4 => "slot 4",
            Self::Slot5 => "slot 5",
            Self::Slot6 => "slot 6",
            Self::Slot7 => "slot 7",
            Self::Slot8 => "slot 8",
            Self::Slot9 => "slot 9",
            Self::PreviousSlot => "previous slot",
            Self::NextSlot => "next slot",
            Self::Mine => "mine",
            Self::Place => "place",
            Self::Drop => "drop",
            Self::Act => "act",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
            Self::Back => "back",
            Self::Pocket => "pocket",
            Self::Chat => "chat",
            Self::Cursor => "cursor",
            Self::Map => "map",
            Self::Character => "character",
            Self::Journal => "journal",
            Self::Target => "target",
            Self::Nameplates => "nameplates",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
        use nightshade_input::{Binding, Pointer, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Move => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Look => &[Binding::Stick(Stick::Right)],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Sink => &[
                Binding::Key(KeyCode::ShiftLeft),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Sprint => &[
                Binding::Key(KeyCode::ControlLeft),
                Binding::Pad(GamepadButton::LeftThumb),
            ],
            Self::Fly => &[
                Binding::Key(KeyCode::KeyF),
                Binding::Pad(GamepadButton::DPadUp),
            ],
            Self::Watch => &[
                Binding::Key(KeyCode::KeyG),
                Binding::Pad(GamepadButton::DPadDown),
            ],
            Self::Slot1 => &[Binding::Key(KeyCode::Digit1)],
            Self::Slot2 => &[Binding::Key(KeyCode::Digit2)],
            Self::Slot3 => &[Binding::Key(KeyCode::Digit3)],
            Self::Slot4 => &[Binding::Key(KeyCode::Digit4)],
            Self::Slot5 => &[Binding::Key(KeyCode::Digit5)],
            Self::Slot6 => &[Binding::Key(KeyCode::Digit6)],
            Self::Slot7 => &[Binding::Key(KeyCode::Digit7)],
            Self::Slot8 => &[Binding::Key(KeyCode::Digit8)],
            Self::Slot9 => &[Binding::Key(KeyCode::Digit9)],
            Self::PreviousSlot => &[Binding::Pad(GamepadButton::LeftBumper)],
            Self::NextSlot => &[Binding::Pad(GamepadButton::RightBumper)],
            Self::Mine => &[
                Binding::Pointer(Pointer::Left),
                Binding::Trigger(GamepadAxis::RightTrigger),
            ],
            Self::Place => &[
                Binding::Pointer(Pointer::Right),
                Binding::Pad(GamepadButton::LeftTrigger),
            ],
            Self::Drop => &[
                Binding::Key(KeyCode::KeyQ),
                Binding::Pad(GamepadButton::North),
            ],
            Self::Act => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Lock => &[Binding::Pointer(Pointer::Left)],
            Self::Unlock => &[Binding::Key(KeyCode::Escape)],
            Self::Back => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Pocket => &[
                Binding::Key(KeyCode::KeyI),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Chat => &[
                Binding::Key(KeyCode::Enter),
                Binding::Key(KeyCode::NumpadEnter),
            ],
            Self::Cursor => &[
                Binding::Key(KeyCode::AltLeft),
                Binding::Pad(GamepadButton::RightThumb),
            ],
            Self::Map => &[Binding::Key(KeyCode::KeyM)],
            Self::Character => &[Binding::Key(KeyCode::KeyC)],
            Self::Journal => &[Binding::Key(KeyCode::KeyJ)],
            Self::Target => &[
                Binding::Key(KeyCode::Tab),
                Binding::Pad(GamepadButton::DPadRight),
            ],
            Self::Nameplates => &[Binding::Key(KeyCode::KeyV)],
        }
    }
}
