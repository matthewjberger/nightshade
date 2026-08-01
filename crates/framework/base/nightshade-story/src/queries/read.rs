use crate::data::story::{Change, Place, Room, Rule, Sign, Step, Story, Test, Thing, Way};

enum Block {
    None,
    Room(usize),
    Thing(usize),
    Rule(usize),
}

pub fn read(source: &str) -> Result<Story, String> {
    let lines: Vec<(usize, &str)> = source
        .lines()
        .enumerate()
        .map(|(number, line)| (number + 1, line.trim()))
        .filter(|(_, line)| !line.is_empty() && !line.starts_with("--"))
        .collect();

    let mut story = Story::default();
    for (_, line) in &lines {
        let (word, rest) = split_first(line);
        match word {
            "room" => story.rooms.push(Room {
                name: rest.to_string(),
                ..Room::default()
            }),
            "thing" => story.things.push(Thing {
                name: rest.to_string(),
                ..Thing::default()
            }),
            _ => {}
        }
    }

    let mut block = Block::None;
    for (number, line) in &lines {
        fill(&mut story, &mut block, line).map_err(|fault| format!("line {number}: {fault}"))?;
    }

    if story.rooms.is_empty() {
        return Err("the story names no room".to_string());
    }
    Ok(story)
}

fn fill(story: &mut Story, block: &mut Block, line: &str) -> Result<(), String> {
    let (word, rest) = split_first(line);
    match word {
        "room" => *block = Block::Room(room_index(story, rest)?),
        "thing" => *block = Block::Thing(thing_index(story, rest)?),
        "verb" => {
            let mut parts = rest.split_whitespace();
            let verb = parts
                .next()
                .ok_or_else(|| "verb wants a word".to_string())?
                .to_lowercase();
            let thing = match parts.next() {
                Some(name) => Some(thing_index(story, name)?),
                None => None,
            };
            story.rules.push(Rule {
                verb,
                thing,
                ..Rule::default()
            });
            *block = Block::Rule(story.rules.len() - 1);
        }
        "story" => story.title = rest.to_string(),
        "opening" => story.opening.push(rest.to_string()),
        "start" => story.start = room_index(story, rest)?,
        "value" => {
            let (name, amount) = split_first(rest);
            story.values.push((name.to_string(), number(amount)?));
        }
        _ => match block {
            Block::None => return Err(format!("{word} stands outside a room, a thing or a verb")),
            Block::Room(index) => {
                let index = *index;
                room_line(story, index, word, rest)?;
            }
            Block::Thing(index) => {
                let index = *index;
                thing_line(story, index, word, rest)?;
            }
            Block::Rule(index) => {
                let index = *index;
                rule_line(story, index, word, rest)?;
            }
        },
    }
    Ok(())
}

fn room_line(story: &mut Story, index: usize, word: &str, rest: &str) -> Result<(), String> {
    match word {
        "title" => story.rooms[index].title = rest.to_string(),
        "look" => join(&mut story.rooms[index].look, rest),
        "way" => {
            let way = way_line(story, rest)?;
            story.rooms[index].ways.push(way);
        }
        _ => return Err(format!("a room does not know {word}")),
    }
    Ok(())
}

fn way_line(story: &Story, rest: &str) -> Result<Way, String> {
    let (rest, barred) = cut(rest, " barred ");
    let (rest, when) = cut(rest, " when ");
    let mut parts = rest.split_whitespace();
    let word = parts
        .next()
        .ok_or_else(|| "way wants a direction".to_string())?
        .to_lowercase();
    let to = room_index(
        story,
        parts.next().ok_or_else(|| "way wants a room".to_string())?,
    )?;
    let when = match when {
        Some(text) => Some(test(story, text)?),
        None => None,
    };
    Ok(Way {
        word,
        to,
        when,
        barred: barred.unwrap_or_default().to_string(),
    })
}

fn thing_line(story: &mut Story, index: usize, word: &str, rest: &str) -> Result<(), String> {
    match word {
        "words" => {
            story.things[index].words = rest
                .split_whitespace()
                .map(|word| word.to_lowercase())
                .collect();
        }
        "look" => join(&mut story.things[index].look, rest),
        "taking" => story.things[index].taking = rest.to_string(),
        "portable" => story.things[index].portable = true,
        "at" => {
            let place = place(story, rest)?;
            story.things[index].start = place;
        }
        _ => return Err(format!("a thing does not know {word}")),
    }
    Ok(())
}

fn rule_line(story: &mut Story, index: usize, word: &str, rest: &str) -> Result<(), String> {
    match word {
        "say" => join(&mut story.rules[index].say, rest),
        "barred" => join(&mut story.rules[index].barred, rest),
        "when" => {
            let when = test(story, rest)?;
            story.rules[index].when = Some(when);
        }
        "set" => {
            let mut parts = rest.split_whitespace();
            let name = parts
                .next()
                .ok_or_else(|| "set wants a value".to_string())?
                .to_string();
            let step = match parts.next() {
                Some("set") => Step::Set,
                Some("add") => Step::Add,
                Some("subtract") => Step::Subtract,
                _ => return Err("set wants set, add or subtract".to_string()),
            };
            let amount = number(parts.next().unwrap_or_default())?;
            story.rules[index]
                .changes
                .push(Change::Value { name, step, amount });
        }
        "move" => {
            let (name, where_to) = split_first(rest);
            let thing = thing_index(story, name)?;
            let place = place(story, where_to)?;
            story.rules[index]
                .changes
                .push(Change::Move { thing, place });
        }
        "go" => {
            let room = room_index(story, rest)?;
            story.rules[index].changes.push(Change::Go(room));
        }
        "end" => story.rules[index]
            .changes
            .push(Change::End(rest.to_string())),
        _ => return Err(format!("a verb does not know {word}")),
    }
    Ok(())
}

fn test(story: &Story, text: &str) -> Result<Test, String> {
    let text = text.trim();
    let parts: Vec<&str> = text.split(" or ").collect();
    if parts.len() > 1 {
        let mut any = Vec::new();
        for part in parts {
            any.push(test(story, part)?);
        }
        return Ok(Test::Any(any));
    }
    let parts: Vec<&str> = text.split(" and ").collect();
    if parts.len() > 1 {
        let mut all = Vec::new();
        for part in parts {
            all.push(test(story, part)?);
        }
        return Ok(Test::All(all));
    }
    term(story, text)
}

fn term(story: &Story, text: &str) -> Result<Test, String> {
    let (word, rest) = split_first(text);
    match word {
        "not" => return Ok(Test::Not(Box::new(term(story, rest)?))),
        "holds" => return Ok(Test::Holds(thing_index(story, rest)?)),
        "here" => return Ok(Test::Here(thing_index(story, rest)?)),
        "at" => return Ok(Test::At(room_index(story, rest)?)),
        _ => {}
    }
    let mut parts = rest.split_whitespace();
    let sign = match parts.next() {
        Some("above") => Sign::Above,
        Some("below") => Sign::Below,
        Some("same") => Sign::Same,
        Some("apart") => Sign::Apart,
        Some("least") => Sign::Least,
        Some("most") => Sign::Most,
        _ => {
            return Err(format!(
                "{word} wants above, below, same, apart, least or most"
            ));
        }
    };
    let amount = number(parts.next().unwrap_or_default())?;
    Ok(Test::Value {
        name: word.to_string(),
        sign,
        amount,
    })
}

fn place(story: &Story, text: &str) -> Result<Place, String> {
    match text {
        "held" => Ok(Place::Held),
        "gone" => Ok(Place::Gone),
        name => Ok(Place::Room(room_index(story, name)?)),
    }
}

fn room_index(story: &Story, name: &str) -> Result<usize, String> {
    story
        .rooms
        .iter()
        .position(|room| room.name == name)
        .ok_or_else(|| format!("no room is named {name}"))
}

fn thing_index(story: &Story, name: &str) -> Result<usize, String> {
    story
        .things
        .iter()
        .position(|thing| thing.name == name)
        .ok_or_else(|| format!("no thing is named {name}"))
}

fn number(text: &str) -> Result<i32, String> {
    text.parse()
        .map_err(|_| format!("{text} is not a whole number"))
}

fn split_first(line: &str) -> (&str, &str) {
    match line.split_once(char::is_whitespace) {
        Some((word, rest)) => (word, rest.trim()),
        None => (line, ""),
    }
}

fn cut<'a>(text: &'a str, marker: &str) -> (&'a str, Option<&'a str>) {
    match text.find(marker) {
        Some(at) => (text[..at].trim(), Some(text[at + marker.len()..].trim())),
        None => (text.trim(), None),
    }
}

fn join(held: &mut String, rest: &str) {
    if !held.is_empty() {
        held.push(' ');
    }
    held.push_str(rest);
}
