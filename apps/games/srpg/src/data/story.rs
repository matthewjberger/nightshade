use crate::data::{Class, Facing, Recruit, Site, StructureKind, Team, recruit, site};
use crate::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mood {
    #[default]
    Steady,
    Bright,
    Heavy,
    Precise,
    Eager,
    Quiet,
    Sharp,
}

impl Mood {
    pub fn bob(self) -> (f32, f32, f32) {
        match self {
            Mood::Steady => (0.045, 5.0, 0.03),
            Mood::Bright => (0.11, 9.5, 0.10),
            Mood::Heavy => (0.03, 3.0, 0.02),
            Mood::Precise => (0.05, 7.0, 0.14),
            Mood::Eager => (0.14, 12.0, 0.06),
            Mood::Quiet => (0.025, 4.0, 0.05),
            Mood::Sharp => (0.08, 10.5, 0.16),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Upgrade {
    #[default]
    Toughened,
    Marksman,
    Charged,
    Warded,
    Swift,
    Surefoot,
    Hardened,
    Focused,
}

pub const UPGRADES: [Upgrade; 8] = [
    Upgrade::Toughened,
    Upgrade::Marksman,
    Upgrade::Charged,
    Upgrade::Warded,
    Upgrade::Swift,
    Upgrade::Surefoot,
    Upgrade::Hardened,
    Upgrade::Focused,
];

impl Upgrade {
    pub fn label(self) -> &'static str {
        match self {
            Upgrade::Toughened => "Toughened   more health",
            Upgrade::Marksman => "Marksman   more attack",
            Upgrade::Charged => "Charged   more magic",
            Upgrade::Warded => "Warded   more resist",
            Upgrade::Swift => "Swift   more speed",
            Upgrade::Surefoot => "Surefoot   more evasion",
            Upgrade::Hardened => "Hardened   more defense",
            Upgrade::Focused => "Focused   deeper reserves",
        }
    }

    pub fn line(self) -> &'static str {
        match self {
            Upgrade::Toughened => "takes the heavier pack without a word.",
            Upgrade::Marksman => "spends the evening on the range.",
            Upgrade::Charged => "strips and rebuilds the firing set.",
            Upgrade::Warded => "studies the burn charts until late.",
            Upgrade::Swift => "runs the perimeter twice before dawn.",
            Upgrade::Surefoot => "learns the ground and where it gives.",
            Upgrade::Hardened => "beats the dents out of the old plate.",
            Upgrade::Focused => "sleeps early and wakes clear.",
        }
    }

    pub fn growth(self) -> Growth {
        match self {
            Upgrade::Toughened => Growth {
                hp: 14,
                ..Growth::NONE
            },
            Upgrade::Marksman => Growth {
                attack: 3,
                ..Growth::NONE
            },
            Upgrade::Charged => Growth {
                magic: 3,
                ..Growth::NONE
            },
            Upgrade::Warded => Growth {
                resist: 3,
                ..Growth::NONE
            },
            Upgrade::Swift => Growth {
                speed: 2,
                ..Growth::NONE
            },
            Upgrade::Surefoot => Growth {
                evasion: 4,
                ..Growth::NONE
            },
            Upgrade::Hardened => Growth {
                defense: 3,
                ..Growth::NONE
            },
            Upgrade::Focused => Growth {
                mp: 8,
                ..Growth::NONE
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Taste {
    #[default]
    Sweets,
    Tools,
    Books,
    Charts,
    Records,
}

impl Taste {
    pub fn name(self) -> &'static str {
        match self {
            Taste::Sweets => "Sweets",
            Taste::Tools => "Tools",
            Taste::Books => "Books",
            Taste::Charts => "Charts",
            Taste::Records => "Records",
        }
    }

    pub fn gift(self) -> &'static str {
        match self {
            Taste::Sweets => "a tin of honey cakes",
            Taste::Tools => "a boxed spanner set",
            Taste::Books => "a worn field manual",
            Taste::Charts => "a river survey chart",
            Taste::Records => "a scratched dance record",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Bond {
    #[default]
    Cold,
    Warm,
    Close,
    Trusted,
    Sworn,
}

pub const BOND_STEPS: [u32; 4] = [20, 50, 90, 140];

impl Bond {
    pub fn of(points: u32) -> Bond {
        match BOND_STEPS.iter().filter(|step| points >= **step).count() {
            0 => Bond::Cold,
            1 => Bond::Warm,
            2 => Bond::Close,
            3 => Bond::Trusted,
            _ => Bond::Sworn,
        }
    }

    pub fn rank(self) -> u32 {
        match self {
            Bond::Cold => 0,
            Bond::Warm => 1,
            Bond::Close => 2,
            Bond::Trusted => 3,
            Bond::Sworn => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Bond::Cold => "New",
            Bond::Warm => "Warm",
            Bond::Close => "Close",
            Bond::Trusted => "Trusted",
            Bond::Sworn => "Sworn",
        }
    }

    pub fn grade(self) -> &'static str {
        match self {
            Bond::Cold => "-",
            Bond::Warm => "C",
            Bond::Close => "B",
            Bond::Trusted => "A",
            Bond::Sworn => "S",
        }
    }

    pub fn next_step(self) -> Option<u32> {
        BOND_STEPS.get(self.rank() as usize).copied()
    }

    pub fn share(self, points: u32) -> f32 {
        let rank = self.rank() as usize;
        let floor = if rank == 0 { 0 } else { BOND_STEPS[rank - 1] };
        match self.next_step() {
            Some(step) if step > floor => {
                ((points - floor) as f32 / (step - floor) as f32).clamp(0.0, 1.0)
            }
            _ => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Growth {
    pub hp: i32,
    pub mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub resist: i32,
    pub speed: i32,
    pub evasion: i32,
}

impl Growth {
    pub const NONE: Growth = Growth {
        hp: 0,
        mp: 0,
        attack: 0,
        defense: 0,
        magic: 0,
        resist: 0,
        speed: 0,
        evasion: 0,
    };
}

impl std::ops::Add for Growth {
    type Output = Growth;

    fn add(self, other: Growth) -> Growth {
        Growth {
            hp: self.hp + other.hp,
            mp: self.mp + other.mp,
            attack: self.attack + other.attack,
            defense: self.defense + other.defense,
            magic: self.magic + other.magic,
            resist: self.resist + other.resist,
            speed: self.speed + other.speed,
            evasion: self.evasion + other.evasion,
        }
    }
}

pub struct ArcStage {
    pub title: &'static str,
    pub line: &'static str,
    pub boon: Growth,
}

const fn stage(title: &'static str, line: &'static str, boon: Growth) -> ArcStage {
    ArcStage { title, line, boon }
}

const fn boon(hp: i32, attack: i32, defense: i32, magic: i32, speed: i32) -> Growth {
    Growth {
        hp,
        attack,
        defense,
        magic,
        speed,
        ..Growth::NONE
    }
}

pub fn level_of(experience: u32) -> u32 {
    (1 + experience / theme::EXPERIENCE_PER_LEVEL).min(theme::MAX_LEVEL)
}

pub fn level_share(experience: u32) -> f32 {
    if level_of(experience) >= theme::MAX_LEVEL {
        return 1.0;
    }
    (experience % theme::EXPERIENCE_PER_LEVEL) as f32 / theme::EXPERIENCE_PER_LEVEL as f32
}

pub fn growth_of(level: u32) -> Growth {
    let steps = level.saturating_sub(1) as i32;
    Growth {
        hp: steps * 6,
        mp: steps,
        attack: steps / 2,
        defense: steps / 3,
        magic: steps / 2,
        resist: steps / 3,
        speed: steps / 4,
        evasion: steps / 3,
    }
}

pub fn morale_edge(morale: i32) -> i32 {
    ((morale - 50) / 25).clamp(-2, 2)
}

pub fn morale_name(morale: i32) -> &'static str {
    match morale {
        value if value >= 85 => "Eager",
        value if value >= 60 => "Steady",
        value if value >= 35 => "Tired",
        _ => "Shaken",
    }
}

pub fn rapport_growth(bond: Bond) -> Growth {
    let rank = bond.rank() as i32;
    Growth {
        hp: rank * 5,
        mp: rank,
        attack: rank,
        defense: rank / 2,
        magic: rank,
        resist: rank / 2,
        speed: rank / 2,
        evasion: rank,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    Talk,
    Train,
    Advance,
    Gift,
    Spar,
    Muster,
}

pub const ACTIVITIES: [Activity; 6] = [
    Activity::Talk,
    Activity::Train,
    Activity::Advance,
    Activity::Gift,
    Activity::Spar,
    Activity::Muster,
];

impl Activity {
    pub fn label(self) -> &'static str {
        match self {
            Activity::Talk => "Talk",
            Activity::Train => "Train",
            Activity::Advance => "Advance",
            Activity::Gift => "Give a gift",
            Activity::Spar => "Spar",
            Activity::Muster => "Muster",
        }
    }

    pub fn actions(self) -> u32 {
        match self {
            Activity::Muster | Activity::Advance => 0,
            _ => 1,
        }
    }

    pub fn supplies(self) -> i32 {
        match self {
            Activity::Gift => theme::GIFT_COST,
            _ => 0,
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Activity::Talk => "An hour of talk lifts the bond.",
            Activity::Train => "Drill for experience.",
            Activity::Advance => "Spend a mark of training.",
            Activity::Gift => "Buy them what they like.",
            Activity::Spar => "Two train and both learn.",
            Activity::Muster => "Put them on the roster.",
        }
    }
}

pub type Scene = &'static [(&'static str, &'static str)];

pub struct Companion {
    pub name: &'static str,
    pub title: &'static str,
    pub class: Class,
    pub taste: Taste,
    pub mood: Mood,
    pub facing: Facing,
    pub greetings: [&'static str; 3],
    pub lines: [&'static str; 4],
    pub arc: &'static str,
    pub stages: [ArcStage; 4],
}

pub const COMPANIONS: [Companion; 9] = [
    Companion {
        name: "Sgt. Beoric",
        title: "the old hand",
        class: Class::Trooper,
        taste: Taste::Records,
        mood: Mood::Steady,
        facing: Facing::North,
        greetings: [
            "Commander. The watch is set and the kettle is on.",
            "Sit if you like. My knee says rain and my knee is never wrong.",
            "You look like you have been thinking again. Out with it.",
        ],
        lines: [
            "Beoric tells you the squad is green, and green burns fast. He says he will keep them wet.",
            "Beoric tells you how he lost his first squad at the ford.",
            "Beoric admits he sleeps better since you took command.",
            "Beoric will hold the line for you until it breaks him.",
        ],
        arc: "The Squad He Lost",
        stages: [
            stage(
                "A Name He Will Not Say",
                "Beoric writes a name on the back of his tags and puts them away.",
                boon(6, 0, 1, 0, 0),
            ),
            stage(
                "The Ford",
                "Beoric walks you to the ford and shows you where the line broke.",
                boon(0, 0, 2, 0, 0),
            ),
            stage(
                "The Letter",
                "Beoric asks you to sign a letter to a widow he has never met.",
                boon(8, 0, 1, 0, 0),
            ),
            stage(
                "The Old Hand",
                "Beoric hands you his whistle. The squad is yours, and it will hold.",
                boon(10, 2, 2, 0, 0),
            ),
        ],
    },
    Companion {
        name: "Tessa",
        title: "the quiet shot",
        class: Class::Grenadier,
        taste: Taste::Books,
        mood: Mood::Bright,
        facing: Facing::North,
        greetings: [
            "Oh. Hello. I did not hear you coming, which is embarrassing for a scout.",
            "I saved you the good chair. Do not tell Dash.",
            "I was hoping you would come by, actually.",
        ],
        lines: [
            "Tessa counts her charges twice, then asks how you are holding up.",
            "Tessa asks what the far side of the river looks like.",
            "Tessa reads you a page from the book she carries.",
            "Tessa says she stays because you always come back for her.",
        ],
        arc: "The Far Bank",
        stages: [
            stage(
                "Counting",
                "Tessa lets you watch her count her charges, and finishes the count aloud.",
                boon(0, 0, 0, 0, 2),
            ),
            stage(
                "The Book",
                "Tessa lends you the book. Half the pages are maps of places she has never been.",
                boon(0, 2, 0, 0, 0),
            ),
            stage(
                "The Question",
                "Tessa asks whether the war ends before her reading list does.",
                boon(0, 2, 0, 0, 1),
            ),
            stage(
                "The Far Bank",
                "Tessa says she will cross any river with you and asks nothing back.",
                boon(6, 3, 0, 0, 3),
            ),
        ],
    },
    Companion {
        name: "Ironhide",
        title: "the tank crew",
        class: Class::Tank,
        taste: Taste::Tools,
        mood: Mood::Heavy,
        facing: Facing::North,
        greetings: [
            "Commander. Mind the oil, I have not cleaned it up.",
            "Come and hold this, would you. Two hands is one short.",
            "You are always welcome under here. It is quiet.",
        ],
        lines: [
            "Ironhide slides out from under the hull and asks whether you have eaten.",
            "Ironhide shows you the dent a shell left last spring.",
            "Ironhide lets you sit in the driver's seat.",
            "Ironhide has painted your mark on the turret.",
        ],
        arc: "The Hull",
        stages: [
            stage(
                "Under the Hull",
                "Ironhide hands you a wrench and does not say which bolt.",
                boon(0, 0, 2, 0, 0),
            ),
            stage(
                "The Dent",
                "Ironhide shows you the shell dent and names the gunner who made it.",
                boon(10, 0, 0, 0, 0),
            ),
            stage(
                "The Driver's Seat",
                "Ironhide teaches you the gears and lets you stall it twice.",
                boon(6, 0, 2, 0, 0),
            ),
            stage(
                "Your Mark",
                "Ironhide paints your mark on the turret and calls the tank by your name.",
                boon(14, 0, 3, 0, 0),
            ),
        ],
    },
    Companion {
        name: "Mireille",
        title: "the quartermaster",
        class: Class::Supply,
        taste: Taste::Sweets,
        mood: Mood::Precise,
        facing: Facing::North,
        greetings: [
            "There you are. I have three things and none of them are urgent.",
            "Coffee? I will pretend I did not save it for you.",
            "Sit down before you fall down, Commander.",
        ],
        lines: [
            "Mireille reads out the supply ledger, then closes it and asks about your day.",
            "Mireille saves you the last of the coffee.",
            "Mireille tells you which of the squad is not sleeping.",
            "Mireille says the camp is a home now, and you made it one.",
        ],
        arc: "The Ledger",
        stages: [
            stage(
                "The Shortfall",
                "Mireille reads the shortfall aloud and waits to see what you do.",
                boon(0, 0, 0, 2, 0),
            ),
            stage(
                "The Last of the Coffee",
                "Mireille keeps a cup back for you and pretends it was an accident.",
                boon(6, 0, 0, 0, 0),
            ),
            stage(
                "Who Is Not Sleeping",
                "Mireille tells you which of the squad walks the wall at night.",
                boon(0, 0, 0, 2, 1),
            ),
            stage(
                "A Home Now",
                "Mireille closes the ledger and says the camp balances at last.",
                boon(8, 0, 0, 3, 0),
            ),
        ],
    },
    Companion {
        name: "Dash",
        title: "the scout",
        class: Class::Recon,
        taste: Taste::Charts,
        mood: Mood::Eager,
        facing: Facing::North,
        greetings: [
            "Hey. Walk with me, I have been sitting too long.",
            "You should see the ridge at this hour. I will take you.",
            "I found something out east. Ask me about it.",
        ],
        lines: [
            "Dash walks the whole camp with you and never once runs ahead.",
            "Dash draws the enemy road in the dirt for you.",
            "Dash asks you to ride out with her at first light.",
            "Dash will scout any road you point at.",
        ],
        arc: "The Long Road",
        stages: [
            stage(
                "The Ridge",
                "Dash is over the ridge and back before you finish the order.",
                boon(0, 0, 0, 0, 3),
            ),
            stage(
                "The Enemy Road",
                "Dash draws the enemy road in the dirt and rubs it out again.",
                boon(0, 1, 0, 0, 2),
            ),
            stage(
                "First Light",
                "Dash rides out with you and says nothing the whole way.",
                boon(6, 0, 0, 0, 2),
            ),
            stage(
                "Any Road",
                "Dash will take any road you point at, and means the last one too.",
                boon(0, 2, 0, 0, 4),
            ),
        ],
    },
    Companion {
        name: "Longarm",
        title: "the gunner",
        class: Class::Artillery,
        taste: Taste::Charts,
        mood: Mood::Quiet,
        facing: Facing::North,
        greetings: [
            "Commander. Pull up a crate, the tables can wait.",
            "I was hoping for company. The numbers stop making sense on their own.",
            "Careful of the casings. I know where every one of them came from.",
        ],
        lines: [
            "Longarm shows you the range tables and admits he does not trust them.",
            "Longarm explains how he reads the wind off the water.",
            "Longarm keeps a shell casing from every battle you won.",
            "Longarm will drop a shell wherever you point.",
        ],
        arc: "The Range Tables",
        stages: [
            stage(
                "The Frown",
                "Longarm shows you the range table that got a crew killed.",
                boon(0, 2, 0, 0, 0),
            ),
            stage(
                "Reading the Wind",
                "Longarm teaches you to read the wind off the water.",
                boon(0, 2, 0, 1, 0),
            ),
            stage(
                "The Casings",
                "Longarm lines up a casing from every battle you won.",
                boon(0, 3, 0, 0, 0),
            ),
            stage(
                "Wherever You Point",
                "Longarm gives you the last casing and says the next one is yours.",
                boon(6, 4, 0, 0, 0),
            ),
        ],
    },
    Companion {
        name: "Kestrel",
        title: "the pilot",
        class: Class::Helicopter,
        taste: Taste::Records,
        mood: Mood::Sharp,
        facing: Facing::North,
        greetings: [
            "Commander. Sit, the record has two songs left.",
            "You are the only one who does not talk over the music.",
            "I was going to come and find you, so you saved me a walk.",
        ],
        lines: [
            "Kestrel turns the record down and asks what you think of the valley.",
            "Kestrel says the air over the hill is rough at dusk.",
            "Kestrel takes you up over the valley at dawn.",
            "Kestrel says she flies for this squad and no other.",
        ],
        arc: "The Record",
        stages: [
            stage(
                "The Field Set",
                "Kestrel turns the record up rather than answer you.",
                boon(0, 0, 0, 0, 2),
            ),
            stage(
                "Rough Air",
                "Kestrel tells you the air over the hill kills pilots at dusk.",
                boon(0, 0, 1, 0, 2),
            ),
            stage(
                "Dawn Over the Valley",
                "Kestrel takes you up and cuts the engine for a moment.",
                boon(6, 0, 0, 0, 3),
            ),
            stage(
                "This Squad",
                "Kestrel gives you the record. She flies for this squad and no other.",
                boon(0, 2, 0, 0, 4),
            ),
        ],
    },
    Companion {
        name: "Atlas",
        title: "the walker pilot",
        class: Class::Walker,
        taste: Taste::Tools,
        mood: Mood::Heavy,
        facing: Facing::North,
        greetings: [
            "Commander. She is running warm but she is running.",
            "Come up, the view from the cockpit is worth the ladder.",
            "I was hoping you would ask about her.",
        ],
        lines: [
            "Atlas oils the leg joints and tells you the walker has a name he has not said yet.",
            "Atlas tells you the walker was his father's machine.",
            "Atlas asks you to name the walker.",
            "Atlas says the machine and the squad are one thing to him.",
        ],
        arc: "His Father's Machine",
        stages: [
            stage(
                "The Hours",
                "Atlas counts the walker's hours aloud like a prayer.",
                boon(0, 0, 2, 0, 0),
            ),
            stage(
                "His Father",
                "Atlas tells you whose machine it was before it was his.",
                boon(10, 0, 0, 0, 0),
            ),
            stage(
                "The Name",
                "Atlas asks you to name the walker and writes what you say on the hull.",
                boon(0, 2, 2, 0, 0),
            ),
            stage(
                "One Thing",
                "Atlas says the machine and the squad are one thing to him now.",
                boon(12, 0, 3, 0, 0),
            ),
        ],
    },
    Companion {
        name: "Tidewater",
        title: "the boat crew",
        class: Class::Gunboat,
        taste: Taste::Sweets,
        mood: Mood::Bright,
        facing: Facing::East,
        greetings: [
            "Commander! Mind the rope. Come aboard, come aboard.",
            "The river is kind today. Sit and I will tell you why.",
            "You are just in time. I have a story and nobody to tell it to.",
        ],
        lines: [
            "Tidewater shouts a greeting from the mooring and will not let you leave hungry.",
            "Tidewater tells you where the river runs shallow.",
            "Tidewater cooks for the camp and will not say where the fish came from.",
            "Tidewater says the river is yours as long as she floats.",
        ],
        arc: "The Shallow Water",
        stages: [
            stage(
                "The Mooring",
                "Tidewater shouts you down from the mooring and makes you laugh.",
                boon(6, 0, 0, 0, 0),
            ),
            stage(
                "Where It Runs Shallow",
                "Tidewater maps the shallows on the back of a crate lid.",
                boon(0, 0, 0, 0, 2),
            ),
            stage(
                "The Fish",
                "Tidewater cooks for the camp and will not say where the fish came from.",
                boon(8, 0, 0, 1, 0),
            ),
            stage(
                "As Long As She Floats",
                "Tidewater says the river is yours as long as the boat floats.",
                boon(10, 2, 0, 0, 2),
            ),
        ],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Place {
    #[default]
    Camp,
    Field,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RoomKind {
    #[default]
    Quarters,
    Mess,
    Yard,
    Workshop,
    Command,
    Dock,
    Gate,
}

impl RoomKind {
    pub fn call(self) -> &'static str {
        match self {
            RoomKind::Quarters => "Turn in for the night",
            RoomKind::Mess => "Call everyone to supper",
            RoomKind::Yard => "Run a squad drill",
            RoomKind::Workshop => "Patch up the wounded",
            RoomKind::Command => "Open the muster",
            RoomKind::Dock => "Send the boat for stores",
            RoomKind::Gate => "March out on the mission",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            RoomKind::Quarters => "the barracks",
            RoomKind::Mess => "the mess hall",
            RoomKind::Yard => "the training yard",
            RoomKind::Workshop => "the workshop",
            RoomKind::Command => "the great hall",
            RoomKind::Dock => "the boathouse",
            RoomKind::Gate => "the camp gate",
        }
    }

    pub fn build(self) -> Build {
        match self {
            RoomKind::Quarters => build(
                theme::PLANK_MATERIAL,
                theme::RED_ROOF_MATERIAL,
                theme::WOOD_DOOR_MATERIAL,
                theme::WOOD_WINDOW_MATERIAL,
                3,
                2,
                2,
            ),
            RoomKind::Mess => build(
                theme::TOWN_STONE_MATERIAL,
                theme::BLUE_ROOF_MATERIAL,
                theme::GREY_DOOR_MATERIAL,
                theme::GREY_WINDOW_MATERIAL,
                3,
                2,
                2,
            ),
            RoomKind::Yard => build(
                theme::PLANK_MATERIAL,
                theme::BLUE_ROOF_MATERIAL,
                theme::WOOD_DOOR_MATERIAL,
                theme::WOOD_WINDOW_MATERIAL,
                3,
                2,
                2,
            ),
            RoomKind::Workshop => build(
                theme::TOWN_STONE_MATERIAL,
                theme::RED_ROOF_MATERIAL,
                theme::GREY_DOOR_MATERIAL,
                theme::GREY_WINDOW_MATERIAL,
                3,
                2,
                2,
            ),
            RoomKind::Command => build(
                theme::KEEP_MATERIAL,
                theme::BLUE_ROOF_MATERIAL,
                theme::KEEP_DOOR_MATERIAL,
                theme::GREY_WINDOW_MATERIAL,
                3,
                2,
                3,
            ),
            RoomKind::Dock => build(
                theme::PLANK_MATERIAL,
                theme::BLUE_ROOF_MATERIAL,
                theme::WOOD_DOOR_MATERIAL,
                theme::WOOD_WINDOW_MATERIAL,
                3,
                2,
                2,
            ),
            RoomKind::Gate => build(
                theme::KEEP_MATERIAL,
                theme::BLUE_ROOF_MATERIAL,
                theme::KEEP_DOOR_MATERIAL,
                theme::GREY_WINDOW_MATERIAL,
                3,
                1,
                3,
            ),
        }
    }

    pub fn actions(self) -> u32 {
        match self {
            RoomKind::Quarters | RoomKind::Command | RoomKind::Gate => 0,
            _ => 1,
        }
    }

    pub fn supplies(self) -> i32 {
        match self {
            RoomKind::Workshop => theme::REPAIR_COST,
            _ => 0,
        }
    }
}

pub struct Standing {
    pub kind: RoomKind,
    pub column: u32,
    pub row: u32,
}

const fn standing(kind: RoomKind, column: u32, row: u32) -> Standing {
    Standing { kind, column, row }
}

pub const CAMP_ROOMS: [Standing; 7] = [
    standing(RoomKind::Gate, 15, 9),
    standing(RoomKind::Command, 9, 2),
    standing(RoomKind::Quarters, 4, 6),
    standing(RoomKind::Mess, 13, 6),
    standing(RoomKind::Yard, 4, 12),
    standing(RoomKind::Workshop, 13, 12),
    standing(RoomKind::Dock, 9, 14),
];

pub struct Build {
    pub wall: &'static str,
    pub roof: &'static str,
    pub door: &'static str,
    pub window: &'static str,
    pub wide: u32,
    pub deep: u32,
    pub height: u32,
}

const fn build(
    wall: &'static str,
    roof: &'static str,
    door: &'static str,
    window: &'static str,
    wide: u32,
    deep: u32,
    height: u32,
) -> Build {
    Build {
        wall,
        roof,
        door,
        window,
        wide,
        deep,
        height,
    }
}

pub fn built(kind: StructureKind) -> Build {
    match kind {
        StructureKind::Headquarters => build(
            theme::KEEP_MATERIAL,
            theme::BLUE_ROOF_MATERIAL,
            theme::KEEP_DOOR_MATERIAL,
            theme::GREY_WINDOW_MATERIAL,
            3,
            2,
            3,
        ),
        StructureKind::Factory => build(
            theme::TOWN_STONE_MATERIAL,
            theme::RED_ROOF_MATERIAL,
            theme::GREY_DOOR_MATERIAL,
            theme::GREY_WINDOW_MATERIAL,
            3,
            2,
            2,
        ),
        StructureKind::Depot => build(
            theme::PLANK_MATERIAL,
            theme::RED_ROOF_MATERIAL,
            theme::WOOD_DOOR_MATERIAL,
            theme::WOOD_WINDOW_MATERIAL,
            3,
            2,
            2,
        ),
        StructureKind::Tower => build(
            theme::KEEP_MATERIAL,
            theme::BLUE_ROOF_MATERIAL,
            theme::KEEP_DOOR_MATERIAL,
            theme::GREY_WINDOW_MATERIAL,
            1,
            1,
            4,
        ),
    }
}

pub const CAMP_TENTS: [(u32, u32); 6] = [(3, 4), (4, 4), (5, 4), (12, 4), (13, 4), (14, 4)];

pub struct Prop {
    pub tile: u32,
    pub column: u32,
    pub row: u32,
    pub scale: f32,
}

const fn prop(tile: u32, column: u32, row: u32, scale: f32) -> Prop {
    Prop {
        tile,
        column,
        row,
        scale,
    }
}

pub const CAMP_PROPS: [Prop; 49] = [
    prop(
        theme::SIGNBOARD,
        theme::GATE_SIGN.0,
        theme::GATE_SIGN.1,
        theme::TOOL_SCALE,
    ),
    prop(theme::FENCE_POST, 1, 5, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 1, 6, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 1, 7, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 1, 8, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 1, 10, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 1, 11, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 1, 12, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 16, 5, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 16, 6, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 16, 7, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 16, 8, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 16, 10, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 16, 11, theme::FENCE_SCALE),
    prop(theme::FENCE_POST, 16, 12, theme::FENCE_SCALE),
    prop(theme::FENCE_RAIL, 6, 3, theme::FENCE_SCALE),
    prop(theme::FENCE_RAIL, 7, 3, theme::FENCE_SCALE),
    prop(theme::FENCE_RAIL, 11, 3, theme::FENCE_SCALE),
    prop(theme::FENCE_RAIL, 12, 3, theme::FENCE_SCALE),
    prop(theme::SIGNBOARD, 8, 3, theme::TOOL_SCALE),
    prop(theme::CHEST, 3, 7, theme::CARGO_SCALE),
    prop(theme::BARREL, 5, 7, theme::CARGO_SCALE),
    prop(theme::FLOWERS, 2, 5, theme::PLANT_SCALE),
    prop(theme::BARREL, 12, 7, theme::CARGO_SCALE),
    prop(theme::CRATE, 14, 7, theme::CARGO_SCALE),
    prop(theme::BREAD, 13, 7, theme::CARGO_SCALE),
    prop(theme::HAMMER, 12, 13, theme::TOOL_SCALE),
    prop(theme::PICKAXE, 14, 13, theme::TOOL_SCALE),
    prop(theme::LADDER, 15, 11, theme::TOOL_SCALE),
    prop(theme::TARGET, 3, 13, theme::TOOL_SCALE),
    prop(theme::PITCHFORK, 5, 13, theme::TOOL_SCALE),
    prop(theme::OPEN_CRATE, 2, 11, theme::CARGO_SCALE),
    prop(theme::CRATE, 7, 14, theme::CARGO_SCALE),
    prop(theme::BARREL, 11, 14, theme::CARGO_SCALE),
    prop(theme::OPEN_CRATE, 7, 15, theme::CARGO_SCALE),
    prop(theme::CRATE, 11, 15, theme::CARGO_SCALE),
    prop(theme::PINE, 0, 4, theme::TREE_SCALE),
    prop(theme::PINE, 0, 8, theme::TREE_SCALE),
    prop(theme::MAPLE, 0, 12, theme::TREE_SCALE),
    prop(theme::MAPLE, 17, 4, theme::TREE_SCALE),
    prop(theme::PINE, 17, 8, theme::TREE_SCALE),
    prop(theme::PINE, 17, 12, theme::TREE_SCALE),
    prop(theme::BUSH, 2, 4, theme::PLANT_SCALE),
    prop(theme::BUSH, 15, 4, theme::PLANT_SCALE),
    prop(theme::SHRUB, 2, 13, theme::PLANT_SCALE),
    prop(theme::SHRUB, 15, 13, theme::PLANT_SCALE),
    prop(theme::MUSHROOMS, 0, 10, theme::PLANT_SCALE),
    prop(theme::SPRIG, 17, 6, theme::PLANT_SCALE),
    prop(theme::FLOWERS, 15, 5, theme::PLANT_SCALE),
];

pub const CAMP_SPOTS: [(u32, u32); 9] = [
    (7, 4),
    (5, 8),
    (12, 8),
    (11, 4),
    (6, 11),
    (12, 10),
    (7, 13),
    (11, 13),
    (9, 15),
];

pub const CAMP_START: (u32, u32) = (9, 12);

pub const CAMP_LANTERNS: [(u32, u32); 4] = [(7, 8), (11, 8), (7, 10), (11, 10)];

pub struct Page {
    pub heading: &'static str,
    pub body: [&'static str; 3],
}

pub const PROLOGUE: [Page; 4] = [
    Page {
        heading: "The Valley",
        body: [
            "The war moved north two winters ago and left the valley to itself.",
            "What it left behind were raiders, and a river road nobody guards.",
            "Last week they burned the mill at the ford and took the season's grain.",
        ],
    },
    Page {
        heading: "Your Commission",
        body: [
            "You are sent out with nine names on a page and one field camp.",
            "A sergeant who has buried one squad already. A pilot who answers to nobody.",
            "A tank crew, a scout, a gunner, a quartermaster, a boat, a walker, a quiet shot.",
        ],
    },
    Page {
        heading: "The Camp",
        body: [
            "Between missions the camp is yours. Each day buys three hours of your time.",
            "Talk with them, drill them, feed them, spar them, or spend supplies on a gift.",
            "A squad that knows you fights harder. Two who eat together cover each other.",
        ],
    },
    Page {
        heading: "First Light",
        body: [
            "Muster who marches, then take the road to the ford.",
            "Hold the high ground, watch your flanks, and bring them home.",
            "The valley is watching what this squad turns out to be.",
        ],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Objective {
    Rout,
    Seize,
    Survive(u32),
}

impl Objective {
    pub fn name(self) -> &'static str {
        match self {
            Objective::Rout => "Rout the enemy",
            Objective::Seize => "Seize the red headquarters",
            Objective::Survive(_) => "Hold the ground",
        }
    }

    pub fn detail(self) -> String {
        match self {
            Objective::Rout => "Every red unit must fall.".to_string(),
            Objective::Seize => "Stand a capturing unit on the red headquarters.".to_string(),
            Objective::Survive(turns) => format!("Hold out for {turns} turns."),
        }
    }
}

pub struct Chapter {
    pub name: &'static str,
    pub intro: &'static str,
    pub victory: &'static str,
    pub objective: Objective,
    pub days: u32,
    pub deploy: usize,
    pub reward: i32,
    pub joins: &'static [usize],
    pub brief: Scene,
    pub debrief: Scene,
    pub plan: &'static Plan,
}

pub struct Plan {
    pub ground: &'static [&'static str; 18],
    pub props: &'static [&'static str; 18],
    pub sites: &'static [Site],
    pub spots: &'static [(u32, u32)],
    pub squad: &'static [Recruit],
}

const FORD_GROUND: [&str; 18] = [
    "444455555554444444",
    "444455555554444444",
    "444444455444444444",
    "334444444444444433",
    "333344444444444333",
    "333333444443333333",
    "333333333333333333",
    "333333433333333333",
    "222222222222222222",
    "~~~~~~~333~~~~~~~~",
    "~~~~~~~333~~~~~~~~",
    "222222222222222222",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
];

const FORD_PROPS: [&str; 18] = [
    "........=.........",
    "..T.....=......T..",
    "........=.....T...",
    ".T...#..=.........",
    "........=......T..",
    "..T.....=..T......",
    "....B...=...B.....",
    "........=.........",
    "......B.==.B......",
    "........==........",
    "........==........",
    "......B.==.B......",
    "........=.........",
    "..T.....=.....T...",
    "........=.........",
    ".T..#...=...#.....",
    "........=.........",
    "..T.....=......T..",
];

const ROAD_GROUND: [&str; 18] = [
    "555554444444455555",
    "555544444444445555",
    "444444444444444444",
    "444444333334444444",
    "444443333333344444",
    "333333333333333333",
    "333333333333333333",
    "444333333333334444",
    "444333333333334444",
    "333333333333333333",
    "333333333333333333",
    "333344444444443333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
];

const ROAD_PROPS: [&str; 18] = [
    ".........=........",
    "....B....=....B...",
    ".........=........",
    "..#......=......#.",
    ".........=........",
    "..T......=......T.",
    ".........=........",
    "###......=......##",
    "###......=......##",
    ".........=........",
    "....B....=....B...",
    ".........=........",
    "..T......=......T.",
    ".........=........",
    "...#.....=.....#..",
    ".........=........",
    "..T......=......T.",
    ".........=........",
];

const RIVER_GROUND: [&str; 18] = [
    "444444444444444444",
    "444444444444444444",
    "444444444444444444",
    "333444444444444333",
    "333333333333333333",
    "333333333333333333",
    "222222222222222222",
    "~~~~~~~~33~~~~~~~~",
    "~~~~~~~~33~~~~~~~~",
    "~~~~~~~~33~~~~~~~~",
    "~~~~~~~~33~~~~~~~~",
    "222222222222222222",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
];

const RIVER_PROPS: [&str; 18] = [
    "........==........",
    "..T.....==.....T..",
    "........==........",
    "...#....==....#...",
    "........==........",
    "..T.....==.....T..",
    "......B.==.B......",
    "........==........",
    "........==........",
    "........==........",
    "........==........",
    "......B.==.B......",
    "........==........",
    "..T.....==.....T..",
    "........==........",
    "...#....==....#...",
    "........==........",
    "..T.....==.....T..",
];

const TOWER_GROUND: [&str; 18] = [
    "666666666666666666",
    "666666666666666666",
    "666666666666666666",
    "555555555555555555",
    "555555555555555555",
    "444444444444444444",
    "442444444444442444",
    "444333333333334444",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "322222222222222223",
    "222222222222222222",
    "222222222222222222",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
];

const TOWER_PROPS: [&str; 18] = [
    ".........=........",
    "...#.....=.....#..",
    ".........=........",
    "..B......=......B.",
    ".........=........",
    "##.......=.......#",
    ".........=........",
    "..#......=......#.",
    ".........=........",
    "....T....=....T...",
    ".........=........",
    "..T......=......T.",
    ".........=........",
    "...#.....=.....#..",
    ".........=........",
    "..T......=......T.",
    ".........=........",
    ".........=........",
];

const DELTA_GROUND: [&str; 18] = [
    "333333333333333333",
    "333333333333333333",
    "222222222222222222",
    "333333333333333333",
    "222222222222222222",
    "~~~~~~~~33~~~~~~~~",
    "222222222222222222",
    "333333333333333333",
    "222222222222222222",
    "~~~~~~~33~~~~~~~~~",
    "222222222222222222",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
];

const DELTA_PROPS: [&str; 18] = [
    "........=.........",
    "..T.....=......T..",
    "........=.........",
    "....B...=...B.....",
    "........=.........",
    "........==........",
    "........=.........",
    "..#.....=.....#...",
    "........=.........",
    ".......==.........",
    "........=.........",
    "..T.....=......T..",
    "........=.........",
    "....B...=...B.....",
    "........=.........",
    "..T.....=......T..",
    "........=.........",
    "...#....=....#....",
];

const LAST_GROUND: [&str; 18] = [
    "666666666666666666",
    "665555555555555566",
    "665555555555555566",
    "665555555555555566",
    "665555555555555566",
    "664444444444444466",
    "444444444444444444",
    "444444444444444444",
    "333333333333333333",
    "~~~~~33~~~~~~33~~~",
    "~~~~~33~~~~~~33~~~",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
    "333333333333333333",
];

const LAST_PROPS: [&str; 18] = [
    "......=......=....",
    "..#...=......=...#",
    "......=......=....",
    "....B.=......=.B..",
    "......=......=....",
    "..#...=......=...#",
    "......=......=....",
    "......=......=....",
    "##....=......=...#",
    "......=......=....",
    "......=......=....",
    "......=......=....",
    "...T..=......=..T.",
    "......=......=....",
    "..B...=......=...B",
    "......=......=....",
    "...T..=......=..T.",
    "......=......=....",
];

const FORD_SITES: [Site; 4] = [
    site(StructureKind::Headquarters, Some(Team::Player), 8, 16),
    site(StructureKind::Depot, None, 3, 12),
    site(StructureKind::Depot, None, 14, 12),
    site(StructureKind::Tower, None, 8, 7),
];

const FORD_SPOTS: [(u32, u32); 6] = [(6, 13), (8, 13), (10, 13), (7, 14), (9, 14), (11, 14)];

const FORD_SQUAD: [Recruit; 4] = [
    recruit("Marauder", Class::Trooper, 5, 4, Facing::South, -8, 0),
    recruit("Brigand", Class::Trooper, 11, 4, Facing::South, -8, 0),
    recruit("Poacher", Class::Grenadier, 8, 2, Facing::South, -6, 0),
    recruit("Cutter", Class::Recon, 13, 6, Facing::South, -6, 0),
];

const FORD_PLAN: Plan = Plan {
    ground: &FORD_GROUND,
    props: &FORD_PROPS,
    sites: &FORD_SITES,
    spots: &FORD_SPOTS,
    squad: &FORD_SQUAD,
};

const ROAD_SITES: [Site; 5] = [
    site(StructureKind::Headquarters, Some(Team::Enemy), 9, 1),
    site(StructureKind::Headquarters, Some(Team::Player), 9, 16),
    site(StructureKind::Factory, Some(Team::Enemy), 3, 4),
    site(StructureKind::Depot, None, 15, 8),
    site(StructureKind::Depot, None, 2, 12),
];

const ROAD_SPOTS: [(u32, u32); 6] = [(7, 13), (9, 13), (11, 13), (6, 14), (12, 14), (9, 14)];

const ROAD_SQUAD: [Recruit; 5] = [
    recruit("Bulwark", Class::Tank, 9, 4, Facing::South, 0, 0),
    recruit("Sentry", Class::Trooper, 7, 5, Facing::South, -4, 0),
    recruit("Sentry", Class::Trooper, 11, 5, Facing::South, -4, 0),
    recruit("Nailer", Class::Grenadier, 9, 2, Facing::South, 0, 1),
    recruit("Outrider", Class::Recon, 14, 7, Facing::South, -4, 0),
];

const ROAD_PLAN: Plan = Plan {
    ground: &ROAD_GROUND,
    props: &ROAD_PROPS,
    sites: &ROAD_SITES,
    spots: &ROAD_SPOTS,
    squad: &ROAD_SQUAD,
};

const RIVER_SITES: [Site; 4] = [
    site(StructureKind::Headquarters, Some(Team::Player), 9, 16),
    site(StructureKind::Tower, Some(Team::Player), 4, 13),
    site(StructureKind::Depot, None, 14, 12),
    site(StructureKind::Depot, Some(Team::Enemy), 9, 2),
];

const RIVER_SPOTS: [(u32, u32); 6] = [(8, 12), (10, 12), (6, 13), (12, 13), (9, 14), (7, 11)];

const RIVER_SQUAD: [Recruit; 6] = [
    recruit("Leech", Class::Gunboat, 3, 8, Facing::South, 0, 0),
    recruit("Leech", Class::Gunboat, 14, 8, Facing::South, 0, 0),
    recruit("Wader", Class::Trooper, 8, 6, Facing::South, -4, 0),
    recruit("Wader", Class::Trooper, 10, 6, Facing::South, -4, 0),
    recruit("Vulture", Class::Helicopter, 9, 3, Facing::South, 0, 1),
    recruit("Nailer", Class::Grenadier, 6, 4, Facing::South, 0, 0),
];

const RIVER_PLAN: Plan = Plan {
    ground: &RIVER_GROUND,
    props: &RIVER_PROPS,
    sites: &RIVER_SITES,
    spots: &RIVER_SPOTS,
    squad: &RIVER_SQUAD,
};

const TOWER_SITES: [Site; 5] = [
    site(StructureKind::Tower, Some(Team::Enemy), 9, 6),
    site(StructureKind::Headquarters, Some(Team::Enemy), 9, 3),
    site(StructureKind::Depot, Some(Team::Enemy), 13, 5),
    site(StructureKind::Depot, None, 4, 9),
    site(StructureKind::Factory, None, 14, 13),
];

const TOWER_SPOTS: [(u32, u32); 6] = [(7, 15), (9, 15), (11, 15), (5, 14), (13, 14), (9, 13)];

const TOWER_SQUAD: [Recruit; 7] = [
    recruit("Gunner", Class::Artillery, 9, 4, Facing::South, 0, 1),
    recruit("Bulwark", Class::Tank, 8, 7, Facing::South, 0, 1),
    recruit("Bulwark", Class::Tank, 10, 7, Facing::South, 0, 1),
    recruit("Sentry", Class::Trooper, 6, 8, Facing::South, 0, 0),
    recruit("Sentry", Class::Trooper, 12, 8, Facing::South, 0, 0),
    recruit("Vulture", Class::Helicopter, 4, 5, Facing::South, 0, 0),
    recruit("Outrider", Class::Recon, 14, 9, Facing::South, 0, 0),
];

const TOWER_PLAN: Plan = Plan {
    ground: &TOWER_GROUND,
    props: &TOWER_PROPS,
    sites: &TOWER_SITES,
    spots: &TOWER_SPOTS,
    squad: &TOWER_SQUAD,
};

const DELTA_SITES: [Site; 4] = [
    site(StructureKind::Headquarters, Some(Team::Player), 9, 16),
    site(StructureKind::Depot, Some(Team::Player), 4, 14),
    site(StructureKind::Depot, Some(Team::Player), 14, 14),
    site(StructureKind::Tower, None, 8, 9),
];

const DELTA_SPOTS: [(u32, u32); 6] = [(7, 13), (9, 13), (11, 13), (5, 14), (13, 14), (9, 12)];

const DELTA_SQUAD: [Recruit; 8] = [
    recruit("Wader", Class::Trooper, 5, 3, Facing::South, 0, 1),
    recruit("Wader", Class::Trooper, 8, 3, Facing::South, 0, 1),
    recruit("Wader", Class::Trooper, 11, 3, Facing::South, 0, 1),
    recruit("Nailer", Class::Grenadier, 13, 6, Facing::South, 0, 1),
    recruit("Bulwark", Class::Tank, 9, 2, Facing::South, 6, 1),
    recruit("Leech", Class::Gunboat, 2, 9, Facing::South, 0, 1),
    recruit("Vulture", Class::Helicopter, 15, 6, Facing::South, 0, 1),
    recruit("Gunner", Class::Artillery, 9, 1, Facing::South, 0, 2),
];

const DELTA_PLAN: Plan = Plan {
    ground: &DELTA_GROUND,
    props: &DELTA_PROPS,
    sites: &DELTA_SITES,
    spots: &DELTA_SPOTS,
    squad: &DELTA_SQUAD,
};

const LAST_SITES: [Site; 6] = [
    site(StructureKind::Headquarters, Some(Team::Enemy), 9, 2),
    site(StructureKind::Factory, Some(Team::Enemy), 5, 3),
    site(StructureKind::Factory, Some(Team::Enemy), 13, 3),
    site(StructureKind::Depot, None, 3, 12),
    site(StructureKind::Depot, None, 15, 12),
    site(StructureKind::Headquarters, Some(Team::Player), 9, 16),
];

const LAST_SPOTS: [(u32, u32); 6] = [(7, 13), (9, 13), (11, 13), (6, 14), (12, 14), (9, 12)];

const LAST_SQUAD: [Recruit; 9] = [
    recruit("Col. Gaffe", Class::Walker, 9, 4, Facing::South, 20, 3),
    recruit("Bulwark", Class::Tank, 7, 5, Facing::South, 6, 2),
    recruit("Bulwark", Class::Tank, 11, 5, Facing::South, 6, 2),
    recruit("Gunner", Class::Artillery, 6, 3, Facing::South, 0, 2),
    recruit("Gunner", Class::Artillery, 12, 3, Facing::South, 0, 2),
    recruit("Sentry", Class::Trooper, 8, 7, Facing::South, 4, 1),
    recruit("Sentry", Class::Trooper, 10, 7, Facing::South, 4, 1),
    recruit("Vulture", Class::Helicopter, 4, 6, Facing::South, 0, 2),
    recruit("Rustbucket", Class::Bomber, 14, 6, Facing::South, 0, 2),
];

const LAST_PLAN: Plan = Plan {
    ground: &LAST_GROUND,
    props: &LAST_PROPS,
    sites: &LAST_SITES,
    spots: &LAST_SPOTS,
    squad: &LAST_SQUAD,
};

const FORD_BRIEF: Scene = &[
    (
        "Sgt. Beoric",
        "Raiders burned the mill at the ford last night. Four of us, that is what we have.",
    ),
    (
        "Mireille",
        "Four is what the ledger says and the ledger does not lie, sir.",
    ),
    ("Tessa", "I can hold the near bank while the others cross."),
    (
        "Sgt. Beoric",
        "Then we go at first light. Say the word, Commander.",
    ),
];

const FORD_DEBRIEF: Scene = &[
    (
        "Ironhide",
        "Ford is clear. I have never seen raiders run that fast.",
    ),
    (
        "Mireille",
        "Hot food tonight. I found flour nobody was counting.",
    ),
    (
        "Sgt. Beoric",
        "Do not let them get used to winning. The road north is worse.",
    ),
];

const ROAD_BRIEF: Scene = &[
    (
        "Dash",
        "Commander. Dash, out of the northern column. I came the long way.",
    ),
    (
        "Dash",
        "Red armor sits on the iron road. Their headquarters is at the top of it.",
    ),
    (
        "Sgt. Beoric",
        "Take the flag and the road is ours. Simple, not easy.",
    ),
    (
        "Dash",
        "Nothing is easy. Let me scout ahead and I will make it easier.",
    ),
];

const ROAD_DEBRIEF: Scene = &[
    (
        "Dash",
        "Their flag came down and half the column just walked away.",
    ),
    (
        "Mireille",
        "Which means the road north is open, and so is the ledger.",
    ),
    (
        "Sgt. Beoric",
        "Gaffe will hear about this by morning. Count on it.",
    ),
];

const RIVER_BRIEF: Scene = &[
    (
        "Tidewater",
        "You must be the Commander. Tidewater, and the boat is mine, so mind the paint.",
    ),
    (
        "Tidewater",
        "Gaffe put gunboats on my river. There is one bridge and they want it.",
    ),
    ("Tessa", "Then we hold the bridge. How long?"),
    (
        "Tidewater",
        "Twelve turns of the glass and their fuel runs dry. I counted.",
    ),
];

const RIVER_DEBRIEF: Scene = &[
    (
        "Tidewater",
        "The river is quiet and the crossing is ours. I told you I counted.",
    ),
    ("Tessa", "You did. Loudly. The whole time."),
    ("Tidewater", "Loud is how you stay alive on the water."),
];

const TOWER_BRIEF: Scene = &[
    (
        "Longarm",
        "Longarm, gunner. They gave me the range tables and no crew to use them.",
    ),
    (
        "Longarm",
        "The tower on that hill carries every order Gaffe gives.",
    ),
    ("Mireille", "Stand on it and the valley hears us instead."),
    (
        "Sgt. Beoric",
        "Then we take the hill. Longarm, you have a crew now.",
    ),
];

const TOWER_DEBRIEF: Scene = &[
    (
        "Longarm",
        "The tower speaks with our voice. I have waited a long time to say that.",
    ),
    ("Dash", "Gaffe rode north. Alone, and fast."),
    (
        "Sgt. Beoric",
        "A cornered colonel is worse than a confident one. Rest while you can.",
    ),
];

const DELTA_BRIEF: Scene = &[
    (
        "Kestrel",
        "Kestrel. I flew for the other side of this valley until this morning.",
    ),
    (
        "Kestrel",
        "They are coming through the delta at first light. All of them.",
    ),
    (
        "Mireille",
        "The column needs fourteen turns to clear the depots.",
    ),
    (
        "Kestrel",
        "Then give me fourteen turns and I will keep the sky off your backs.",
    ),
];

const DELTA_DEBRIEF: Scene = &[
    (
        "Kestrel",
        "Column is away. The delta cost them more than it cost us.",
    ),
    ("Ironhide", "It cost me a track and a week of sleep."),
    (
        "Mireille",
        "It cost me the last of the coffee. Nobody thank me.",
    ),
];

const LAST_BRIEF: Scene = &[
    (
        "Atlas",
        "Atlas. This walker was my father's. Gaffe took the field he died on.",
    ),
    (
        "Sgt. Beoric",
        "The colonel has dug in with everything he has left.",
    ),
    ("Tessa", "Everything he has left is still a great deal."),
    (
        "Atlas",
        "Then we bring everything we have. This is the last field, Commander.",
    ),
];

const LAST_DEBRIEF: Scene = &[
    (
        "Sgt. Beoric",
        "The valley is quiet. I have not heard it quiet in two years.",
    ),
    ("Mireille", "The ledger balances. Every name still on it."),
    (
        "Atlas",
        "Then we go home together, and the machine stays parked.",
    ),
];

pub const CHAPTERS: [Chapter; 6] = [
    Chapter {
        name: "Smoke Over the Ford",
        intro: "Raiders burned the mill at the ford. Four of you march. Clear the bank.",
        victory: "The ford is clear. The squad eats hot food for the first time in a week.",
        objective: Objective::Rout,
        days: 2,
        deploy: 4,
        reward: 60,
        joins: &[0, 1, 2, 3],
        brief: FORD_BRIEF,
        debrief: FORD_DEBRIEF,
        plan: &FORD_PLAN,
    },
    Chapter {
        name: "The Iron Road",
        intro: "Red armor holds the north road. Take their headquarters and the road is ours.",
        victory: "The red flag comes down. The road north is open.",
        objective: Objective::Seize,
        days: 2,
        deploy: 4,
        reward: 80,
        joins: &[4],
        brief: ROAD_BRIEF,
        debrief: ROAD_DEBRIEF,
        plan: &ROAD_PLAN,
    },
    Chapter {
        name: "The River Crossing",
        intro: "Gunboats work the river and the bridge is the only way over. Hold it.",
        victory: "The river runs quiet again and the crossing is ours.",
        objective: Objective::Survive(12),
        days: 3,
        deploy: 5,
        reward: 90,
        joins: &[8],
        brief: RIVER_BRIEF,
        debrief: RIVER_DEBRIEF,
        plan: &RIVER_PLAN,
    },
    Chapter {
        name: "The Radio Tower",
        intro: "The tower on the hill carries Gaffe's orders. Stand on it and the valley hears us.",
        victory: "The tower speaks with our voice. Gaffe rode north, alone.",
        objective: Objective::Seize,
        days: 3,
        deploy: 5,
        reward: 110,
        joins: &[5],
        brief: TOWER_BRIEF,
        debrief: TOWER_DEBRIEF,
        plan: &TOWER_PLAN,
    },
    Chapter {
        name: "The Long Retreat",
        intro: "They came through the delta at first light. Hold the depots until the column clears.",
        victory: "The column is away. The delta cost them more than it cost us.",
        objective: Objective::Survive(14),
        days: 3,
        deploy: 6,
        reward: 120,
        joins: &[6],
        brief: DELTA_BRIEF,
        debrief: DELTA_DEBRIEF,
        plan: &DELTA_PLAN,
    },
    Chapter {
        name: "Gaffe's Last Field",
        intro: "The colonel has dug in with everything he has left. This is the last field.",
        victory: "The valley is quiet. The squad goes home together.",
        objective: Objective::Rout,
        days: 4,
        deploy: 6,
        reward: 160,
        joins: &[7],
        brief: LAST_BRIEF,
        debrief: LAST_DEBRIEF,
        plan: &LAST_PLAN,
    },
];
