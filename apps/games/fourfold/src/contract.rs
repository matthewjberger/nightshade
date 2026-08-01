#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dimension {
    #[default]
    Hub,
    Glass,
    Loom,
    Cairn,
    Hearth,
}

impl Dimension {
    pub fn name(self) -> &'static str {
        match self {
            Dimension::Hub => "HUB",
            Dimension::Glass => "GLASS",
            Dimension::Loom => "LOOM",
            Dimension::Cairn => "CAIRN",
            Dimension::Hearth => "HEARTH",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Glyph {
    #[default]
    Bar,
    Fork,
    Ring,
    Wedge,
    Cross,
    Comb,
}

impl Glyph {
    pub const ALL: [Glyph; 6] = [
        Glyph::Bar,
        Glyph::Fork,
        Glyph::Ring,
        Glyph::Wedge,
        Glyph::Cross,
        Glyph::Comb,
    ];

    pub fn next(self) -> Glyph {
        let index = Glyph::ALL.iter().position(|found| *found == self);
        Glyph::ALL[(index.unwrap_or(0) + 1) % Glyph::ALL.len()]
    }

    pub fn name(self) -> &'static str {
        match self {
            Glyph::Bar => "BAR",
            Glyph::Fork => "FORK",
            Glyph::Ring => "RING",
            Glyph::Wedge => "WEDGE",
            Glyph::Cross => "CROSS",
            Glyph::Comb => "COMB",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Opening,
    Entered,
    Solved,
    Returned,
    NearDials,
    Turned,
    Opened,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FourfoldEvent {
    Crossed(Dimension),
    Turned { dial: usize, glyph: Glyph },
    Opened,
}
