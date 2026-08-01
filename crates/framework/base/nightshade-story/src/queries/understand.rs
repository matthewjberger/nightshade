use crate::data::deed::Deed;
use crate::data::story::Story;
use crate::queries::look::{held, here, reach};
use crate::resources::Play;

const SKIPPED: [&str; 6] = ["the", "a", "an", "at", "to", "into"];

pub fn understand(story: &Story, play: &Play, line: &str) -> Result<Deed, String> {
    let lowered = line.trim().to_lowercase();
    let words: Vec<&str> = lowered
        .split_whitespace()
        .filter(|word| !SKIPPED.contains(word))
        .collect();
    let Some((verb, rest)) = words.split_first() else {
        return Err("Say what you want to do.".to_string());
    };

    if rest.is_empty()
        && let Some(way) = way_named(story, play, verb)
    {
        return Ok(Deed::Go(way));
    }

    match *verb {
        "look" | "examine" | "study" | "read" | "inspect" => {
            if rest.is_empty() {
                return Ok(Deed::Look);
            }
            return match thing_named(story, rest, &reach(story, play)) {
                Some(thing) => Ok(Deed::Study(thing)),
                None => Err(missing(rest)),
            };
        }
        "go" | "walk" | "head" | "enter" => {
            let Some(word) = rest.first() else {
                return Err("Say which way you want to go.".to_string());
            };
            return match way_named(story, play, word) {
                Some(way) => Ok(Deed::Go(way)),
                None => Err(format!("There is no way {word} from here.")),
            };
        }
        "take" | "get" | "pick" | "grab" | "hold" => {
            return match thing_named(story, rest, &here(story, play)) {
                Some(thing) => Ok(Deed::Take(thing)),
                None => Err(missing(rest)),
            };
        }
        "drop" | "leave" | "release" => {
            return match thing_named(story, rest, &held(story, play)) {
                Some(thing) => Ok(Deed::Drop(thing)),
                None => Err("You are not holding that.".to_string()),
            };
        }
        "inventory" | "holding" | "pack" => return Ok(Deed::Held),
        _ => {}
    }

    let within = reach(story, play);
    let named = thing_named(story, rest, &within);
    for (index, rule) in story.rules.iter().enumerate() {
        if rule.verb != *verb {
            continue;
        }
        match (rule.thing, named) {
            (None, _) if rest.is_empty() => return Ok(Deed::Follow(index)),
            (Some(wanted), Some(found)) if wanted == found => return Ok(Deed::Follow(index)),
            (Some(wanted), None) if rest.is_empty() && within.contains(&wanted) => {
                return Ok(Deed::Follow(index));
            }
            _ => {}
        }
    }

    if !rest.is_empty() && named.is_none() {
        return Err(missing(rest));
    }
    Err(format!("You cannot {verb} that."))
}

fn way_named(story: &Story, play: &Play, word: &str) -> Option<usize> {
    story
        .rooms
        .get(play.at)?
        .ways
        .iter()
        .position(|way| way.word == word)
}

fn thing_named(story: &Story, words: &[&str], within: &[usize]) -> Option<usize> {
    within.iter().copied().find(|thing| {
        story.things.get(*thing).is_some_and(|found| {
            found
                .words
                .iter()
                .any(|word| words.contains(&word.as_str()))
        })
    })
}

fn missing(words: &[&str]) -> String {
    match words.last() {
        Some(word) => format!("You see no {word} here."),
        None => "You see nothing like that here.".to_string(),
    }
}
