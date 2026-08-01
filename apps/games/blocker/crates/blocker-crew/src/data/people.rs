use blocker_screen::data::names::same_name;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Giver,
    Trader,
    Folk,
}

pub struct Folk {
    pub name: &'static str,
    pub role: &'static str,
    pub lines: &'static [&'static str],
}

pub const FOLK_VILLAGERS: usize = 16;

pub const KEPT_PERSONA: usize = persona_named("Bramwell the Kept");

pub const PERCH_PERSONA: usize = persona_named("Wren the Quiet");

pub const WAITING_PERSONA: usize = persona_named("Tam Renn");

pub const WATCHMAN_PERSONA: usize = persona_named("Watchman Orrin");

pub const GATE_PERSONAS: [usize; 2] = [
    persona_named("Gatewarden Hesk"),
    persona_named("Watchman Tolly"),
];

pub const fn persona_named(name: &str) -> usize {
    let mut index = 0;
    while index < FOLK.len() {
        if same_name(FOLK[index].name, name) {
            return index;
        }
        index += 1;
    }
    panic!("the people table holds no one by that name")
}

pub const FOLK: [Folk; 22] = [
    Folk {
        name: "Old Marlow",
        role: "Fisher",
        lines: &[
            "Caught nothing but a boot today. A fine boot, mind you.",
            "The big carp lives under the far dock. One day he is mine.",
            "Rod, line, and patience. That is the whole of it.",
            "You should try the water at dawn. The fish are sleepy and slow.",
        ],
    },
    Folk {
        name: "Bram",
        role: "Blacksmith",
        lines: &[
            "The forge runs hot today. Mind your sleeves.",
            "Bring me iron and I will bring you an edge.",
            "A good hammer is worth ten swords you never swing.",
        ],
    },
    Folk {
        name: "Wren",
        role: "Herbalist",
        lines: &[
            "The blue flowers by the water heal a fever. The red ones do not.",
            "Everything is a cure or a poison. The trick is the dose.",
            "Smell this. No, the other end.",
        ],
    },
    Folk {
        name: "Tobin",
        role: "Baker",
        lines: &[
            "Fresh bread at first light. Get here early or get here sad.",
            "Flour, water, fire, time. Same as a good life.",
            "The cat keeps stealing my rolls. I have named him Toll.",
        ],
    },
    Folk {
        name: "Greta",
        role: "Innkeeper",
        lines: &[
            "A bed, a bowl, and a warm fire. What more is there.",
            "Travelers pay in coin or in stories. I take both.",
            "Do not mind the noise upstairs. That is just Dunn.",
        ],
    },
    Folk {
        name: "Cort",
        role: "Guard",
        lines: &[
            "Move along. Or do not. It is a slow day.",
            "Boars in the fields again. Watch your step past the wall.",
            "I have stood at this gate so long the gate leans on me.",
        ],
    },
    Folk {
        name: "Lark",
        role: "Bard",
        lines: &[
            "I would sing you a song, but you have not paid the toll of a smile.",
            "Every hero I sing of started as someone late for supper.",
            "Give me a deed worth a verse and I will make you famous.",
        ],
    },
    Folk {
        name: "Dunn",
        role: "Layabout",
        lines: &[
            "The trick to a long day is to lie down for most of it.",
            "I meant to be a hero. Then I sat down.",
            "You are standing in my sun.",
        ],
    },
    Folk {
        name: "Pell",
        role: "Farmer",
        lines: &[
            "Rain when I want sun, sun when I want rain. Such is the crop.",
            "The scarecrow does nothing. The boars laugh at him.",
            "Pull a weed for me and I will not tell you to.",
        ],
    },
    Folk {
        name: "Mira",
        role: "Merchant",
        lines: &[
            "Everything has a price. Even the things I say are free.",
            "I came for the trade and stayed for the view.",
            "Buy low, sell high, and never explain the middle.",
        ],
    },
    Folk {
        name: "Rue",
        role: "Miner",
        lines: &[
            "The deep stone hides the bright ones. And the dark ones.",
            "I dig down so I do not have to look up.",
            "Heard a knocking in the tunnels. Probably nothing. Probably.",
        ],
    },
    Folk {
        name: "Oswin",
        role: "Scholar",
        lines: &[
            "The standing stones are older than the maps. Older than the names.",
            "I read that a hero once walked this very road. It did not say who.",
            "Facts are stubborn. So am I. We get along.",
        ],
    },
    Folk {
        name: "Nessa",
        role: "Shepherd",
        lines: &[
            "The sheep wander less than the people, and complain half as much.",
            "A quiet hill and a full flock. That is my whole ambition.",
            "Watch for wolves at dusk. And boars. And Dunn.",
        ],
    },
    Folk {
        name: "Hob",
        role: "Wanderer",
        lines: &[
            "I have walked to every capital and back. My boots have opinions.",
            "Frostmere is cold, Emberfort is warm, and Dawnhold is home.",
            "The road gives and the road takes. Mostly it takes socks.",
        ],
    },
    Folk {
        name: "Fen",
        role: "Fisher",
        lines: &[
            "Old Marlow taught me to fish. I taught him to nap.",
            "One day I will catch the carp before he does.",
            "The river talks if you sit still long enough.",
        ],
    },
    Folk {
        name: "Ada",
        role: "Potter",
        lines: &[
            "Clay does not lie. Push it wrong and it falls apart.",
            "Every pot I make holds a little of the river in it.",
            "Drop that and you have bought it.",
        ],
    },
    Folk {
        name: "Bramwell the Kept",
        role: "Barrow Keeper",
        lines: &[
            "Put that down. I kept my wits when the rest of them lost theirs, and I would like to keep my head as well.",
            "The four came down in the wet week, laughing, with rope they never used. Then two of the watch came after them. I counted all six in.",
            "None of them are here. That is the part nobody believes. They stood up, they formed a line, and they walked out past me into the daylight.",
            "Something walked them out. It came up the corridor behind them and it did not hurry. It was counting them as they went.",
            "If you want them, do not look down. Look east, where the trees start. That is the way they went and none of them looked back at me.",
            "Tell the city I am still here. Tell them I still remember my own name, which is more than the rest of them can say.",
        ],
    },
    Folk {
        name: "Wren the Quiet",
        role: "Roof Watcher",
        lines: &[
            "You are the first one in two years to look up. Most of them never do.",
            "I can see every door in this village from here, and I know which of them open at night.",
            "Do not tell the watch I am up here. They would want reports, and then it would be work.",
            "The trees are better than the walls. Nothing that climbs a wall expects somebody to already be above it.",
            "Come back after dark. That is when this village gets interesting.",
        ],
    },
    Folk {
        name: "Tam Renn",
        role: "Waiting",
        lines: &[
            "Aldis is my brother. He went in with rope and a joke, and I have sat here twenty days listening to that mound breathe.",
            "The watch put the markers up. I did not ask them to. A marker is a way of saying you have stopped looking.",
            "Nothing comes out. Things go in, I have watched them, foxes and once a dog, and nothing comes out.",
            "If you go down there, count your torches on the way in and count them again on the way out. It is the only counting I trust now.",
            "Tell Aldis the boat is still his. He will know what that means.",
        ],
    },
    Folk {
        name: "Watchman Orrin",
        role: "Watchman",
        lines: &[
            "It sat us round the fire like it was one of us. It counted us every night before it slept, if that was sleeping.",
            "Last night it only got to five. I do not know which of us it stopped counting.",
            "I could not run. I could not lift a hand. But I could hear the whole time, and I heard you coming through the trees.",
            "Sergeant Ovilda will want a report. Tell her the roll can come down. Tell her I am sorry about the shutters.",
            "The others are still here, in a way. Do not look at their faces when you go past.",
        ],
    },
    Folk {
        name: "Gatewarden Hesk",
        role: "Gate Watch",
        lines: &[
            "Ovilda keeps the lodge. South-east quarter, past the well and bear left. She will not come to you.",
            "Three names on that board are mine to read out every dawn. I would like to stop reading them.",
            "The road is yours by day. By night it belongs to whatever does the counting out there.",
            "Nobody goes out this gate after dusk with fewer than two torches. Not on my watch, and it is always my watch.",
        ],
    },
    Folk {
        name: "Watchman Tolly",
        role: "Watchman",
        lines: &[
            "Orrin was my bunkmate. They sent him after the four, and now his name is on the board with theirs.",
            "The sergeant wants anyone who can hold a blade. You are holding one. The lodge is south-east, inside the wall.",
            "If you hear counting in the trees east of here, do not answer it. Do not stop to work out the number.",
            "The board is not for reading. It is for remembering. Go on, then, read it.",
        ],
    },
];

const _: () = assert!(FOLK_VILLAGERS < FOLK.len());
const _: () = assert!(KEPT_PERSONA == FOLK_VILLAGERS);
