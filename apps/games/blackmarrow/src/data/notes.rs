pub struct Note {
    pub title: &'static str,
    pub lines: &'static [&'static str],
}

pub const NOTE_ROWS: usize = 12;

pub const NOTES: &[Note] = &[Note {
    title: "Shift log, last page",
    lines: &[
        "Day nine. Pressure in the main is still falling and the relay",
        "to the surface has not answered since Tuesday. Halvard went",
        "up the shaft with the spare cell and the good torch. He did",
        "not come back down and the ladder is iced over again.",
        "",
        "Something came down instead. I heard it in the pump room at",
        "four, claws on the grating, and I have shut every door I can",
        "reach. The shaft door is locked and the key is in the store,",
        "in the drawer, where I put it so that I would not use it.",
        "",
        "If you are reading this, keep the torch low and do not run.",
        "It hunts by sound.",
    ],
}];
