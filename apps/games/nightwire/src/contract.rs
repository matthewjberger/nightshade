#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NightwireCommand {
    BeginShift,
    Reply { index: usize },
    RaiseCard { index: usize },
    Dispatch,
    Hold,
    Advance,
    ReturnToTitle,
    LeaveStation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NightwireEvent {
    CardFound { index: usize },
    CallClosed { index: usize },
    ShiftEnded,
}

#[derive(Default)]
pub struct NightwireCommands {
    pub pending: Vec<NightwireCommand>,
}
