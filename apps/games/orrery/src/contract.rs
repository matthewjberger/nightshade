#[derive(Clone, Debug, PartialEq)]
pub enum OrreryCommand {
    LeaveShip,
    EnterShip,
    MatchSpeed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrreryEvent {
    Landed { body: usize },
    Launched,
    SignalFound { body: usize },
}

#[derive(Default)]
pub struct OrreryCommands {
    pub pending: Vec<OrreryCommand>,
}
