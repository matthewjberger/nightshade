use nightshade::prelude::nalgebra_glm::Vec4;

pub type Fact = &'static str;

#[derive(Debug)]
pub struct Condition {
    pub requires: &'static [Fact],
    pub forbids: &'static [Fact],
}

pub const ALWAYS: Condition = Condition {
    requires: &[],
    forbids: &[],
};

pub fn condition_holds(condition: &Condition, facts: &[Fact]) -> bool {
    condition.requires.iter().all(|fact| facts.contains(fact))
        && !condition.forbids.iter().any(|fact| facts.contains(fact))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Presence {
    #[default]
    Offline,
    Idle,
    Online,
}

pub fn presence_label(presence: Presence) -> &'static str {
    match presence {
        Presence::Offline => "offline",
        Presence::Idle => "idle",
        Presence::Online => "online",
    }
}

pub struct Line {
    pub text: &'static str,
    pub rhythm: &'static [f32],
}

pub fn sent_program(text: &str) -> Option<&'static str> {
    let name = text.strip_prefix("store:")?.trim();
    crate::data::programs::PROGRAMS
        .iter()
        .find(|program| program.id == name)
        .map(|program| program.id)
}

pub fn segment_is_typing(segment: usize) -> bool {
    segment.is_multiple_of(2)
}

pub struct Choice {
    pub text: &'static str,
    pub goto: &'static str,
    pub tactic: Tactic,
    pub condition: Condition,
    pub asserts: &'static [Fact],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tactic {
    #[default]
    Plain,
    Lie,
    Persuade,
    Press,
}

pub fn tactic_fails(
    tactic: Tactic,
    bot: bool,
    gave: bool,
    spent: bool,
    used: bool,
) -> Option<&'static str> {
    match tactic {
        Tactic::Plain => None,
        Tactic::Lie if bot => Some("it only matches strings. there is nothing in it to mislead"),
        Tactic::Lie if gave => Some("they have already given you something"),
        Tactic::Press if bot => Some("it has nothing to add"),
        Tactic::Press if spent => Some("they have already finished"),
        Tactic::Persuade if bot => Some("it has nothing of its own to give"),
        Tactic::Persuade if used => Some("it does not work twice on the same person"),
        _ => None,
    }
}

pub fn tactic_lesson(tactic: Tactic) -> Option<Fact> {
    match tactic {
        Tactic::Plain => None,
        Tactic::Lie => Some("knows_when_lying_fails"),
        Tactic::Press => Some("knows_when_pressing_fails"),
        Tactic::Persuade => Some("knows_when_persuading_fails"),
    }
}

pub fn tactic_label(tactic: Tactic) -> &'static str {
    match tactic {
        Tactic::Plain => "",
        Tactic::Lie => "[LIE] ",
        Tactic::Persuade => "[PERSUADE] ",
        Tactic::Press => "[PRESS] ",
    }
}

pub const WILL_NOT_WORK: &str = "[WILL NOT WORK] ";

#[derive(Debug)]
pub enum Effect {
    Reveal(&'static str),
    Invite(&'static str, &'static str),
    SetPresence(&'static str, Presence),
    Message(&'static str, &'static str),
    RevealServer(&'static str),
    Deactivate(&'static str),
    Reactivate(&'static str),
    Automate(&'static str),
    Assert(Fact),
    TakeHandle(&'static str),
    Ending,
    Finish(crate::data::endings::Ending),
    Seize,
    Noticed,
}

pub struct DialogueNode {
    pub id: &'static str,
    pub condition: Condition,
    pub lines: &'static [Line],
    pub effects: &'static [Effect],
    pub choices: &'static [Choice],
}

pub struct ContactDefinition {
    pub id: &'static str,
    pub display_name: &'static str,
    pub handle: &'static str,
    pub status_line: &'static str,
    pub accent: Vec4,
    pub presence: Presence,
    pub bot: bool,
    pub listed: bool,
    pub entry: &'static str,
    pub history: &'static [HistoryLine],
    pub nodes: &'static [DialogueNode],
}

pub struct HistoryLine {
    pub mine: bool,
    pub stamp: &'static str,
    pub text: &'static str,
}

#[derive(Debug)]
pub struct Post {
    pub author: &'static str,
    pub text: &'static str,
    pub night: usize,
    pub delay_seconds: f32,
    pub condition: Condition,
}

pub struct Question {
    pub text: &'static str,
    pub tactic: Tactic,
    pub condition: Condition,
    pub asserts: &'static [Fact],
    pub effects: &'static [Effect],
    pub replies: &'static [Post],
}

pub struct ChannelDefinition {
    pub name: &'static str,
    pub topic: &'static str,
    pub on_open: &'static [Effect],
    pub posts: &'static [Post],
    pub questions: &'static [Question],
}

pub fn available_questions(channel: &ChannelDefinition, facts: &[Fact]) -> Vec<usize> {
    channel
        .questions
        .iter()
        .enumerate()
        .filter(|(_, question)| condition_holds(&question.condition, facts))
        .map(|(index, _)| index)
        .collect()
}

pub struct ServerDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub topic: &'static str,
    pub accent: Vec4,
    pub listed: bool,
    pub channels: &'static [ChannelDefinition],
}

pub struct NightDefinition {
    pub player_status: &'static str,
    pub day: &'static str,
    pub date: &'static str,
    pub opening: &'static [Effect],
    pub condition: Condition,
    pub locked_hints: &'static [Hint],
}

pub struct Hint {
    pub condition: Condition,
    pub text: &'static str,
}

pub fn resolve_hint(night: &NightDefinition, facts: &[Fact]) -> &'static str {
    night
        .locked_hints
        .iter()
        .find(|hint| condition_holds(&hint.condition, facts))
        .map(|hint| hint.text)
        .unwrap_or_default()
}

pub struct Script {
    pub player_handle: &'static str,
    pub contacts: &'static [ContactDefinition],
    pub authors: &'static [(&'static str, Vec4)],
    pub servers: &'static [ServerDefinition],
    pub nights: &'static [NightDefinition],
    pub instant: &'static [&'static str],
    pub instant_until: &'static [(&'static str, usize)],
}

pub const HOST: &str = "iambot";

pub const INSTANT_RATE: f32 = 8.0;

pub fn types_instantly(script: &Script, id: &str, night: usize) -> bool {
    script.instant.contains(&id)
        || script
            .instant_until
            .iter()
            .any(|(handle, stops)| *handle == id && night < *stops)
}

pub fn stopped_being_a_program(script: &Script, id: &str, night: usize) -> bool {
    script
        .instant_until
        .iter()
        .any(|(handle, stops)| *handle == id && night >= *stops)
}

pub fn avatar_letter(name: &str) -> String {
    name.chars()
        .find(|character| character.is_alphanumeric())
        .map(|character| character.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

pub fn speaker_color(script: &Script, author: &str) -> Vec4 {
    if let Some(definition) = script
        .contacts
        .iter()
        .find(|definition| definition.handle == author)
    {
        return definition.accent;
    }
    if let Some((_, color)) = script.authors.iter().find(|(name, _)| *name == author) {
        return *color;
    }
    author_color(author)
}

pub fn author_color(author: &str) -> Vec4 {
    const PALETTE: [Vec4; 6] = [
        Vec4::new(0.45, 0.44, 1.00, 1.0),
        Vec4::new(0.30, 0.80, 0.72, 1.0),
        Vec4::new(0.94, 0.45, 0.58, 1.0),
        Vec4::new(0.95, 0.70, 0.25, 1.0),
        Vec4::new(0.56, 0.84, 0.60, 1.0),
        Vec4::new(0.72, 0.60, 0.95, 1.0),
    ];
    let mut hash: u32 = 2166136261;
    for byte in author.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    PALETTE[hash as usize % PALETTE.len()]
}

pub fn contact_index(script: &Script, id: &str) -> Option<usize> {
    script
        .contacts
        .iter()
        .position(|definition| definition.id == id)
}

pub fn server_index(script: &Script, id: &str) -> Option<usize> {
    script
        .servers
        .iter()
        .position(|definition| definition.id == id)
}

pub fn resolve_node(definition: &ContactDefinition, id: &str, facts: &[Fact]) -> Option<usize> {
    definition
        .nodes
        .iter()
        .position(|node| node.id == id && condition_holds(&node.condition, facts))
}

pub fn available_choices(node: &DialogueNode, facts: &[Fact]) -> Vec<usize> {
    node.choices
        .iter()
        .enumerate()
        .filter(|(_, choice)| condition_holds(&choice.condition, facts))
        .map(|(index, _)| index)
        .collect()
}

pub fn validate(script: &Script) {
    let mut asserted: Vec<Fact> = established_elsewhere();
    for definition in script.contacts {
        for node in definition.nodes {
            collect_asserts(node.effects, &mut asserted);
            for choice in node.choices {
                asserted.extend(choice.asserts);
            }
        }
    }
    for night in script.nights {
        collect_asserts(night.opening, &mut asserted);
        asserted.push(night.day);
    }
    for definition in script.servers {
        for channel in definition.channels {
            collect_asserts(channel.on_open, &mut asserted);
            for question in channel.questions {
                asserted.extend(question.asserts);
                collect_asserts(question.effects, &mut asserted);
            }
        }
    }

    for definition in script.contacts {
        assert!(
            definition.entry.is_empty()
                || definition
                    .nodes
                    .iter()
                    .any(|node| node.id == definition.entry),
            "{} starts on {}, which is not one of their beats",
            definition.id,
            definition.entry,
        );
        for node in definition.nodes {
            check_reachable(
                &node.condition,
                &asserted,
                &format!("{}, beat {}", definition.id, node.id),
            );
            for choice in node.choices {
                assert!(
                    choice.goto.is_empty()
                        || definition.nodes.iter().any(|other| other.id == choice.goto),
                    "{}, beat {}: a reply jumps to {}, which is not one of their beats",
                    definition.id,
                    node.id,
                    choice.goto,
                );
                check_reachable(
                    &choice.condition,
                    &asserted,
                    &format!(
                        "{}, beat {}, reply \"{}\"",
                        definition.id, node.id, choice.text
                    ),
                );
            }
            for effect in node.effects {
                check_effect(
                    script,
                    effect,
                    &format!("{}, beat {}", definition.id, node.id),
                );
            }
        }
    }
    for night in script.nights {
        for effect in night.opening {
            check_effect(script, effect, night.day);
        }
        check_reachable(&night.condition, &asserted, night.day);
    }
    for definition in script.servers {
        for channel in definition.channels {
            let room = format!("{}, {}", definition.id, channel.name);
            for effect in channel.on_open {
                check_effect(script, effect, &room);
            }
            for question in channel.questions {
                let asked = format!("{room}, question \"{}\"", question.text);
                check_reachable(&question.condition, &asserted, &asked);
                for effect in question.effects {
                    check_effect(script, effect, &asked);
                }
                for post in question.replies {
                    check_reachable(&post.condition, &asserted, &asked);
                }
            }
            for post in channel.posts {
                check_reachable(
                    &post.condition,
                    &asserted,
                    &format!("{room}, post by {}", post.author),
                );
            }
        }
    }
}

fn check_reachable(condition: &Condition, asserted: &[Fact], whose: &str) {
    for fact in condition.requires {
        assert!(
            asserted.contains(fact),
            "{whose} waits on {fact}, which nothing establishes",
        );
    }
}

fn check_effect(script: &Script, effect: &Effect, whose: &str) {
    assert!(
        effect_is_valid(script, effect),
        "{whose} has an effect that names nothing: {effect:?}",
    );
}

pub const SYSTEM_FACTS: &[Fact] = &[
    "saw_offline_list",
    "opened_a_server",
    "opened_an_account",
    "burned_somebody",
    crate::data::lights::PLAYED,
    crate::data::lights::COUNTED,
    crate::data::lights::TURNED_ON,
];

pub fn established_elsewhere() -> Vec<Fact> {
    let mut facts: Vec<Fact> = Vec::from(SYSTEM_FACTS);
    for place in crate::data::office::PLACES {
        facts.extend(place.asserts);
    }
    for thing in crate::data::office::THINGS {
        facts.extend(thing.asserts);
    }
    for posted in crate::data::games::POSTED {
        facts.extend(posted.asserts);
    }
    for file in crate::data::files::LEFTOVERS {
        facts.extend(file.asserts);
    }
    for dossier in crate::data::case::DOSSIERS {
        facts.extend(dossier.asserts);
    }
    for pairing in crate::data::cross::PAIRINGS {
        facts.extend(pairing.asserts);
    }
    for page in crate::data::web::PAGES {
        facts.extend(page.asserts);
        if let Some(prompt) = page.prompt.as_ref() {
            for answer in prompt.answers {
                facts.extend(answer.asserts);
            }
        }
    }
    facts.sort_unstable();
    facts.dedup();
    facts
}

fn collect_asserts(effects: &'static [Effect], asserted: &mut Vec<Fact>) {
    for effect in effects {
        if let Effect::Assert(fact) = effect {
            asserted.push(fact);
        }
    }
}

fn effect_is_valid(script: &Script, effect: &Effect) -> bool {
    match effect {
        Effect::Reveal(id)
        | Effect::Invite(id, _)
        | Effect::SetPresence(id, _)
        | Effect::Deactivate(id)
        | Effect::Reactivate(id)
        | Effect::Automate(id) => contact_index(script, id).is_some(),
        Effect::Message(id, node) => contact_index(script, id)
            .map(|index| &script.contacts[index])
            .is_some_and(|definition| definition.nodes.iter().any(|other| other.id == *node)),
        Effect::RevealServer(id) => server_index(script, id).is_some(),
        Effect::Assert(_)
        | Effect::TakeHandle(_)
        | Effect::Ending
        | Effect::Finish(_)
        | Effect::Seize
        | Effect::Noticed => true,
    }
}
