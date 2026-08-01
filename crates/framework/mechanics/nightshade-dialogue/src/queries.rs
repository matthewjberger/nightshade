use crate::resources::Conversation;

pub fn shown(conversation: &Conversation) -> String {
    conversation
        .line
        .chars()
        .take(conversation.revealed as usize)
        .collect()
}

pub fn fully_shown(conversation: &Conversation) -> bool {
    conversation.revealed as usize >= conversation.line.chars().count()
}

pub fn talking(conversation: &Conversation) -> bool {
    !conversation.line.is_empty() || !conversation.waiting.is_empty()
}

pub fn last_line(conversation: &Conversation) -> bool {
    conversation.waiting.is_empty()
}
