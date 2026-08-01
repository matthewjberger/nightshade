pub mod enemies;
pub mod towers;

pub use enemies::*;
pub use towers::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GameState {
    #[default]
    WaitingForWave,
    WaveInProgress,
    GameOver,
}

#[derive(Debug, Clone)]
pub enum UiAction {
    SelectTower(TowerType),
    DecreaseSpeed,
    IncreaseSpeed,
    ResetSpeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EffectType {
    #[default]
    Explosion,
    PoisonBubble,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    PickBasic,
    PickFrost,
    PickCannon,
    PickSniper,
    PickPoison,
    SlowDown,
    SpeedUp,
    ResetSpeed,
    ResetCamera,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::PickBasic,
            Self::PickFrost,
            Self::PickCannon,
            Self::PickSniper,
            Self::PickPoison,
            Self::SlowDown,
            Self::SpeedUp,
            Self::ResetSpeed,
            Self::ResetCamera,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::PickBasic => "pick basic",
            Self::PickFrost => "pick frost",
            Self::PickCannon => "pick cannon",
            Self::PickSniper => "pick sniper",
            Self::PickPoison => "pick poison",
            Self::SlowDown => "slow down",
            Self::SpeedUp => "speed up",
            Self::ResetSpeed => "reset speed",
            Self::ResetCamera => "reset camera",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_input::Binding;
        use nightshade_platform::KeyCode;
        match self {
            Self::PickBasic => &[Binding::Key(KeyCode::Digit1)],
            Self::PickFrost => &[Binding::Key(KeyCode::Digit2)],
            Self::PickCannon => &[Binding::Key(KeyCode::Digit3)],
            Self::PickSniper => &[Binding::Key(KeyCode::Digit4)],
            Self::PickPoison => &[Binding::Key(KeyCode::Digit5)],
            Self::SlowDown => &[Binding::Key(KeyCode::BracketLeft)],
            Self::SpeedUp => &[Binding::Key(KeyCode::BracketRight)],
            Self::ResetSpeed => &[Binding::Key(KeyCode::Backslash)],
            Self::ResetCamera => &[Binding::Key(KeyCode::KeyC), Binding::Key(KeyCode::Home)],
        }
    }
}
