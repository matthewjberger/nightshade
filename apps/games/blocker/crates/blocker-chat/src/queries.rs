use crate::data::chat::Channel;
use crate::resources::ChatLine;

pub fn formatted(line: &ChatLine) -> String {
    let speaker = line.speaker.as_deref().unwrap_or("");
    match line.channel {
        Channel::General => format!("[General] {speaker}: {}", line.text),
        Channel::Trade => format!("[Trade] {speaker}: {}", line.text),
        Channel::Lfg => format!("[LFG] {speaker}: {}", line.text),
        Channel::Whisper => format!("{speaker} whispers: {}", line.text),
        Channel::Say => {
            if speaker.is_empty() {
                line.text.clone()
            } else {
                format!("{speaker}: {}", line.text)
            }
        }
        Channel::System => line.text.clone(),
    }
}
