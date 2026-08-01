use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::items::Item;
use blocker_screen::data::names::same_name;

pub const TRADERS: [&str; 4] = ["Peddler", "Quartermaster", "Stonemason", "Lampwright"];

pub struct Good {
    pub item: Item,
    pub count: u32,
    pub price: u32,
}

pub const STOCK: [Good; 9] = [
    good(Item::Block(BlockKind::Torch), 4, 8),
    good(Item::Block(BlockKind::Glowstone), 1, 20),
    good(Item::Block(BlockKind::Glass), 4, 12),
    good(Item::Block(BlockKind::Bricks), 4, 16),
    good(Item::Block(BlockKind::WoolWhite), 2, 10),
    good(Item::Block(BlockKind::WoolRed), 2, 10),
    good(Item::Block(BlockKind::WoolBlue), 2, 10),
    good(Item::Block(BlockKind::Obsidian), 1, 30),
    good(Item::Block(BlockKind::Signpost), 1, 6),
];

const fn good(item: Item, count: u32, price: u32) -> Good {
    Good { item, count, price }
}

pub struct Giver {
    pub name: &'static str,
    pub title: &'static str,
}

pub const WATCH_GIVER: usize = giver_named("Sergeant Ovilda");

pub const GIVERS: [Giver; 14] = [
    giver("Marren", "Hunter"),
    giver("Holt", "Farmer"),
    giver("Garron", "Blacksmith"),
    giver("Wynne", "Apothecary"),
    giver("Bram", "Innkeeper"),
    giver("Mabb", "Prospector"),
    giver("Idris", "Watch Captain"),
    giver("Ael", "Priestess"),
    giver("Fenn", "Trader"),
    giver("Orla", "Banker"),
    giver("Pell", "Dockmaster"),
    giver("Rook", "Stablemaster"),
    giver("Wenna", "Grocer"),
    giver("Sergeant Ovilda", "Watch Sergeant"),
];

const fn giver(name: &'static str, title: &'static str) -> Giver {
    Giver { name, title }
}

pub const fn giver_named(name: &str) -> usize {
    let mut index = 0;
    while index < GIVERS.len() {
        if same_name(GIVERS[index].name, name) {
            return index;
        }
        index += 1;
    }
    panic!("the givers table holds no one by that name")
}
