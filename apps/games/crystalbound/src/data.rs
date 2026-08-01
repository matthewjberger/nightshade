use crate::contract::{Job, MonsterKind, Place};
use crate::theme;

pub const fn map_of(place: Place) -> &'static [&'static str] {
    match place {
        Place::Overworld => &OVERWORLD,
        Place::Town => &TOWN,
        Place::Dungeon => &VAULT,
    }
}

pub const OVERWORLD: [&str; 24] = [
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^",
    "^..............................^",
    "^...TT......,,......TT.........^",
    "^..T..........................T^",
    "^.........ppppppp..............^",
    "^....,....p.....p.......TT.....^",
    "^.........p..t..p..............^",
    "^....TT...p.....p.....,,.......^",
    "^.........pppp@pp..............^",
    "^..............p...............^",
    "^~~~~..........p..........TT...^",
    "^~~~~..........p...............^",
    "^~~~............p..............^",
    "^....,,.........ppppp..........^",
    "^....................p.........^",
    "^.......TT...........p...,,....^",
    "^....................p.........^",
    "^.........,,.........p.........^",
    "^....................d.........^",
    "^..............................^",
    "^....TT................TT......^",
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^",
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^",
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^",
];

pub const TOWN: [&str; 20] = [
    "############################",
    "#....TT..............TT....#",
    "#..........................#",
    "#..........................#",
    "#......I........S..........#",
    "#......p........p..........#",
    "#......p........p..........#",
    "#......pppppppppp..........#",
    "#...........p..............#",
    "#......,,...p..............#",
    "#...........p....C.........#",
    "#...........p..............#",
    "#....TT.....p.......,,.....#",
    "#...........p..............#",
    "#...........p..............#",
    "#...........x..............#",
    "#..........................#",
    "############################",
    "############################",
    "############################",
];

pub const VAULT: [&str; 22] = [
    "############################",
    "#.....#############........#",
    "#.....#..........#.........#",
    "#.....#..........#.........#",
    "#.....#....c.....#.........#",
    "#.....#..........#.........#",
    "#.....##.....#####.........#",
    "#............#.............#",
    "#............#.............#",
    "#####.########.............#",
    "#..........................#",
    "#..........................#",
    "#.....#####.....########...#",
    "#.....#..............#.....#",
    "#.....#......c.......#.....#",
    "#.....#..............#.....#",
    "#.....###############......#",
    "#..........................#",
    "#.............x............#",
    "############################",
    "############################",
    "############################",
];

pub const HERO_NAMES: [&str; 4] = ["Rell", "Ysolde", "Marek", "Anwen"];

pub struct JobStats {
    pub power: i32,
    pub guard: i32,
    pub speed: i32,
    pub health: i32,
    pub magic: i32,
}

pub const fn job_stats(job: Job) -> JobStats {
    match job {
        Job::Freelancer => JobStats {
            power: 6,
            guard: 3,
            speed: 6,
            health: 30,
            magic: 4,
        },
        Job::Fighter => JobStats {
            power: 11,
            guard: 6,
            speed: 5,
            health: 42,
            magic: 0,
        },
        Job::WhiteMage => JobStats {
            power: 4,
            guard: 3,
            speed: 6,
            health: 26,
            magic: 18,
        },
        Job::BlackMage => JobStats {
            power: 4,
            guard: 2,
            speed: 7,
            health: 24,
            magic: 20,
        },
        Job::Thief => JobStats {
            power: 8,
            guard: 4,
            speed: 11,
            health: 32,
            magic: 4,
        },
        Job::Monk => JobStats {
            power: 13,
            guard: 4,
            speed: 8,
            health: 38,
            magic: 0,
        },
    }
}

pub struct MonsterStats {
    pub tile: u32,
    pub health: i32,
    pub power: i32,
    pub guard: i32,
    pub speed: i32,
    pub gold: u32,
    pub experience: u32,
}

pub const fn monster_stats(kind: MonsterKind) -> MonsterStats {
    match kind {
        MonsterKind::Slime => MonsterStats {
            tile: theme::SLIME_TILE,
            health: 12,
            power: 5,
            guard: 1,
            speed: 3,
            gold: 6,
            experience: 4,
        },
        MonsterKind::Wolf => MonsterStats {
            tile: theme::WOLF_TILE,
            health: 20,
            power: 8,
            guard: 2,
            speed: 9,
            gold: 12,
            experience: 7,
        },
        MonsterKind::Imp => MonsterStats {
            tile: theme::IMP_TILE,
            health: 24,
            power: 9,
            guard: 3,
            speed: 7,
            gold: 16,
            experience: 10,
        },
        MonsterKind::Bat => MonsterStats {
            tile: theme::BAT_TILE,
            health: 16,
            power: 7,
            guard: 1,
            speed: 12,
            gold: 11,
            experience: 8,
        },
        MonsterKind::Spider => MonsterStats {
            tile: theme::SPIDER_TILE,
            health: 28,
            power: 10,
            guard: 4,
            speed: 6,
            gold: 20,
            experience: 13,
        },
        MonsterKind::Ghost => MonsterStats {
            tile: theme::GHOST_TILE,
            health: 22,
            power: 12,
            guard: 2,
            speed: 8,
            gold: 24,
            experience: 15,
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Swing,
    Interact,
    Accept,
    Pause,
    CommandOne,
    CommandTwo,
    CommandThree,
    CommandFour,
}

impl Ask {
    pub const COMMANDS: [Ask; 4] = [
        Ask::CommandOne,
        Ask::CommandTwo,
        Ask::CommandThree,
        Ask::CommandFour,
    ];
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Swing,
            Self::Interact,
            Self::Accept,
            Self::Pause,
            Self::CommandOne,
            Self::CommandTwo,
            Self::CommandThree,
            Self::CommandFour,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Swing => "swing",
            Self::Interact => "interact",
            Self::Accept => "accept",
            Self::Pause => "pause",
            Self::CommandOne => "command one",
            Self::CommandTwo => "command two",
            Self::CommandThree => "command three",
            Self::CommandFour => "command four",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_input::Binding;
        use nightshade_platform::KeyCode;
        match self {
            Self::Walk => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Keys {
                    up: KeyCode::ArrowUp,
                    down: KeyCode::ArrowDown,
                    left: KeyCode::ArrowLeft,
                    right: KeyCode::ArrowRight,
                },
            ],
            Self::Swing => &[Binding::Key(KeyCode::Space), Binding::Key(KeyCode::KeyJ)],
            Self::Interact => &[Binding::Key(KeyCode::KeyE), Binding::Key(KeyCode::Enter)],
            Self::Accept => &[Binding::Key(KeyCode::Enter)],
            Self::Pause => &[Binding::Key(KeyCode::Escape)],
            Self::CommandOne => &[Binding::Key(KeyCode::Digit1)],
            Self::CommandTwo => &[Binding::Key(KeyCode::Digit2)],
            Self::CommandThree => &[Binding::Key(KeyCode::Digit3)],
            Self::CommandFour => &[Binding::Key(KeyCode::Digit4)],
        }
    }
}
