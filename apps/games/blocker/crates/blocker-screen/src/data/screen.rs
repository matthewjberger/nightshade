#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Screen {
    #[default]
    Setup,
    Title,
    NewWorld,
    Loading,
    Playing,
    Inventory,
    Crafting,
    Settings,
    Chest,
    Map,
    Character,
    Journal,
    Dialogue,
    Shop,
}
