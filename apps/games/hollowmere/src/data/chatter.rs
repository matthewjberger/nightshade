use crate::contract::Channel;

pub struct Thread {
    pub channel: Channel,
    pub lines: &'static [&'static str],
}

pub const ITEMS: [&str; 12] = [
    "Boar Hide",
    "Wolf Pelt",
    "Bandit Coinpurse",
    "Barrow Dust",
    "Bitterroot Bundle",
    "Rusty Toll Blade",
    "Old Tusk's Tusk",
    "Gravelord's Crown",
    "Linen Bandage",
    "Mirror Lake Trout",
    "Copper Lantern",
    "Thornmere Ale",
];

pub const IDLE_THREADS: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &[
            "anyone know where {monster} spawn?",
            "{place}, follow the road and turn off at the stream",
            "ty",
            "np, watch the pairs",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "is {monster} up right now?",
            "was up ten minutes ago, someone probably tagged it",
            "of course they did",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "how do I get to {place} from town",
            "down the hill, keep left at the field",
            "left at the field or left at the board?",
            "the field. the board is in town {b}",
            "ok found it thx",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "first time in thornmere, hi all",
            "welcome, boars are your friend until level 5",
            "and then they are your enemy",
            "they were always my enemy",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "who keeps pulling the whole camp at {place}",
            "not me",
            "it was {c}, saw it",
            "it was an accident ok",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "the rare at the barrow is camped again",
            "it is always camped",
            "I have been waiting an hour",
            "welcome to thornmere",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "this hill is a workout",
            "imagine doing it without a mount",
            "we are all doing it without a mount {b}",
            "oh right",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "did the board take anyone else's turn in twice?",
            "no it takes mine once and then forgets",
            "classic board",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "any healers on? {place} is rough solo",
            "I can come after this boar",
            "ty {b} you are a saint",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "what does bitterroot even do",
            "the apothecary makes a salve",
            "for what",
            "for the bitterroot rash",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "the lake is so pretty at night",
            "until you see what is under it",
            "what is under it",
            "fish {c}. it is fish",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "level {level} and still dying to boars",
            "boars do not care about levels",
            "boars only care about crops",
        ],
    },
    Thread {
        channel: Channel::Trade,
        lines: &[
            "WTS [{item}] {price}",
            "{price}? lol",
            "fine, half that",
            "deal, meet at the board",
        ],
    },
    Thread {
        channel: Channel::Trade,
        lines: &[
            "WTB [{item}], paying well",
            "how well",
            "well enough",
            "that is not a number {a}",
        ],
    },
    Thread {
        channel: Channel::Trade,
        lines: &[
            "selling stacks of boar meat, pst",
            "who buys boar meat",
            "the inn does, cheaper than mine though",
        ],
    },
    Thread {
        channel: Channel::Lfg,
        lines: &["LFG {quest}", "inv", "grp is full sry", "it was never full"],
    },
    Thread {
        channel: Channel::Lfg,
        lines: &[
            "LFM barrow run, need healer",
            "healer here, {level} ok?",
            "perfect, summon at the road",
            "we cannot summon {a}",
            "walk then",
        ],
    },
    Thread {
        channel: Channel::Lfg,
        lines: &[
            "anyone want to duo the toll camp",
            "sure, I am at the field",
            "meet you at the fork",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "<{guild}> is recruiting, friendly leveling guild, pst",
            "do you have a tabard",
            "we have a colour",
            "sold",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "*polishes blade* a fine evening for the hunt",
            "please stop",
            "*sheathes blade*",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "Idris gave me the toll road quest, four bandits, is that hard",
            "they hit hard but they run at half health",
            "cowards",
            "smart cowards",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "Sister Ael told me not to read the stones in the barrow",
            "so did you",
            "obviously",
            "what did they say",
            "I am not allowed to tell you",
        ],
    },
];

pub const SINGLES: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &["brb"],
    },
    Thread {
        channel: Channel::General,
        lines: &["lol"],
    },
    Thread {
        channel: Channel::General,
        lines: &["the road is packed tonight"],
    },
    Thread {
        channel: Channel::General,
        lines: &["anyone seen a rare up?"],
    },
    Thread {
        channel: Channel::General,
        lines: &["Marren has good quests, do them before you leave the wood"],
    },
    Thread {
        channel: Channel::General,
        lines: &["Bram's ale is half water, pass it on"],
    },
    Thread {
        channel: Channel::General,
        lines: &["is it just me or is {place} darker than it used to be"],
    },
    Thread {
        channel: Channel::General,
        lines: &["I keep getting lost behind the chapel"],
    },
    Thread {
        channel: Channel::Trade,
        lines: &["WTS [{item}] {price} or best offer"],
    },
    Thread {
        channel: Channel::Trade,
        lines: &["WTB [{item}] x5, pst"],
    },
    Thread {
        channel: Channel::Trade,
        lines: &["selling [{item}], cheap, need the gold for a mount one day"],
    },
    Thread {
        channel: Channel::Lfg,
        lines: &["LFG {quest}, I am at the board"],
    },
    Thread {
        channel: Channel::Lfg,
        lines: &["LF1M {monster} farming, {place}"],
    },
    Thread {
        channel: Channel::General,
        lines: &["gz to whoever just dinged"],
    },
    Thread {
        channel: Channel::General,
        lines: &["the sunset over the lake is unreal right now"],
    },
    Thread {
        channel: Channel::General,
        lines: &["wolves in pairs, boars in crops, bandits in denial"],
    },
    Thread {
        channel: Channel::Lfg,
        lines: &["{class} {level} lf group for {place}, can {role}"],
    },
    Thread {
        channel: Channel::General,
        lines: &["is {class} any good at the barrow or should I just bring a {role}"],
    },
];

pub const CAMP_LINES: [&str; 6] = [
    "off to {place}",
    "{monster} time",
    "heading to {place} for the six",
    "pulling {monster} on the east side, come if you want",
    "going to farm {monster} for a bit",
    "{place} again. my life is {place}",
];

pub const RETURN_LINES: [&str; 5] = [
    "six down, heading back",
    "quest done, running back up the hill",
    "back to the board",
    "got my {monster}, going up",
    "turn in time",
];

pub const TURN_IN_LINES: [&str; 5] = [
    "turned in, again?",
    "another {quest} done",
    "handed in, back down",
    "the board paid. the board always pays",
    "quest complete, coins in the bag",
];

pub const DEATH: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &["died to a {monster} lol", "F", "rip {a}"],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "{monster} hits harder than they look",
            "they look like they hit hard {a}",
            "fair",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &["rezzed at the board again", "again?", "third time today"],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "I was afk and a {monster} found me",
            "the {monster} always finds you",
        ],
    },
];

pub const DING: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &["ding {level}", "gz", "gz!", "ty ty"],
    },
    Thread {
        channel: Channel::General,
        lines: &["level {level}!", "grats", "gz {a}", "thanks all"],
    },
    Thread {
        channel: Channel::General,
        lines: &["ding! {level}", "nice", "gz"],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "{level} on the {class}, finally",
            "gz, {class} gets good soon",
            "define soon",
            "level 60 {b}",
        ],
    },
];

pub const BOSS_DOWN: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &[
            "!{killer} has slain {monster}!",
            "gz {killer}",
            "was that solo??",
            "finally, it has been up for an hour",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "!{killer} has slain {monster}!",
            "nice",
            "gz",
            "what dropped",
        ],
    },
];

pub const PLAYER_DING: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &["gz {player}", "gz", "grats {player}"],
    },
    Thread {
        channel: Channel::General,
        lines: &["{player} dinged, gz", "gz!"],
    },
];

pub const PLAYER_DEATH: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &[
            "did {player} just die to a {monster}",
            "everyone dies to one once",
            "F",
        ],
    },
    Thread {
        channel: Channel::General,
        lines: &["F for {player}", "rip"],
    },
];

pub const LOGOFF: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &["logging off, night all", "night {a}", "gn"],
    },
    Thread {
        channel: Channel::General,
        lines: &["off to bed, save me a boar", "there are always boars {a}"],
    },
    Thread {
        channel: Channel::General,
        lines: &["gtg, cya tomorrow"],
    },
];

pub const LOGON: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &["hey all", "wb {a}", "hey"],
    },
    Thread {
        channel: Channel::General,
        lines: &["morning thornmere", "morning", "is the rare up"],
    },
    Thread {
        channel: Channel::General,
        lines: &["back, what did I miss", "{c} died to a boar", "classic"],
    },
    Thread {
        channel: Channel::General,
        lines: &[
            "back on the {class}, anyone need a {role}?",
            "always",
            "omw",
        ],
    },
];

pub const WHISPERS: &[Thread] = &[
    Thread {
        channel: Channel::Whisper,
        lines: &[
            "hey, you new here? boars are down the road from town",
            "the board gives you a quest for them",
            "ok good luck",
        ],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &[
            "want to group for {quest}? I am at the board",
            "hello?",
            "ok nvm",
        ],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &["nice name"],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &[
            "if you see {monster} up, tell me pls",
            "I have been camping it for two days",
        ],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &[
            "do you know where {monster} spawn?",
            "nvm found them",
            "ty anyway",
        ],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &["sry wrong person"],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &[
            "lf healer for the barrow, you a healer?",
            "you look like a healer from here",
            "ok you are not a healer",
        ],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &["<{guild}> is recruiting if you want a guild", "no pressure"],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &["you dropped something at the board", "jk"],
    },
    Thread {
        channel: Channel::Whisper,
        lines: &["are you the one that killed {monster}?", "gz if so"],
    },
];

pub const BOSS_UP: &[Thread] = &[
    Thread {
        channel: Channel::General,
        lines: &["{monster} is up at {place}", "omw", "save it for me"],
    },
    Thread {
        channel: Channel::General,
        lines: &["{monster} UP", "everyone to {place}", "not again"],
    },
];

pub const DISCOVERY_LINES: [&str; 12] = [
    "Thornmere. Walls, a moat, a keep, and every road in the vale starts at its gate.",
    "The Boar Field. The crops are lost, the boars are not.",
    "Wolf Wood. The trees are close and the pairs are closer.",
    "Redharrow Fort. The bandits call it a road tax. The palisade calls it home.",
    "The Barrow. The stones are older than the city and they know it.",
    "Mirror Lake. Still water and a lot of trout.",
    "Holt's Farm. Fences, hay, and a farmer who owes the innkeeper.",
    "The Sunken Crypt. Stairs under the barrow, and a cold that is not the weather.",
    "Ironjaw Mine. The headframe still stands. The kobolds hold everything under it.",
    "Mirror Harbor. Docks, a moored ship, and a dockmaster who will sell you a boat.",
    "Ironjaw Deep. The bottom of the pit, where the foreman keeps the best ore.",
    "Fallowmere Wood. Oaks, a winding road, an inn at the crossroads, and wolves that are still learning.",
];
