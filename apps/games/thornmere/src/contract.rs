#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Channel {
    #[default]
    General,
    Trade,
    Lfg,
    Whisper,
    Say,
    System,
    Combat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatLine {
    pub channel: Channel,
    pub speaker: Option<String>,
    pub text: String,
}
