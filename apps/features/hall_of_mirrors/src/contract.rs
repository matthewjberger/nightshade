#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PortalSide {
    #[default]
    Blue,
    Orange,
}

impl PortalSide {
    pub const BOTH: [PortalSide; 2] = [PortalSide::Blue, PortalSide::Orange];

    pub fn index(self) -> usize {
        match self {
            PortalSide::Blue => 0,
            PortalSide::Orange => 1,
        }
    }
}
