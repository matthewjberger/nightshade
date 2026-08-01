#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    #[default]
    ShaftKey,
    Cell,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fixture {
    #[default]
    Loose,
    Door,
    Drawer,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum HuntState {
    #[default]
    Dormant,
    Searching,
    Hunting,
    Feeding,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Surface {
    #[default]
    Rock,
    Wall,
    Floor,
    Ceiling,
    Plate,
    Metal,
    Rust,
    Door,
    Drawer,
    Grille,
}

pub fn item_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::ShaftKey => "shaft key",
        ItemKind::Cell => "spare cell",
    }
}
