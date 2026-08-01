use crate::resources::Readout;

pub fn say(readout: &mut Readout, note: impl Into<String>) {
    readout.notes.push(note.into());
}

pub fn clear_notes(readout: &mut Readout) {
    readout.notes.clear();
}
