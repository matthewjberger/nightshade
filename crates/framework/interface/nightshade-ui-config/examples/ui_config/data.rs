nightshade_config::config! {
    pub enum Panel {
        "Motion" {
            Spin: number(0.0, 4.0, 1.2),
            Turning: toggle(true),
        }
        "Body" {
            Lift: number(0.0, 2.0, 0.0),
            Palette: choice(0, ["Ember", "Meadow", "Ice"]),
        }
    }
}
