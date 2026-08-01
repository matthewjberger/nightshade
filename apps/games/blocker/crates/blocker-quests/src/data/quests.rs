use crate::data::spots::{BARROW_MOUTH, WATCH_LODGE, WOODS_CAMP};
use crate::data::trade::giver_named;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::crops::CropKind;
use blocker_blocks::data::items::Item;
use blocker_blocks::data::tools::ToolKind;

pub enum Objective {
    Gather { item: Item, count: u32 },
    Explore { count: usize },
    Slay { beast: usize, count: u32 },
    Meet { giver: usize },
    Visit { place: usize },
    Report,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QuestId {
    BreadAndBoard,
    TheForgeRunsCold,
    BlackRock,
    ARoofThatHolds,
    RootsAndPetals,
    LearnTheLand,
    ClearPanes,
    CoinOfTheRealm,
    TheLongPatrol,
    TheMarketStall,
    ALightBelow,
    StoneOnStone,
    BoarTrouble,
    WolvesAtTheTreeline,
    VenisonForTheInn,
    TheBearOfTheHollow,
    CullTheDirewolves,
    WhatWalksInTheBarrow,
    WhatTookThem,
    ReportToTheWatch,
    NothingDownThere,
    TheTrailEast,
}

pub struct Quest {
    pub id: QuestId,
    pub title: &'static str,
    pub story: &'static str,
    pub giver: usize,
    pub objective: Objective,
    pub reward: Item,
    pub reward_count: u32,
    pub loan: &'static [(Item, u32)],
    pub next: Option<usize>,
    pub renown: u32,
    pub accept: &'static [&'static str],
    pub complete: &'static [&'static str],
    pub nudge: &'static str,
    pub place: Option<usize>,
}

pub const BOAR_KIND: usize = 0;

pub const WOLF_KIND: usize = 1;

pub const DEER_KIND: usize = 2;

pub const BEAR_KIND: usize = 3;

pub const DIREWOLF_KIND: usize = 4;

pub const ZOMBIE_KIND: usize = 5;

pub const STARTER: usize = QuestId::WhatWalksInTheBarrow as usize;

pub const WIGHT_HUNT: usize = QuestId::WhatTookThem as usize;

pub const WIGHT_KIND: usize = 6;

pub const OPENING: usize = QuestId::ReportToTheWatch as usize;

pub const EMPTY_CHAMBER: usize = QuestId::NothingDownThere as usize;

pub const TRAIL_EAST: usize = QuestId::TheTrailEast as usize;

const fn wandering_quests() -> usize {
    let mut count = 0;
    while count < QUESTS.len() && QUESTS[count].giver != crate::data::trade::WATCH_GIVER {
        count += 1;
    }
    count
}

pub const WANDERING_QUESTS: usize = wandering_quests();

const fn watch_quests_come_last() -> bool {
    let mut index = WANDERING_QUESTS;
    while index < QUESTS.len() {
        if QUESTS[index].giver != crate::data::trade::WATCH_GIVER {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(watch_quests_come_last());

pub const QUESTS: [Quest; 22] = [
    Quest {
        id: QuestId::BreadAndBoard,
        title: "Bread and Board",
        story: "The inn eats what Holt grows, and the fields are bare.",
        giver: giver_named("Holt"),
        objective: Objective::Gather {
            item: Item::Crop(CropKind::Wheat),
            count: 5,
        },
        reward: Item::Block(BlockKind::Torch),
        reward_count: 8,
        loan: &[],
        next: None,
        renown: 20,
        accept: &[
            "Five bundles of wheat and the ovens stay warm this week.",
            "The field is past the last house. Mind the crows.",
        ],
        complete: &["Good weight, all of it. Take these, the nights are long."],
        nudge: "Five bundles. Come back when the sacks are full.",
        place: None,
    },
    Quest {
        id: QuestId::TheForgeRunsCold,
        title: "The Forge Runs Cold",
        story: "Garron has orders and no iron to fill them.",
        giver: giver_named("Garron"),
        objective: Objective::Gather {
            item: Item::Block(BlockKind::IronOre),
            count: 5,
        },
        reward: Item::Tool(ToolKind::Pickaxe),
        reward_count: 1,
        loan: &[],
        next: None,
        renown: 30,
        accept: &[
            "Five lumps of iron ore and the forge lights again. Nothing in this village gets mended until it does.",
            "You will not find it on the surface. Cut a shaft and go down past the coal, down where your torch stops showing you the walls. It runs in short seams of three, so where you find one there are two more beside it.",
            "Bring a pick and bring torches. People who bring one and not the other come back up with neither.",
        ],
        complete: &["Real iron. Here, I made this before you asked. Wear it out and come back."],
        nudge: "Five lumps of iron. Deeper than the coal, where the torch stops reaching.",
        place: None,
    },
    Quest {
        id: QuestId::BlackRock,
        title: "Black Rock",
        story: "Every lantern in the vale burns Mabb's coal.",
        giver: giver_named("Mabb"),
        objective: Objective::Gather {
            item: Item::Block(BlockKind::CoalOre),
            count: 8,
        },
        reward: Item::Block(BlockKind::Torch),
        reward_count: 16,
        loan: &[],
        next: None,
        renown: 25,
        accept: &[
            "Eight of coal and every lantern in the vale stays lit through the dark months.",
            "Coal is the easy one. Break through the dirt into the grey stone and it is already there, black seams in threes, not far under your own feet.",
            "Any hillside with its stone showing will do. You do not need a deep shaft, you need patience and a pick.",
        ],
        complete: &["Enough to see the winter through. Torches, on the house."],
        nudge: "Eight of coal. Just under the dirt, in the grey stone.",
        place: None,
    },
    Quest {
        id: QuestId::ARoofThatHolds,
        title: "A Roof That Holds",
        story: "The inn's roof leaks and Bram is out of wood.",
        giver: giver_named("Bram"),
        objective: Objective::Gather {
            item: Item::Block(BlockKind::OakLog),
            count: 8,
        },
        reward: Item::Block(BlockKind::OakPlanks),
        reward_count: 16,
        loan: &[],
        next: None,
        renown: 15,
        accept: &[
            "Eight good logs and the roof stops leaking on the guests.",
            "The forest starts past the well.",
        ],
        complete: &["That will patch it. Planks for your trouble, from the same load."],
        nudge: "Eight logs. The rain is still coming through.",
        place: None,
    },
    Quest {
        id: QuestId::RootsAndPetals,
        title: "Roots and Petals",
        story: "Wynne's shelves are bare of red bloom.",
        giver: giver_named("Wynne"),
        objective: Objective::Gather {
            item: Item::Block(BlockKind::RedFlower),
            count: 4,
        },
        reward: Item::Block(BlockKind::Glowstone),
        reward_count: 2,
        loan: &[],
        next: None,
        renown: 20,
        accept: &[
            "Four red flowers. The salve does not mix without them.",
            "They grow where the grass is thick.",
        ],
        complete: &["Bright ones. Here, a little light for the dark places. Do not eat it."],
        nudge: "Four red flowers. The salve is only half made.",
        place: None,
    },
    Quest {
        id: QuestId::LearnTheLand,
        title: "Learn the Land",
        story: "Marren says a hunter who does not know the roads is a dead one.",
        giver: giver_named("Marren"),
        objective: Objective::Explore { count: 3 },
        reward: Item::Tool(ToolKind::Sword),
        reward_count: 1,
        loan: &[],
        next: None,
        renown: 40,
        accept: &[
            "Walk to three of the settlements and see them with your own eyes.",
            "The land teaches faster than I do. Go.",
        ],
        complete: &["Three, and back in one piece. Take the blade, you have earned its weight."],
        nudge: "Three settlements. You have not seen enough of them yet.",
        place: None,
    },
    Quest {
        id: QuestId::ClearPanes,
        title: "Clear Panes",
        story: "Pell wants glass for the harbor lamps and needs sand to melt.",
        giver: giver_named("Pell"),
        objective: Objective::Gather {
            item: Item::Block(BlockKind::Sand),
            count: 12,
        },
        reward: Item::Block(BlockKind::Glass),
        reward_count: 8,
        loan: &[],
        next: None,
        renown: 15,
        accept: &[
            "A dozen loads of sand and the harbor gets its glass.",
            "The beaches are yellow with it. Bring a shovel.",
        ],
        complete: &["Fine grain. Glass for you, cut square. Watch it does not break in the pack."],
        nudge: "A dozen of sand. The lamps are still dark.",
        place: None,
    },
    Quest {
        id: QuestId::CoinOfTheRealm,
        title: "Coin of the Realm",
        story: "Fenn deals in gold and is short of it.",
        giver: giver_named("Fenn"),
        objective: Objective::Gather {
            item: Item::Block(BlockKind::GoldOre),
            count: 3,
        },
        reward: Item::Block(BlockKind::Torch),
        reward_count: 32,
        loan: &[],
        next: None,
        renown: 35,
        accept: &[
            "Three of gold ore and I will not ask which hole you pulled them out of.",
            "Gold sits low. Not at the bottom, but close enough that you will feel the air change on the way down. Cut past the iron seams and keep going.",
            "Take twice the torches you think you need and leave a lit one at every turn. The ones who get lost down there are the ones who dug in the dark.",
        ],
        complete: &["Heavy. Good. A trader pays in what he has, and I have torches. Plenty."],
        nudge: "Three of gold. Below the iron, and take lights.",
        place: None,
    },
    Quest {
        id: QuestId::TheLongPatrol,
        title: "The Long Patrol",
        story: "Idris wants the roads walked and the settlements counted.",
        giver: giver_named("Idris"),
        objective: Objective::Explore { count: 5 },
        reward: Item::Block(BlockKind::Bricks),
        reward_count: 16,
        loan: &[],
        next: None,
        renown: 50,
        accept: &[
            "Five settlements. The watch cannot be everywhere, but you can.",
            "Report back when your boots are worn through.",
        ],
        complete: &["Five, and the roads still open. Brick, for a wall of your own."],
        nudge: "Five settlements. The patrol is not done.",
        place: None,
    },
    Quest {
        id: QuestId::TheMarketStall,
        title: "The Market Stall",
        story: "Wenna's stall is empty and market day is coming.",
        giver: giver_named("Wenna"),
        objective: Objective::Gather {
            item: Item::Crop(CropKind::Carrot),
            count: 6,
        },
        reward: Item::Block(BlockKind::Torch),
        reward_count: 8,
        loan: &[],
        next: None,
        renown: 15,
        accept: &[
            "Six carrots for the stall. Market day waits for no one.",
            "Holt's field, or grow your own. Either way, six.",
        ],
        complete: &["Bright orange, the lot. Torches, so you can dig after dark."],
        nudge: "Six carrots. The stall is still bare.",
        place: None,
    },
    Quest {
        id: QuestId::ALightBelow,
        title: "A Light Below",
        story: "Ael would set one true light in the deepest dark.",
        giver: giver_named("Ael"),
        objective: Objective::Gather {
            item: Item::Block(BlockKind::DiamondOre),
            count: 1,
        },
        reward: Item::Block(BlockKind::Glowstone),
        reward_count: 4,
        loan: &[],
        next: None,
        renown: 60,
        accept: &[
            "One diamond. Nothing else in the ground holds a light the way it does.",
            "It lies almost on the bedrock, in the last few layers before the world stops. If you have not hit the black floor yet, you have not gone deep enough.",
            "Dig a long gallery down there rather than a hole. You will walk past ten before you see one, and you will only see it because it is the only thing shining back.",
        ],
        complete: &["It shines even here, in my hand. Carry some of it out with you."],
        nudge: "One diamond. Down on the bedrock, and cut a long gallery.",
        place: None,
    },
    Quest {
        id: QuestId::StoneOnStone,
        title: "Stone on Stone",
        story: "Garron is laying a new wall and short of cobble.",
        giver: giver_named("Garron"),
        objective: Objective::Gather {
            item: Item::Block(BlockKind::Cobblestone),
            count: 16,
        },
        reward: Item::Block(BlockKind::Bricks),
        reward_count: 8,
        loan: &[],
        next: None,
        renown: 20,
        accept: &[
            "Sixteen of cobble for the wall. Rough is fine, it goes under the render.",
            "Any hillside will give it up to a pick.",
        ],
        complete: &[
            "That is a course and a half. Fired brick for you, better than you brought me.",
        ],
        nudge: "Sixteen of cobble. The wall is barely a foot high.",
        place: None,
    },
    Quest {
        id: QuestId::BoarTrouble,
        title: "Boar Trouble",
        story: "The boars have been at the fields again, and Marren wants them thinned.",
        giver: giver_named("Marren"),
        objective: Objective::Slay {
            beast: BOAR_KIND,
            count: 3,
        },
        reward: Item::Block(BlockKind::IronOre),
        reward_count: 3,
        loan: &[],
        next: None,
        renown: 24,
        accept: &[
            "Three boars off the field and the crop has a chance.",
            "They are stubborn. Corner one and do not let it turn on you.",
        ],
        complete: &["Three less to root up the barley. Take this for your trouble."],
        nudge: "Three boars. The field is still churned to mud.",
        place: None,
    },
    Quest {
        id: QuestId::WolvesAtTheTreeline,
        title: "Wolves at the Treeline",
        story: "A pack has come down from the hills and the road is not safe.",
        giver: giver_named("Marren"),
        objective: Objective::Slay {
            beast: WOLF_KIND,
            count: 3,
        },
        reward: Item::Block(BlockKind::Glowstone),
        reward_count: 2,
        loan: &[],
        next: None,
        renown: 34,
        accept: &[
            "Three wolves and the traders will use the road again.",
            "They hunt in a pack, so keep your back to a wall.",
        ],
        complete: &["The road is quiet again. You earned this light."],
        nudge: "Three wolves. The pack still holds the treeline.",
        place: None,
    },
    Quest {
        id: QuestId::VenisonForTheInn,
        title: "Venison for the Inn",
        story: "Holt needs meat for the tables and the deer are quick.",
        giver: giver_named("Holt"),
        objective: Objective::Slay {
            beast: DEER_KIND,
            count: 4,
        },
        reward: Item::Block(BlockKind::Torch),
        reward_count: 12,
        loan: &[],
        next: None,
        renown: 20,
        accept: &[
            "Four deer and the inn eats well for a week.",
            "They will bolt at the first sound. Come at them from downwind.",
        ],
        complete: &["Good meat, and plenty of it. The tables will be full tonight."],
        nudge: "Four deer. The larder is still bare.",
        place: None,
    },
    Quest {
        id: QuestId::TheBearOfTheHollow,
        title: "The Bear of the Hollow",
        story: "A bear has taken the hollow and no one will cut wood there.",
        giver: giver_named("Marren"),
        objective: Objective::Slay {
            beast: BEAR_KIND,
            count: 1,
        },
        reward: Item::Block(BlockKind::DiamondOre),
        reward_count: 1,
        loan: &[],
        next: None,
        renown: 60,
        accept: &[
            "One bear. It is bigger than you, so pick your ground.",
            "Do not let it close. It hits like a falling tree.",
        ],
        complete: &["You took the bear. The woodcutters can go back. This is yours by right."],
        nudge: "The bear still holds the hollow.",
        place: None,
    },
    Quest {
        id: QuestId::CullTheDirewolves,
        title: "Cull the Direwolves",
        story: "Direwolves have crossed the ridge and the herds are afraid.",
        giver: giver_named("Marren"),
        objective: Objective::Slay {
            beast: DIREWOLF_KIND,
            count: 3,
        },
        reward: Item::Block(BlockKind::GoldOre),
        reward_count: 4,
        loan: &[],
        next: None,
        renown: 44,
        accept: &[
            "Three direwolves and the herds settle.",
            "They are faster than the grey wolves. Do not run.",
        ],
        complete: &["The ridge is quiet. Gold for a hard day's work."],
        nudge: "Three direwolves. The herds still will not graze.",
        place: None,
    },
    Quest {
        id: QuestId::WhatWalksInTheBarrow,
        title: "What Walks in the Barrow",
        story: "Four hikers went into the barrow south-east of the city three weeks ago. The watch sent two of its own after them. None of the six walked back out, and the ones who go near now say they hear somebody counting.",
        giver: crate::data::trade::WATCH_GIVER,
        objective: Objective::Slay {
            beast: ZOMBIE_KIND,
            count: 4,
        },
        reward: Item::Block(BlockKind::Glowstone),
        reward_count: 3,
        loan: &[
            (Item::Tool(ToolKind::Sword), 1),
            (Item::Block(BlockKind::Torch), 6),
        ],
        next: Some(EMPTY_CHAMBER),
        renown: 40,
        accept: &[
            "Out the south-east gate, then keep the wall on your right until it runs out. The mound is the only thing standing on that ground and it has three grave markers in front of it. Our markers. We put them up when we stopped hoping.",
            "The things inside wear the faces of the people we sent. Four of them put down and I will know what we are dealing with. Do not try to count them. You will lose your nerve.",
            "Take the blade off the rack and six torches from the barrel. The blade comes back if you do. The torches are yours either way, and you will want every one of them.",
        ],
        complete: &[
            "Four. And no bodies, you say. No packs, no boots, nothing of theirs at all.",
            "Then they walked out of there on their own feet. Which is worse.",
        ],
        nudge: "Four of them. South-east gate, keep the wall on your right, look for the mound with the three markers.",
        place: Some(BARROW_MOUTH),
    },
    Quest {
        id: QuestId::WhatTookThem,
        title: "What Took Them",
        story: "The camp in the trees is still pitched and the fire is still warm. Whatever walked the hikers out of the barrow brought them here, and it is still sitting with them.",
        giver: crate::data::trade::WATCH_GIVER,
        objective: Objective::Slay {
            beast: WIGHT_KIND,
            count: 1,
        },
        reward: Item::Block(BlockKind::GoldOre),
        reward_count: 6,
        loan: &[(Item::Block(BlockKind::Torch), 8)],
        next: None,
        renown: 110,
        accept: &[
            "There is one thing in that camp that is not shambling. It stands straight, it is a head taller than you, and it is the one that has been counting.",
            "It hits harder than the walkers and it does not tire. Fight it in the open between the tents where you can give ground, not in the trees.",
            "Bring me nothing. I do not want a trophy. I want it to stop.",
        ],
        complete: &[
            "Then that is the end of it. Six names come off the roll tonight as dead instead of missing, which is the only mercy this job ever hands out.",
            "Gold from the city chest. Nobody downstairs will ask what it was for.",
        ],
        nudge: "The tall one is still in the camp. Between the tents, in the open.",
        place: Some(WOODS_CAMP),
    },
    Quest {
        id: QuestId::ReportToTheWatch,
        title: "Report to the Watch",
        story: "You came in through the gate with nothing to your name. The city watch takes on anyone who can hold a blade, and they are short of people this month.",
        giver: crate::data::trade::WATCH_GIVER,
        objective: Objective::Meet {
            giver: crate::data::trade::WATCH_GIVER,
        },
        reward: Item::Block(BlockKind::Torch),
        reward_count: 4,
        loan: &[],
        next: Some(STARTER),
        renown: 10,
        accept: &["Ask at the watch lodge, in the south-east quarter inside the wall."],
        complete: &[
            "You took your time. I am Ovilda, I keep the watch on this quarter, and I have a job nobody else will take.",
        ],
        nudge: "The watch lodge stands in the south-east quarter of the city, inside the wall. Follow the gold marker on your compass.",
        place: Some(WATCH_LODGE),
    },
    Quest {
        id: QuestId::NothingDownThere,
        title: "Nothing Down There",
        story: "Six people went into that mound and none of them are in it. Ovilda needs to hear it from you, and she needs to hear it tonight.",
        giver: crate::data::trade::WATCH_GIVER,
        objective: Objective::Report,
        reward: Item::Block(BlockKind::Torch),
        reward_count: 6,
        loan: &[],
        next: Some(TRAIL_EAST),
        renown: 25,
        accept: &["Go back to the lodge and tell her what you did not find."],
        complete: &[
            "You are looking at the shutters. Yes. They came off while you were underground.",
            "It came in through there, put me on the floor, and went straight for the roll on that table. The roll with the six names on it. It did not touch the coin box.",
            "It went out the east gate at a walk. It left wet earth on the stones the whole way, and nobody on the wall thought to stop a thing that was walking that calmly.",
        ],
        nudge: "Ovilda is at the watch lodge, south-east quarter. She needs to hear this.",
        place: Some(WATCH_LODGE),
    },
    Quest {
        id: QuestId::TheTrailEast,
        title: "The Trail East",
        story: "Wet earth on the stones, out the east gate, holding a straight line for the treeline. Something wanted to be followed, or it did not care.",
        giver: crate::data::trade::WATCH_GIVER,
        objective: Objective::Visit {
            place: crate::data::spots::WOODS_CAMP,
        },
        reward: Item::Block(BlockKind::Torch),
        reward_count: 8,
        loan: &[],
        next: Some(WIGHT_HUNT),
        renown: 35,
        accept: &[
            "East gate, then hold the line the trail holds. It runs about a hundred paces past the last field and into the oaks. You will see the smoke before you see the tents.",
            "Do not walk into the middle of it. Look first, then come back and tell me, or do not come back at all and I will know what that means.",
        ],
        complete: &[
            "Their own camp. Their own tents, their own fire, still going.",
            "So it did not take them anywhere. It walked them home and sat down with them.",
        ],
        nudge: "East out of the city, past the last field, into the oaks.",
        place: Some(WOODS_CAMP),
    },
];

const fn quests_line_up() -> bool {
    let mut index = 0;
    while index < QUESTS.len() {
        if QUESTS[index].id as usize != index {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(quests_line_up());
