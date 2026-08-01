nightshade_config::config! {
    pub enum Plan {
        "Village" {
            Houses: whole(2, 18, 10),
            Seed: whole(1, 9999, 1234),
            Street: whole(6, 16, 12),
            Square: whole(2, 5, 3),
            Spacing: whole(2, 5, 3),
        }
        "Country" {
            Trees: whole(0, 120, 60),
            Rocks: whole(0, 40, 14),
            Reach: number(10.0, 24.0, 16.0),
            Grass: color([0.33, 0.44, 0.24, 1.0]),
        }
    }
}
