nightshade_config::config! {
    pub enum Settings {
        "Handling" {
            Engine: number(5.0, 40.0, 18.0),
            Braking: number(5.0, 60.0, 26.0),
            TopSpeed: number(10.0, 60.0, 30.0),
            Reverse: number(2.0, 20.0, 7.0),
            Drag: number(0.0, 2.0, 0.45),
            Grip: number(1.0, 15.0, 7.0),
            DriftGrip: number(0.2, 5.0, 1.3),
            TurnRate: number(0.5, 5.0, 2.3),
            SteerRate: number(1.0, 20.0, 6.0),
            KnockSpeed: number(1.0, 30.0, 6.0),
            ToppleSeconds: number(0.1, 3.0, 0.7),
            KnockSlide: number(0.0, 6.0, 1.5),
        }
        "On foot" {
            WalkSpeed: number(1.0, 8.0, 3.4),
            SprintSpeed: number(2.0, 14.0, 6.5),
            Acceleration: number(2.0, 40.0, 14.0),
            BoardReach: number(1.0, 8.0, 3.2),
            DoorReach: number(0.5, 6.0, 2.2),
        }
        "Traffic" {
            Traffic: number(0.0, 3.0, 1.0),
            Cruise: number(3.0, 25.0, 11.0),
            CruiseSpread: number(0.0, 0.8, 0.25),
            TurnChance: number(0.0, 1.0, 0.35),
            TurnSpeed: number(2.0, 12.0, 5.0),
            LightCycle: number(4.0, 40.0, 16.0),
            GapLook: number(6.0, 40.0, 16.0),
            GapStop: number(2.0, 12.0, 5.0),
        }
        "People" {
            Crowd: number(0.0, 3.0, 1.0),
            Stroll: number(0.5, 3.0, 1.3),
            Hurry: number(1.0, 3.0, 1.85),
            IdleChance: number(0.0, 1.0, 0.18),
            IdleSeconds: number(1.0, 20.0, 7.0),
            HitSpeed: number(1.0, 15.0, 4.0),
            DyingSeconds: number(1.0, 30.0, 6.0),
        }
        "Night" {
            StartHour: number(0.0, 24.0, 21.5),
            DaySpeed: number(0.0, 0.2, 0.0167),
            Bloom: number(0.0, 1.0, 0.3),
            Rain: number(0.0, 2000.0, 700.0),
            FogReach: number(50.0, 800.0, 330.0),
            LampLight: number(0.0, 200.0, 70.0),
            LampReach: number(4.0, 60.0, 26.0),
            RoomLight: number(0.0, 200.0, 40.0),
            NeonGlow: number(0.0, 20.0, 7.0),
            Contrast: number(0.5, 2.0, 1.12),
            Saturation: number(0.0, 2.0, 0.92),
            Vignette: number(0.0, 1.0, 0.28),
        }
        "Mayhem" {
            ComboSeconds: number(0.5, 10.0, 3.0),
            FlashSize: number(10.0, 80.0, 28.0),
            FlashSeconds: number(0.4, 5.0, 1.4),
            FlashRise: number(0.0, 6.0, 1.6),
        }
        "Camera" {
            ChaseBack: number(4.0, 25.0, 11.0),
            ChaseHeight: number(1.0, 15.0, 4.8),
            ChaseLag: number(1.0, 20.0, 6.0),
            WalkDistance: number(2.0, 14.0, 6.0),
            WalkPitch: number(-0.5, 1.2, 0.32),
            WalkHead: number(0.5, 3.0, 1.4),
            FlySpeed: number(5.0, 300.0, 45.0),
            FlyLook: number(0.001, 0.02, 0.004),
        }
        "City" {
            NeonChance: number(0.0, 1.0, 0.7),
            Parking: number(0.0, 1.0, 0.6),
            Trees: whole(0, 40, 10),
            TowerChance: number(0.0, 1.0, 0.2),
            Props: number(0.0, 3.0, 1.0),
            SiteChance: number(0.0, 1.0, 0.2),
            ViewCells: whole(1, 8, 5),
            KeepCells: whole(2, 10, 6),
            KeepBehind: whole(1, 6, 2),
            LifeCells: whole(1, 6, 2),
            LightCells: whole(1, 8, 3),
        }
        "Streaming" {
            Deadzone: number(0.0, 1.0, 0.35),
            SettleMillis: number(0.1, 8.0, 1.0),
            UnloadRoots: whole(8, 512, 32),
            ModelsPerFrame: whole(1, 16, 2),
            DressMillis: number(0.1, 8.0, 1.0),
        }
    }
}
