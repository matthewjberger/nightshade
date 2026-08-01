use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
    Over,
}

#[derive(Default)]
pub struct CoinRushResources {
    pub round: RoundState,
    pub ui: UiHandles,
}

pub struct RoundState {
    pub screen: Screen,
    pub round: u32,
    pub goal: u32,
    pub score: u32,
    pub roll: nightshade_random::Random,
}

impl Default for RoundState {
    fn default() -> Self {
        Self {
            screen: Screen::Title,
            round: 1,
            goal: crate::theme::FIRST_GOAL + crate::theme::GOAL_STEP,
            score: 0,
            roll: nightshade_random::Random::seeded(0x1234_5678),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub title: Option<Entity>,
    pub play_button: Option<Entity>,
    pub quit_button: Option<Entity>,
    pub pause: Option<Entity>,
    pub resume_button: Option<Entity>,
    pub restart_button: Option<Entity>,
    pub pause_title_button: Option<Entity>,
    pub over: Option<Entity>,
    pub over_score: Option<Entity>,
    pub again_button: Option<Entity>,
    pub over_title_button: Option<Entity>,
}
