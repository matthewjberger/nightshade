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
pub struct CoinDashResources {
    pub round: RoundState,
    pub ui: UiHandles,
}

pub struct RoundState {
    pub screen: Screen,
    pub round: u32,
    pub goal: u32,
    pub score: u32,
    pub time: f32,
    pub rng: fastrand::Rng,
}

impl Default for RoundState {
    fn default() -> Self {
        Self {
            screen: Screen::Title,
            round: 1,
            goal: 8,
            score: 0,
            time: crate::systems::world::ROUND_SECONDS,
            rng: fastrand::Rng::with_seed(0x9E37_79B9),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub hud: Option<Entity>,
    pub hud_label: Option<Entity>,
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
