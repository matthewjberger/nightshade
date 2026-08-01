use crate::data::deed::{Deed, Link};
use crate::data::story::{Place, Sign, Story, Test};
use crate::resources::Play;

pub fn value(play: &Play, name: &str) -> i32 {
    play.values
        .iter()
        .find(|(held, _)| held == name)
        .map(|(_, amount)| *amount)
        .unwrap_or_default()
}

pub fn place_of(play: &Play, thing: usize) -> Place {
    play.places.get(thing).copied().unwrap_or_default()
}

pub fn holds(play: &Play, thing: usize) -> bool {
    place_of(play, thing) == Place::Held
}

pub fn here(story: &Story, play: &Play) -> Vec<usize> {
    (0..story.things.len())
        .filter(|thing| place_of(play, *thing) == Place::Room(play.at))
        .collect()
}

pub fn held(story: &Story, play: &Play) -> Vec<usize> {
    (0..story.things.len())
        .filter(|thing| holds(play, *thing))
        .collect()
}

pub fn reach(story: &Story, play: &Play) -> Vec<usize> {
    let mut found = here(story, play);
    found.extend(held(story, play));
    found
}

pub fn passes(play: &Play, test: &Test) -> bool {
    match test {
        Test::Holds(thing) => holds(play, *thing),
        Test::At(room) => play.at == *room,
        Test::Here(thing) => {
            let place = place_of(play, *thing);
            place == Place::Held || place == Place::Room(play.at)
        }
        Test::Value { name, sign, amount } => {
            let held = value(play, name);
            match sign {
                Sign::Above => held > *amount,
                Sign::Below => held < *amount,
                Sign::Same => held == *amount,
                Sign::Apart => held != *amount,
                Sign::Least => held >= *amount,
                Sign::Most => held <= *amount,
            }
        }
        Test::Not(inner) => !passes(play, inner),
        Test::All(every) => every.iter().all(|inner| passes(play, inner)),
        Test::Any(some) => some.iter().any(|inner| passes(play, inner)),
    }
}

pub fn open(story: &Story, play: &Play, way: usize) -> bool {
    let Some(room) = story.rooms.get(play.at) else {
        return false;
    };
    let Some(way) = room.ways.get(way) else {
        return false;
    };
    match &way.when {
        Some(test) => passes(play, test),
        None => true,
    }
}

pub fn word_for(story: &Story, thing: usize) -> String {
    let Some(found) = story.things.get(thing) else {
        return String::new();
    };
    match found.words.first() {
        Some(word) => word.clone(),
        None => found.name.clone(),
    }
}

pub fn look(story: &Story, play: &Play) -> Vec<String> {
    let Some(room) = story.rooms.get(play.at) else {
        return Vec::new();
    };
    let mut told = vec![room.title.clone(), room.look.clone()];
    let standing = here(story, play);
    if !standing.is_empty() {
        let names: Vec<String> = standing
            .iter()
            .map(|thing| format!("the {}", word_for(story, *thing)))
            .collect();
        told.push(format!("You see {}.", listed(&names)));
    }
    let ways: Vec<String> = room.ways.iter().map(|way| way.word.clone()).collect();
    if !ways.is_empty() {
        told.push(format!("Ways from here: {}.", listed(&ways)));
    }
    told
}

pub fn pack(story: &Story, play: &Play) -> String {
    let holding = held(story, play);
    if holding.is_empty() {
        return "You are holding nothing.".to_string();
    }
    let names: Vec<String> = holding
        .iter()
        .map(|thing| format!("the {}", word_for(story, *thing)))
        .collect();
    format!("You are holding {}.", listed(&names))
}

pub fn links(story: &Story, play: &Play) -> Vec<Link> {
    let mut offered = Vec::new();
    if let Some(room) = story.rooms.get(play.at) {
        for (index, way) in room.ways.iter().enumerate() {
            offered.push(Link {
                words: format!("Go {}", way.word),
                deed: Deed::Go(index),
            });
        }
    }
    for thing in here(story, play) {
        let word = word_for(story, thing);
        offered.push(Link {
            words: format!("Look at the {word}"),
            deed: Deed::Study(thing),
        });
        if story.things[thing].portable {
            offered.push(Link {
                words: format!("Take the {word}"),
                deed: Deed::Take(thing),
            });
        }
    }
    for thing in held(story, play) {
        offered.push(Link {
            words: format!("Drop the {}", word_for(story, thing)),
            deed: Deed::Drop(thing),
        });
    }
    let within = reach(story, play);
    for (index, rule) in story.rules.iter().enumerate() {
        let words = match rule.thing {
            Some(thing) if within.contains(&thing) => {
                format!("{} the {}", upper_first(&rule.verb), word_for(story, thing))
            }
            Some(_) => continue,
            None => upper_first(&rule.verb),
        };
        offered.push(Link {
            words,
            deed: Deed::Follow(index),
        });
    }
    offered
}

pub fn listed(names: &[String]) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        _ => {
            let (last, first) = names.split_last().unwrap_or((&names[0], &[]));
            format!("{} and {last}", first.join(", "))
        }
    }
}

pub fn upper_first(text: &str) -> String {
    let mut letters = text.chars();
    match letters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + letters.as_str(),
        None => String::new(),
    }
}
