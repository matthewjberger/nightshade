pub struct BootLine {
    pub text: &'static str,
    pub after: f32,
}

const fn out(text: &'static str, after: f32) -> BootLine {
    BootLine { text, after }
}

pub const LINES: &[BootLine] = &[
    out("HOLLOWAY & SONS   WORKSTATION 14", 0.24),
    out("BIOS v2.04   14 jan 2016", 0.16),
    out("", 0.18),
    out("MEMORY  ...............  16384K OK", 0.30),
    out("DRIVES  ...............  ST3120026A  120GB", 0.34),
    out("", 0.22),
    out("Disk 0   last write   09 nov 2021, 03:07", 0.40),
    out("Disk 0   wipe on record   none", 0.46),
    out("", 0.26),
    out("Mounting /home  .......  OK", 0.34),
    out("  guest", 0.13),
    out("  r.mears        retained", 0.15),
    out("  d.okonkwo      retained", 0.15),
    out("  j.alderete     retained", 0.15),
    out("", 0.26),
    out("  network  .............  OK", 0.22),
    out("  iam client  ..........  OK", 0.26),
    out("  session  .............  OPEN", 0.34),
    out("  close handler  .......  not configured", 0.52),
    out("", 0.34),
    out("Ready.", 0.70),
];

pub fn duration() -> f32 {
    LINES.iter().map(|line| line.after).sum()
}

pub fn shown(elapsed: f32) -> String {
    let mut at = 0.0;
    let mut out = String::new();
    for line in LINES {
        at += line.after;
        if at > elapsed {
            break;
        }
        out.push_str(line.text);
        out.push('\n');
    }
    out
}
