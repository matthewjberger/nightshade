#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GridlockEvent {
    Boarded { car: String },
    Left { car: String },
    Entered { place: String },
    Exited { place: String },
}
