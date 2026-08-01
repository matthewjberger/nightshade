nightshade_config::config! {
    pub enum Settings {
        "Moving" {
            WalkSpeed: number(1.0, 12.0, 4.3),
            SprintSpeed: number(1.0, 20.0, 6.5),
            CrouchSpeed: number(0.2, 5.0, 1.5),
            SwimSpeed: number(0.5, 8.0, 2.6),
            FlySpeed: number(2.0, 60.0, 12.0),
            JumpSpeed: number(2.0, 20.0, 8.6),
            Gravity: number(5.0, 60.0, 28.0),
            WaterGravity: number(0.0, 20.0, 6.0),
            GlideSpeed: number(2.0, 30.0, 11.0),
            GlideFall: number(0.5, 10.0, 2.6),
            MouseSensitivity: number(0.0005, 0.01, 0.0022),
            PadLookRate: number(0.5, 8.0, 2.8),
            ZoomSmooth: number(1.0, 40.0, 12.0),
            BoatSpeed: number(2.0, 20.0, 8.0),
        }
        "View" {
            GliderInFirstPerson: toggle(true),
        }
        "Living" {
            FoodBurn: number(0.0, 0.5, 0.055),
            FoodHeal: number(0.0, 20.0, 14.0),
            HealRate: number(0.0, 5.0, 0.5),
            StarveRate: number(0.0, 5.0, 0.35),
            DrownSeconds: number(2.0, 60.0, 12.0),
            DrownRate: number(0.0, 10.0, 2.0),
            FallFloor: number(1.0, 30.0, 9.0),
            FallHurt: number(0.0, 10.0, 1.6),
            PickupRadius: number(0.5, 5.0, 1.4),
            ItemLifetime: number(10.0, 3600.0, 300.0),
        }
    }
}
