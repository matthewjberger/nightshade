#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Joint {
    #[default]
    ArmLeft,
    ArmRight,
    LegLeft,
    LegRight,
}

impl Joint {
    pub fn offset(self) -> f32 {
        match self {
            Self::LegLeft | Self::ArmRight => 0.0,
            Self::LegRight | Self::ArmLeft => std::f32::consts::PI,
        }
    }

    pub fn arm(self) -> bool {
        matches!(self, Self::ArmLeft | Self::ArmRight)
    }

    pub fn left(self) -> bool {
        matches!(self, Self::ArmLeft | Self::LegLeft)
    }
}
