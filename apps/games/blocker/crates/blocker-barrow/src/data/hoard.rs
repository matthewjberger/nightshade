use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::items::Item;
use blocker_blocks::data::tools::ToolKind;

pub const HOARD: [(Item, u32); 5] = [
    (Item::Tool(ToolKind::Pickaxe), 1),
    (Item::Block(BlockKind::GoldOre), 4),
    (Item::Block(BlockKind::IronOre), 6),
    (Item::Block(BlockKind::Glowstone), 2),
    (Item::Block(BlockKind::Torch), 12),
];
