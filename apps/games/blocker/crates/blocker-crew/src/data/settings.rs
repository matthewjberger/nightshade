nightshade_config::config! {
    pub enum Settings {
        "Villagers" {
            Speed: number(0.5, 6.0, 2.1),
            Rest: number(0.5, 10.0, 3.0),
            RestMin: number(0.0, 5.0, 0.8),
            Linger: number(0.5, 10.0, 2.6),
            WorkHour: number(0.0, 24.0, 7.0),
            NoonHour: number(0.0, 24.0, 11.5),
            AfternoonHour: number(0.0, 24.0, 13.5),
            EveningHour: number(0.0, 24.0, 17.5),
            BedHour: number(0.0, 24.0, 20.5),
        }
    }
}
