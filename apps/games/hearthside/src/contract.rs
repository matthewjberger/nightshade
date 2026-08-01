#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HearthsideCommand {
    Grant { amount: u32 },
    FillNeeds,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HearthsideEvent {
    Granted { amount: u32 },
    NeedsFilled,
}

#[derive(Default)]
pub struct HearthsideCommands {
    pub pending: Vec<HearthsideCommand>,
}
