nightshade_config::config! {
    pub enum Settings {
        "Beasts" {
            Near: number(4.0, 60.0, 14.0),
            Far: number(10.0, 120.0, 44.0),
            Aggro: number(2.0, 40.0, 11.0),
            Anger: number(1.0, 30.0, 6.0),
            Leash: number(5.0, 100.0, 30.0),
            RoamRadius: number(1.0, 30.0, 8.0),
            TargetRange: number(5.0, 100.0, 40.0),
        }
        "Fighting" {
            Melee: number(0.5, 6.0, 2.2),
            PlayerMelee: number(1.0, 10.0, 4.5),
            AttackPeriod: number(0.3, 5.0, 1.4),
            SwordDamage: number(1.0, 40.0, 12.0),
            AxeDamage: number(1.0, 40.0, 7.0),
            FistDamage: number(0.5, 20.0, 3.0),
            Knockback: number(0.0, 20.0, 7.5),
        }
    }
}
