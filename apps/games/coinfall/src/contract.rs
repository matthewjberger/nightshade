#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoinfallEvent {
    CoinTaken,
    WalkerStomped,
    WalkerHit,
    FlagReached,
}
