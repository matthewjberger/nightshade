use std::collections::VecDeque;

#[derive(Default)]
pub struct Conversation {
    pub speaker: String,
    pub waiting: VecDeque<String>,
    pub line: String,
    pub revealed: f32,
    pub replies: Vec<String>,
    pub chosen: Option<usize>,
}

pub struct DialogueSettings {
    pub letters_a_second: f32,
}

impl Default for DialogueSettings {
    fn default() -> Self {
        Self {
            letters_a_second: 45.0,
        }
    }
}
