use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ExplosionType {
    #[default]
    Peony,
    Chrysanthemum,
    Willow,
    Ring,
    MultiBreak,
    Glitter,
    Crackle,
    Palm,
    Crossette,
    Strobe,
    Kamuro,
    ShapeSmiley,
    ShapeHeart,
    ShapeStar,
    ShapeCircle,
    ShapeCross,
    ShapeDiamond,
    SplittingShell,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ShellType {
    #[default]
    Standard,
    Comet,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum EffectKind {
    #[default]
    Fire,
    Smoke,
    Sprite,
}

#[derive(Clone)]
pub enum ShowAction {
    Type(ExplosionType),
    TypeWithColor(ExplosionType, Vec3),
    Random,
    Salvo(u32),
    Symmetric(ExplosionType, u32),
}

#[derive(Clone)]
pub struct ShowEvent {
    pub time: f32,
    pub action: ShowAction,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Stats {
    pub shells: usize,
    pub sub_shells: usize,
    pub fire: usize,
    pub sprite_total: usize,
    pub sprite_active: usize,
    pub smoke: usize,
    pub emitters: usize,
}

pub const FIREWORK_COLORS: [[f32; 3]; 30] = [
    [1.0, 0.1, 0.1],
    [1.0, 0.2, 0.2],
    [0.1, 1.0, 0.1],
    [0.2, 1.0, 0.4],
    [0.1, 0.3, 1.0],
    [0.3, 0.5, 1.0],
    [1.0, 1.0, 0.1],
    [1.0, 0.9, 0.3],
    [1.0, 0.1, 1.0],
    [1.0, 0.4, 0.9],
    [0.1, 1.0, 1.0],
    [0.4, 1.0, 0.9],
    [1.0, 0.5, 0.0],
    [1.0, 0.6, 0.2],
    [1.0, 1.0, 1.0],
    [0.95, 0.95, 1.0],
    [1.0, 0.6, 0.8],
    [1.0, 0.4, 0.6],
    [0.5, 0.2, 1.0],
    [0.7, 0.3, 1.0],
    [1.0, 0.8, 0.3],
    [0.3, 1.0, 0.6],
    [1.0, 0.3, 0.5],
    [0.2, 0.8, 1.0],
    [1.0, 0.7, 0.0],
    [0.8, 1.0, 0.2],
    [1.0, 0.2, 0.6],
    [0.6, 0.9, 1.0],
    [1.0, 0.5, 0.3],
    [0.4, 0.6, 1.0],
];

pub const LAUNCH_TYPES: [ExplosionType; 24] = [
    ExplosionType::Peony,
    ExplosionType::Peony,
    ExplosionType::Peony,
    ExplosionType::Chrysanthemum,
    ExplosionType::Chrysanthemum,
    ExplosionType::Willow,
    ExplosionType::Ring,
    ExplosionType::MultiBreak,
    ExplosionType::MultiBreak,
    ExplosionType::Glitter,
    ExplosionType::Crackle,
    ExplosionType::Palm,
    ExplosionType::Palm,
    ExplosionType::Crossette,
    ExplosionType::Strobe,
    ExplosionType::Kamuro,
    ExplosionType::ShapeSmiley,
    ExplosionType::ShapeHeart,
    ExplosionType::ShapeStar,
    ExplosionType::ShapeCircle,
    ExplosionType::ShapeCross,
    ExplosionType::ShapeDiamond,
    ExplosionType::SplittingShell,
    ExplosionType::SplittingShell,
];

pub const SUB_TYPES: [ExplosionType; 5] = [
    ExplosionType::Peony,
    ExplosionType::Chrysanthemum,
    ExplosionType::Willow,
    ExplosionType::Glitter,
    ExplosionType::Crackle,
];

pub const STANDARD_BUTTONS: [(&str, ExplosionType); 11] = [
    ("Peony", ExplosionType::Peony),
    ("Chrysanthemum", ExplosionType::Chrysanthemum),
    ("Willow", ExplosionType::Willow),
    ("Ring", ExplosionType::Ring),
    ("MultiBreak", ExplosionType::MultiBreak),
    ("Glitter", ExplosionType::Glitter),
    ("Crackle", ExplosionType::Crackle),
    ("Palm", ExplosionType::Palm),
    ("Crossette", ExplosionType::Crossette),
    ("Strobe", ExplosionType::Strobe),
    ("Kamuro", ExplosionType::Kamuro),
];

pub const SHAPE_BUTTONS: [(&str, ExplosionType); 6] = [
    ("Smiley", ExplosionType::ShapeSmiley),
    ("Heart", ExplosionType::ShapeHeart),
    ("Star", ExplosionType::ShapeStar),
    ("Circle", ExplosionType::ShapeCircle),
    ("Cross", ExplosionType::ShapeCross),
    ("Diamond", ExplosionType::ShapeDiamond),
];

pub const PARTICLE_TEXTURES: &[(u32, &[u8])] = &[
    (
        1,
        include_bytes!("../../../../assets/particles/flame_01.png"),
    ),
    (
        2,
        include_bytes!("../../../../assets/particles/flame_02.png"),
    ),
    (
        3,
        include_bytes!("../../../../assets/particles/flame_06.png"),
    ),
    (
        4,
        include_bytes!("../../../../assets/particles/smoke_01.png"),
    ),
    (
        5,
        include_bytes!("../../../../assets/particles/smoke_04.png"),
    ),
    (
        6,
        include_bytes!("../../../../assets/particles/smoke_07.png"),
    ),
    (
        7,
        include_bytes!("../../../../assets/particles/spark_01.png"),
    ),
    (
        8,
        include_bytes!("../../../../assets/particles/spark_05.png"),
    ),
    (
        9,
        include_bytes!("../../../../assets/particles/star_04.png"),
    ),
    (
        10,
        include_bytes!("../../../../assets/particles/star_06.png"),
    ),
    (
        11,
        include_bytes!("../../../../assets/particles/twirl_01.png"),
    ),
    (
        12,
        include_bytes!("../../../../assets/particles/magic_01.png"),
    ),
    (
        13,
        include_bytes!("../../../../assets/particles/magic_04.png"),
    ),
    (
        14,
        include_bytes!("../../../../assets/particles/circle_01.png"),
    ),
    (
        15,
        include_bytes!("../../../../assets/particles/circle_05.png"),
    ),
    (
        16,
        include_bytes!("../../../../assets/particles/flare_01.png"),
    ),
    (
        17,
        include_bytes!("../../../../assets/particles/light_01.png"),
    ),
    (
        18,
        include_bytes!("../../../../assets/particles/fire_01.png"),
    ),
    (
        19,
        include_bytes!("../../../../assets/particles/scorch_01.png"),
    ),
    (
        20,
        include_bytes!("../../../../assets/particles/slash_01.png"),
    ),
    (
        21,
        include_bytes!("../../../../assets/particles/symbol_01.png"),
    ),
    (
        22,
        include_bytes!("../../../../assets/particles/trace_05.png"),
    ),
    (
        23,
        include_bytes!("../../../../assets/particles/muzzle_03.png"),
    ),
    (
        24,
        include_bytes!("../../../../assets/particles/window_01.png"),
    ),
];

pub fn show_script() -> Vec<ShowEvent> {
    let mut events = Vec::new();
    let mut time = 0.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::Peony),
    });
    time += 1.5;
    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::Chrysanthemum),
    });
    time += 1.5;
    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::Willow),
    });
    time += 2.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Symmetric(ExplosionType::ShapeHeart, 3),
    });
    time += 3.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::SplittingShell),
    });
    time += 2.5;
    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::SplittingShell),
    });
    time += 3.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Symmetric(ExplosionType::ShapeStar, 5),
    });
    time += 3.5;

    for _ in 0..3 {
        events.push(ShowEvent {
            time,
            action: ShowAction::Random,
        });
        time += 0.8;
    }
    time += 1.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Symmetric(ExplosionType::ShapeSmiley, 3),
    });
    time += 3.5;

    events.push(ShowEvent {
        time,
        action: ShowAction::Salvo(5),
    });
    time += 2.5;

    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::Kamuro),
    });
    time += 2.0;
    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::Kamuro),
    });
    time += 3.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Symmetric(ExplosionType::ShapeCross, 4),
    });
    time += 3.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::SplittingShell),
    });
    events.push(ShowEvent {
        time,
        action: ShowAction::Type(ExplosionType::SplittingShell),
    });
    time += 3.5;

    for _ in 0..5 {
        events.push(ShowEvent {
            time,
            action: ShowAction::Random,
        });
        time += 0.5;
    }
    time += 1.5;

    events.push(ShowEvent {
        time,
        action: ShowAction::Symmetric(ExplosionType::ShapeDiamond, 5),
    });
    time += 3.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Salvo(8),
    });
    time += 2.0;

    for _ in 0..8 {
        events.push(ShowEvent {
            time,
            action: ShowAction::Random,
        });
        time += 0.4;
    }
    time += 2.0;

    events.push(ShowEvent {
        time,
        action: ShowAction::Symmetric(ExplosionType::ShapeHeart, 7),
    });
    time += 4.0;

    time += 2.0;

    for burst in 0..15 {
        let count = 3 + burst / 3;
        events.push(ShowEvent {
            time,
            action: ShowAction::Salvo(count),
        });
        time += 0.3 - (burst as f32 * 0.01);
    }

    for _ in 0..30 {
        events.push(ShowEvent {
            time,
            action: ShowAction::Random,
        });
        time += 0.08;
    }

    events.push(ShowEvent {
        time,
        action: ShowAction::Symmetric(ExplosionType::ShapeHeart, 5),
    });
    events.push(ShowEvent {
        time,
        action: ShowAction::Symmetric(ExplosionType::ShapeStar, 5),
    });
    events.push(ShowEvent {
        time,
        action: ShowAction::TypeWithColor(ExplosionType::Kamuro, Vec3::new(1.0, 0.8, 0.2)),
    });
    events.push(ShowEvent {
        time,
        action: ShowAction::TypeWithColor(ExplosionType::Kamuro, Vec3::new(1.0, 0.3, 0.1)),
    });
    time += 0.5;

    for _ in 0..20 {
        events.push(ShowEvent {
            time,
            action: ShowAction::Random,
        });
        time += 0.05;
    }

    events
}
