#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrosstownCommand {
    Punch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrosstownEvent {
    Landed { down: bool },
    Cleared,
}

#[derive(Default)]
pub struct CrosstownCommands {
    pub pending: Vec<CrosstownCommand>,
}
