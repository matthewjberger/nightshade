use crate::resources::Conversation;

pub fn open(conversation: &mut Conversation, speaker: impl Into<String>, lines: &[&str]) {
    conversation.speaker = speaker.into();
    conversation.waiting = lines.iter().map(|line| line.to_string()).collect();
    conversation.line.clear();
    conversation.revealed = 0.0;
    conversation.replies.clear();
    conversation.chosen = None;
    step(conversation);
}

pub fn step(conversation: &mut Conversation) {
    match conversation.waiting.pop_front() {
        Some(line) => {
            conversation.line = line;
            conversation.revealed = 0.0;
        }
        None => close(conversation),
    }
}

pub fn reveal_all(conversation: &mut Conversation) {
    conversation.revealed = conversation.line.chars().count() as f32;
}

pub fn offer(conversation: &mut Conversation, replies: &[&str]) {
    conversation.replies = replies.iter().map(|reply| reply.to_string()).collect();
}

pub fn choose(conversation: &mut Conversation, index: usize) {
    if index < conversation.replies.len() {
        conversation.chosen = Some(index);
    }
}

pub fn close(conversation: &mut Conversation) {
    conversation.speaker.clear();
    conversation.line.clear();
    conversation.waiting.clear();
    conversation.revealed = 0.0;
}
