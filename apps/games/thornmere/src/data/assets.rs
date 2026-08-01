pub const ITEM_ICON_COUNT: usize = 24;

pub static ITEM_ICONS: [(&str, &[u8]); ITEM_ICON_COUNT] = [
    ("award", include_bytes!("../../assets/ui/items/award.png")),
    (
        "book_closed",
        include_bytes!("../../assets/ui/items/book_closed.png"),
    ),
    ("bow", include_bytes!("../../assets/ui/items/bow.png")),
    (
        "character",
        include_bytes!("../../assets/ui/items/character.png"),
    ),
    (
        "crown_a",
        include_bytes!("../../assets/ui/items/crown_a.png"),
    ),
    (
        "crown_b",
        include_bytes!("../../assets/ui/items/crown_b.png"),
    ),
    (
        "dice_sword",
        include_bytes!("../../assets/ui/items/dice_sword.png"),
    ),
    ("fire", include_bytes!("../../assets/ui/items/fire.png")),
    (
        "flask_full",
        include_bytes!("../../assets/ui/items/flask_full.png"),
    ),
    (
        "flask_half",
        include_bytes!("../../assets/ui/items/flask_half.png"),
    ),
    (
        "hourglass",
        include_bytes!("../../assets/ui/items/hourglass.png"),
    ),
    (
        "notepad",
        include_bytes!("../../assets/ui/items/notepad.png"),
    ),
    ("pouch", include_bytes!("../../assets/ui/items/pouch.png")),
    (
        "resource_apple",
        include_bytes!("../../assets/ui/items/resource_apple.png"),
    ),
    (
        "resource_iron",
        include_bytes!("../../assets/ui/items/resource_iron.png"),
    ),
    (
        "resource_wheat",
        include_bytes!("../../assets/ui/items/resource_wheat.png"),
    ),
    (
        "resource_wood",
        include_bytes!("../../assets/ui/items/resource_wood.png"),
    ),
    (
        "rhombus",
        include_bytes!("../../assets/ui/items/rhombus.png"),
    ),
    ("shield", include_bytes!("../../assets/ui/items/shield.png")),
    ("skull", include_bytes!("../../assets/ui/items/skull.png")),
    (
        "structure_tower",
        include_bytes!("../../assets/ui/items/structure_tower.png"),
    ),
    (
        "suit_diamonds",
        include_bytes!("../../assets/ui/items/suit_diamonds.png"),
    ),
    ("sword", include_bytes!("../../assets/ui/items/sword.png")),
    ("token", include_bytes!("../../assets/ui/items/token.png")),
];
