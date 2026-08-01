#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Sky {
    #[default]
    Clear,
    Cloudy,
    Rain,
    Storm,
}
