use nalgebra_glm::Vec2;
use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
use nightshade_platform::KeyCode;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const DEFAULT_PROFILE: &str = "default";

pub trait Action: Copy + Eq + std::hash::Hash + Send + Sync + 'static {
    fn all() -> &'static [Self];

    fn name(&self) -> &'static str;

    fn default_bindings(&self) -> &'static [Binding];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Pointer {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Stick {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Press {
    Key(KeyCode),
    Pointer(Pointer),
    Pad(GamepadButton),
    Trigger(GamepadAxis),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Binding {
    Key(KeyCode),
    Keys {
        up: KeyCode,
        down: KeyCode,
        left: KeyCode,
        right: KeyCode,
    },
    Pointer(Pointer),
    Pad(GamepadButton),
    Stick(Stick),
    Trigger(GamepadAxis),
    Pair {
        positive: Press,
        negative: Press,
    },
}

pub struct Bindings<A: Action> {
    pub profiles: HashMap<String, HashMap<A, Vec<Binding>>>,
    pub worn: String,
    pub revision: u64,
}

impl<A: Action> Default for Bindings<A> {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(DEFAULT_PROFILE.to_string(), filled::<A>());
        Self {
            profiles,
            worn: DEFAULT_PROFILE.to_string(),
            revision: 0,
        }
    }
}

pub fn filled<A: Action>() -> HashMap<A, Vec<Binding>> {
    let mut table = HashMap::new();
    for action in A::all() {
        table.insert(*action, action.default_bindings().to_vec());
    }
    table
}

pub struct Actions<A: Action> {
    pub held: HashSet<A>,
    pub pressed: HashSet<A>,
    pub released: HashSet<A>,
    pub axes: HashMap<A, Vec2>,
}

impl<A: Action> Default for Actions<A> {
    fn default() -> Self {
        Self {
            held: HashSet::new(),
            pressed: HashSet::new(),
            released: HashSet::new(),
            axes: HashMap::new(),
        }
    }
}

impl<A: Action> Actions<A> {
    pub fn pressed(&self, action: A) -> bool {
        self.held.contains(&action)
    }

    pub fn just_pressed(&self, action: A) -> bool {
        self.pressed.contains(&action)
    }

    pub fn just_released(&self, action: A) -> bool {
        self.released.contains(&action)
    }

    pub fn axis(&self, action: A) -> f32 {
        self.axes.get(&action).map(|axes| axes.x).unwrap_or(0.0)
    }

    pub fn axis2(&self, action: A) -> Vec2 {
        self.axes
            .get(&action)
            .copied()
            .unwrap_or_else(|| Vec2::new(0.0, 0.0))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EngineAction {
    Move,
    Look,
    Lift,
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
    Sprint,
    Crouch,
    Menu,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
}

impl Action for EngineAction {
    fn all() -> &'static [Self] {
        &[
            Self::Move,
            Self::Look,
            Self::Lift,
            Self::Primary,
            Self::Secondary,
            Self::Tertiary,
            Self::Quaternary,
            Self::Sprint,
            Self::Crouch,
            Self::Menu,
            Self::NavigateUp,
            Self::NavigateDown,
            Self::NavigateLeft,
            Self::NavigateRight,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Look => "look",
            Self::Lift => "lift",
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Tertiary => "tertiary",
            Self::Quaternary => "quaternary",
            Self::Sprint => "sprint",
            Self::Crouch => "crouch",
            Self::Menu => "menu",
            Self::NavigateUp => "navigate up",
            Self::NavigateDown => "navigate down",
            Self::NavigateLeft => "navigate left",
            Self::NavigateRight => "navigate right",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
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
            Self::Lift => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::Space),
                    negative: Press::Key(KeyCode::ControlLeft),
                },
                Binding::Pair {
                    positive: Press::Pad(GamepadButton::RightBumper),
                    negative: Press::Pad(GamepadButton::LeftBumper),
                },
            ],
            Self::Primary => &[
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::Enter),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Secondary => &[Binding::Pad(GamepadButton::East)],
            Self::Tertiary => &[Binding::Pad(GamepadButton::West)],
            Self::Quaternary => &[Binding::Pad(GamepadButton::North)],
            Self::Sprint => &[
                Binding::Key(KeyCode::ShiftLeft),
                Binding::Pad(GamepadButton::LeftThumb),
            ],
            Self::Crouch => &[
                Binding::Key(KeyCode::ControlLeft),
                Binding::Key(KeyCode::KeyC),
                Binding::Pad(GamepadButton::RightThumb),
            ],
            Self::Menu => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::NavigateUp => &[Binding::Pad(GamepadButton::DPadUp)],
            Self::NavigateDown => &[Binding::Pad(GamepadButton::DPadDown)],
            Self::NavigateLeft => &[Binding::Pad(GamepadButton::DPadLeft)],
            Self::NavigateRight => &[Binding::Pad(GamepadButton::DPadRight)],
        }
    }
}
