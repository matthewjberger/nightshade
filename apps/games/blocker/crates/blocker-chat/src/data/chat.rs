#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Channel {
    #[default]
    General,
    Trade,
    Lfg,
    Whisper,
    Say,
    System,
}

pub const CHANNELS: usize = 6;

pub const GATE_CALLER: &str = "Gatewarden Hesk";

impl Channel {
    pub fn index(self) -> usize {
        self as usize
    }
}

const _: () = assert!(
    Channel::System as usize + 1 == CHANNELS,
    "CHANNELS does not count the Channel variants"
);

pub struct Thread {
    pub channel: Channel,
    pub lines: &'static [&'static str],
}

pub struct ChatTab {
    pub name: &'static str,
    pub channels: &'static [Channel],
}

impl ChatTab {
    pub fn shows(&self, channel: Channel) -> bool {
        self.channels.contains(&channel)
    }
}

pub const CHAT_TABS: [ChatTab; 5] = [
    tab(
        "All",
        &[
            Channel::General,
            Channel::Trade,
            Channel::Lfg,
            Channel::Whisper,
            Channel::Say,
            Channel::System,
        ],
    ),
    tab(
        "General",
        &[Channel::General, Channel::Say, Channel::System],
    ),
    tab("Trade", &[Channel::Trade]),
    tab("LFG", &[Channel::Lfg]),
    tab("Whisper", &[Channel::Whisper]),
];

const fn tab(name: &'static str, channels: &'static [Channel]) -> ChatTab {
    ChatTab { name, channels }
}

pub const GREETING_WORDS: &[&str] = &[
    "hi",
    "hello",
    "hey",
    "yo",
    "hail",
    "greetings",
    "howdy",
    "heya",
    "sup",
];

pub const GREETING_REPLIES: &[&str] = &[
    "Well met, {player}.",
    "Hail, {player}. The road is long out here.",
    "{player}. Good to see a face that is not a wight.",
    "Morning, {player}. Keep your lantern lit.",
    "You are {player}, the one the watch spoke of.",
    "Hail. Mind the dark, {player}.",
    "Hello to you, {player}.",
    "{player}, is it? Stay out of the barrow after dusk.",
];

pub const GREETING_ASIDES: &[&str] = &[
    "Aye, well met.",
    "Hail, stranger.",
    "Keep your pick close.",
    "The road is quiet today.",
];

pub const KEYWORD_REPLIES: &[(&str, &str)] = &[
    (
        "quest",
        "The folk with a title over their heads have work for you.",
    ),
    (
        "work",
        "The folk with a title over their heads have work for you.",
    ),
    (
        "job",
        "The folk with a title over their heads have work for you.",
    ),
    (
        "help",
        "Mine with the pick, cut wood with the axe, and mind the dark.",
    ),
    (
        "gold",
        "Coin comes from the deep stone, if you dare to go down.",
    ),
    (
        "diamond",
        "The deepest stone hides the bright ones. Few come back up.",
    ),
    (
        "monster",
        "After dark the hills are not safe. Stay near a lantern.",
    ),
    (
        "night",
        "After dark the hills are not safe. Stay near a lantern.",
    ),
    (
        "village",
        "Follow the smoke. Where there is a hearth, there is a village.",
    ),
    (
        "home",
        "My home is small, but the door is always open to you.",
    ),
    (
        "weather",
        "The sky turns fast out here. Rain today, snow tomorrow.",
    ),
    (
        "secret",
        "They say the standing stones remember an old name. Speak it, and see.",
    ),
    (
        "stone",
        "The standing stones hum at dusk. Some say they listen.",
    ),
];

pub const SECRET_PHRASE: &str = "the stones remember";
pub const SECRET_REPLY: &str = "The standing stones hum. An old name is known to you now, and the hills know yours. (+50 renown)";
pub const SECRET_RENOWN: u32 = 50;

pub struct Calling {
    pub name: &'static str,
    pub role: &'static str,
}

pub const CLASSES: [Calling; 6] = [
    calling("warrior", "tank"),
    calling("ranger", "damage"),
    calling("cleric", "healer"),
    calling("mage", "damage"),
    calling("rogue", "damage"),
    calling("warden", "tank"),
];

const fn calling(name: &'static str, role: &'static str) -> Calling {
    Calling { name, role }
}

pub const MONSTERS: [&str; 10] = [
    "goblin",
    "bog lurker",
    "cave troll",
    "barrow wight",
    "frost wolf",
    "stone golem",
    "shade",
    "giant spider",
    "wraith",
    "ridge drake",
];

pub const PLACES: [&str; 8] = [
    "the standing stones",
    "the sunken mine",
    "the old barrow",
    "the deep road",
    "the frostwood",
    "the drowned vault",
    "the ridge caves",
    "the henge",
];

pub const LOOT: [&str; 8] = [
    "a rusted key",
    "an iron blade",
    "a healing draught",
    "a coil of rope",
    "a silver ring",
    "a worn map",
    "a bundle of torches",
    "a chunk of ore",
];

pub const QUEST_NAMES: [&str; 6] = [
    "Down the Deep Road",
    "The Barrow Below",
    "Stones That Hum",
    "Clear the Mine",
    "The Drowned Vault",
    "Wolves at the Gate",
];

pub const GUILDS: [&str; 6] = [
    "the Lantern Watch",
    "Ashen Company",
    "the Deepdelvers",
    "the Standing Stone Society",
    "the Free Blades",
    "the Hollow Kings",
];

pub const IDLE_THREADS: [Thread; 6] = [
    thread(
        Channel::General,
        &[
            "anyone up for {place}? need one more",
            "what are you farming down there {a}",
            "{monster}s drop {item} if you are lucky",
        ],
    ),
    thread(
        Channel::General,
        &[
            "wiped to a {monster} again",
            "pull slower {b}",
            "not my fault the healer was afk",
        ],
    ),
    thread(
        Channel::Trade,
        &[
            "WTS {item}, {price}, mail {a}",
            "{b}: too rich for me",
            "fine, {price} and it is yours",
        ],
    ),
    thread(
        Channel::Lfg,
        &[
            "LFG {quest}, have a tank and a healer",
            "{a}: invite me, {class} dps",
            "one more then we go",
        ],
    ),
    thread(
        Channel::General,
        &[
            "is {place} worth the walk?",
            "{c}: only for the {item}",
            "grabbing torches then",
        ],
    ),
    thread(
        Channel::General,
        &[
            "grats {a} on level {level}",
            "ty, the {monster}s gave good coin",
            "!{a} reaches level {level}",
        ],
    ),
];

pub const SINGLES: [Thread; 8] = [
    thread(Channel::General, &["cold night. good night to be indoors"]),
    thread(Channel::Trade, &["WTB {item}, will pay {price}"]),
    thread(Channel::Lfg, &["LFM {place} run, need a healer"]),
    thread(
        Channel::General,
        &["heard the {monster}s are thick near {place}"],
    ),
    thread(
        Channel::General,
        &["{guild} is recruiting, whisper an officer"],
    ),
    thread(Channel::Trade, &["selling {item}, cheaper than {guild}"]),
    thread(Channel::General, &["watch the road after dark"]),
    thread(
        Channel::Lfg,
        &["need one dps for {quest}, {price} for the trouble"],
    ),
];

pub const WHISPERS: [Thread; 5] = [
    thread(Channel::Whisper, &["you awake? something moved in {place}"]),
    thread(
        Channel::Whisper,
        &["do not tell anyone i found {item} down there"],
    ),
    thread(Channel::Whisper, &["meet me by the well, bring torches"]),
    thread(
        Channel::Whisper,
        &["the {monster} took the last party, be careful"],
    ),
    thread(
        Channel::Whisper,
        &["i saw lights under {place} again last night"],
    ),
];

const fn thread(channel: Channel, lines: &'static [&'static str]) -> Thread {
    Thread { channel, lines }
}

pub const SAY_LINES: [&str; 8] = [
    "Well met.",
    "Cold one today.",
    "Mind the road after dark.",
    "The well is that way.",
    "Safe travels.",
    "Long day ahead.",
    "You look far from home.",
    "Stay for the fire.",
];

pub const VILLAGE_WELCOME: [&str; 6] = [
    "Smoke rises from the chimneys.",
    "Lanterns swing at the crossroads.",
    "The well stands at the heart of it.",
    "Fields run out to the treeline.",
    "A quiet place, for now.",
    "Voices carry from the square.",
];

pub const GATE_DAWN: [&str; 2] = [
    "Gate is open. The board still reads the same three names.",
    "Day watch. The road east is clear as far as the henge and no farther.",
];

pub const GATE_DUSK: [&str; 2] = [
    "The gate stays open, but nobody goes out to the barrow in the dark. Sergeant's orders.",
    "Torches on the wall. If you are going east tonight, go with two.",
];

pub const DAWN_LINES: [&str; 3] = [
    "Dawn breaks over the land.",
    "The sun climbs and the fog thins.",
    "Somewhere down the valley, a cock crows.",
];

pub const DUSK_LINES: [&str; 3] = [
    "Night falls. The lanterns come up.",
    "The sun sinks behind the hills.",
    "Shadows pool in the low ground.",
];

pub const CHAT_ROWS: usize = 8;
