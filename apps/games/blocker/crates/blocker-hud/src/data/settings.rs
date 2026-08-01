nightshade_config::config! {
    pub enum Settings {
        "Screen" {
            ShowRate: toggle(true),
            Sight: choice(crate::theme::SIGHT_DEFAULT, crate::theme::SIGHT_LABELS),
        }
    }
}
