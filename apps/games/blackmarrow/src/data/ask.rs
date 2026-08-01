use nightshade_input::{Action, Binding, Pointer};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Fly,
    Rise,
    Sink,
    Haste,
    Reach,
    Throw,
    Pocket,
    Torch,
    Pause,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Fly,
            Self::Rise,
            Self::Sink,
            Self::Haste,
            Self::Reach,
            Self::Throw,
            Self::Pocket,
            Self::Torch,
            Self::Pause,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Fly => "fly",
            Self::Rise => "rise",
            Self::Sink => "sink",
            Self::Haste => "haste",
            Self::Reach => "reach",
            Self::Throw => "throw",
            Self::Pocket => "pocket",
            Self::Torch => "torch",
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Fly => &[Binding::Keys {
                up: KeyCode::KeyW,
                down: KeyCode::KeyS,
                left: KeyCode::KeyA,
                right: KeyCode::KeyD,
            }],
            Self::Rise => &[Binding::Key(KeyCode::Space)],
            Self::Sink => &[
                Binding::Key(KeyCode::ControlLeft),
                Binding::Key(KeyCode::KeyC),
            ],
            Self::Haste => &[Binding::Key(KeyCode::ShiftLeft)],
            Self::Reach => &[Binding::Pointer(Pointer::Left)],
            Self::Throw => &[Binding::Pointer(Pointer::Right)],
            Self::Pocket => &[Binding::Key(KeyCode::KeyE)],
            Self::Torch => &[Binding::Key(KeyCode::KeyF)],
            Self::Pause => &[Binding::Key(KeyCode::Escape)],
        }
    }
}
