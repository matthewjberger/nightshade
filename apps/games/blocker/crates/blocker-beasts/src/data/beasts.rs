use blocker_screen::data::names::same_name;

pub enum Form {
    Cube,
    Figure { look: usize, fit: f32 },
}

pub struct BeastKind {
    pub name: &'static str,
    pub health: f32,
    pub damage: f32,
    pub speed: f32,
    pub aggro: f32,
    pub renown: u32,
    pub color: [f32; 4],
    pub snout: [f32; 4],
    pub scale: f32,
    pub head: f32,
    pub weight: u32,
    pub form: Form,
}

pub const BEAST_KINDS: usize = 7;

pub const ZOMBIE: usize = beast_named("Barrow Zombie");

pub const WIGHT: usize = beast_named("Barrow Wight");

pub const fn beast_named(name: &str) -> usize {
    let mut index = 0;
    while index < BEASTS.len() {
        if same_name(BEASTS[index].name, name) {
            return index;
        }
        index += 1;
    }
    panic!("the beasts table holds no kind by that name")
}

pub const BEASTS: [BeastKind; BEAST_KINDS] = [
    beast("Boar", 30.0, 5.0, 4.2, 0.0, 8)
        .sized(1.0, 0.6, 5)
        .painted([0.36, 0.24, 0.16, 1.0], [0.2, 0.14, 0.1, 1.0]),
    beast("Wolf", 26.0, 7.0, 5.2, 0.0, 14)
        .sized(0.9, 0.6, 3)
        .painted([0.45, 0.45, 0.5, 1.0], [0.25, 0.25, 0.28, 1.0]),
    beast("Deer", 18.0, 2.0, 5.6, 0.0, 6)
        .sized(1.0, 0.6, 4)
        .painted([0.6, 0.42, 0.26, 1.0], [0.3, 0.2, 0.12, 1.0]),
    beast("Bear", 60.0, 12.0, 3.8, 0.0, 34)
        .sized(1.5, 0.9, 2)
        .painted([0.3, 0.2, 0.13, 1.0], [0.18, 0.12, 0.08, 1.0]),
    beast("Direwolf", 42.0, 10.0, 6.0, 0.0, 26)
        .sized(1.15, 0.7, 2)
        .painted([0.28, 0.28, 0.32, 1.0], [0.14, 0.14, 0.17, 1.0]),
    beast("Barrow Zombie", 34.0, 8.0, 2.7, 1.0, 18)
        .sized(1.0, 1.2, 0)
        .painted([0.24, 0.42, 0.3, 1.0], [0.16, 0.28, 0.2, 1.0])
        .figured(blocker_figure::data::characters::ROTTEN_LOOK, 0.66),
    beast("Barrow Wight", 110.0, 17.0, 3.3, 1.0, 70)
        .sized(1.25, 1.55, 0)
        .painted([0.18, 0.3, 0.26, 1.0], [0.1, 0.18, 0.16, 1.0])
        .figured(blocker_figure::data::characters::ROTTEN_LOOK, 0.85),
];

const fn beast(
    name: &'static str,
    health: f32,
    damage: f32,
    speed: f32,
    aggro: f32,
    renown: u32,
) -> BeastKind {
    BeastKind {
        name,
        health,
        damage,
        speed,
        aggro,
        renown,
        color: [0.0, 0.0, 0.0, 0.0],
        snout: [0.0, 0.0, 0.0, 0.0],
        scale: 1.0,
        head: 0.0,
        weight: 0,
        form: Form::Cube,
    }
}

impl BeastKind {
    const fn sized(mut self, scale: f32, head: f32, weight: u32) -> Self {
        self.scale = scale;
        self.head = head;
        self.weight = weight;
        self
    }

    const fn painted(mut self, color: [f32; 4], snout: [f32; 4]) -> Self {
        self.color = color;
        self.snout = snout;
        self
    }

    const fn figured(mut self, look: usize, fit: f32) -> Self {
        self.form = Form::Figure { look, fit };
        self
    }
}

const _: () = assert!(BEASTS.len() == BEAST_KINDS);
const _: () = assert!(beast_named("Boar") == blocker_quests::data::quests::BOAR_KIND);
const _: () = assert!(beast_named("Wolf") == blocker_quests::data::quests::WOLF_KIND);
const _: () = assert!(beast_named("Deer") == blocker_quests::data::quests::DEER_KIND);
const _: () = assert!(beast_named("Bear") == blocker_quests::data::quests::BEAR_KIND);
const _: () = assert!(beast_named("Direwolf") == blocker_quests::data::quests::DIREWOLF_KIND);
const _: () = assert!(beast_named("Barrow Zombie") == blocker_quests::data::quests::ZOMBIE_KIND);
const _: () = assert!(beast_named("Barrow Wight") == blocker_quests::data::quests::WIGHT_KIND);

pub const THREAT_OUTLINE_COLOR: [f32; 4] = [1.0, 0.15, 0.12, 1.0];

pub const NEUTRAL_OUTLINE_COLOR: [f32; 4] = [1.0, 0.85, 0.2, 1.0];
