#[derive(Clone, Debug, PartialEq)]
pub enum TidefallEvent {
    Landed { body: String },
    Lifted { body: String },
    Disembarked { body: String },
    Boarded,
}
