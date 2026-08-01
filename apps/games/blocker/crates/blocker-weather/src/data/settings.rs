nightshade_config::config! {
    pub enum Settings {
        "Weather" {
            Rain: toggle(true),
            Snow: toggle(true),
            Lightning: toggle(true),
            Fog: toggle(true),
            Splashes: toggle(true),
            Wind: toggle(true),
            SnowCover: toggle(true),
            SnowCoverRate: number(0.0, 60.0, 12.0),
        }
        "Sky" {
            WeatherShortest: number(20.0, 600.0, 110.0),
            WeatherLongest: number(30.0, 1200.0, 280.0),
            CloudyShare: number(0.0, 1.0, 0.25),
            LightningGapMin: number(1.0, 60.0, 5.0),
            LightningGapMax: number(2.0, 120.0, 24.0),
        }
        "Seasons" {
            SeasonSeconds: number(60.0, 7200.0, 1200.0),
            SeasonMid: number(-20.0, 30.0, 8.0),
            SeasonSwing: number(0.0, 40.0, 16.0),
            NightChill: number(0.0, 20.0, 6.0),
            AridWarmth: number(0.0, 30.0, 10.0),
            ColdChill: number(0.0, 30.0, 12.0),
        }
    }
}
