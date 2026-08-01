use crate::resources::Readout;

pub fn line(readout: &Readout, rate: f32) -> String {
    let mut parts = Vec::with_capacity(readout.notes.len() + 2);
    parts.push(readout.title.clone());
    for note in &readout.notes {
        parts.push(note.clone());
    }
    parts.push(format!("{rate:.0} fps"));
    parts.join(" - ")
}
