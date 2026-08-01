use crate::data::deed::Deed;
use crate::data::story::{Change, Place, Step, Story};
use crate::queries::look::{look, open, pack, passes, place_of, word_for};
use crate::resources::Play;

pub fn begin(play: &mut Play, story: &Story) {
    play.at = story.start;
    play.places = story.things.iter().map(|thing| thing.start).collect();
    play.values = story.values.clone();
    play.told.clear();
    play.ended = false;
    if !story.title.is_empty() {
        play.told.push(story.title.clone());
    }
    for line in &story.opening {
        play.told.push(line.clone());
    }
    arrive(play, story);
}

pub fn say(play: &mut Play, text: impl Into<String>) {
    let text = text.into();
    if !text.is_empty() {
        play.told.push(text);
    }
}

pub fn move_thing(play: &mut Play, thing: usize, place: Place) {
    while play.places.len() <= thing {
        play.places.push(Place::Gone);
    }
    play.places[thing] = place;
}

pub fn set_value(play: &mut Play, name: &str, step: Step, amount: i32) {
    let held = play
        .values
        .iter_mut()
        .find(|(known, _)| known == name)
        .map(|(_, held)| held);
    match held {
        Some(held) => {
            *held = match step {
                Step::Set => amount,
                Step::Add => *held + amount,
                Step::Subtract => *held - amount,
            }
        }
        None => {
            let start = match step {
                Step::Set | Step::Add => amount,
                Step::Subtract => -amount,
            };
            play.values.push((name.to_string(), start));
        }
    }
}

pub fn obey(play: &mut Play, story: &Story, deed: Deed) {
    if play.ended {
        return;
    }
    match deed {
        Deed::Look => arrive(play, story),
        Deed::Held => {
            let line = pack(story, play);
            say(play, line);
        }
        Deed::Go(way) => walk(play, story, way),
        Deed::Take(thing) => take(play, story, thing),
        Deed::Drop(thing) => put_down(play, story, thing),
        Deed::Study(thing) => study(play, story, thing),
        Deed::Follow(rule) => follow(play, story, rule),
    }
}

fn arrive(play: &mut Play, story: &Story) {
    for line in look(story, play) {
        play.told.push(line);
    }
}

fn walk(play: &mut Play, story: &Story, way: usize) {
    let Some(room) = story.rooms.get(play.at) else {
        return;
    };
    let Some(step) = room.ways.get(way) else {
        return;
    };
    let to = step.to;
    let barred = step.barred.clone();
    if !open(story, play, way) {
        let line = if barred.is_empty() {
            "You cannot go that way.".to_string()
        } else {
            barred
        };
        say(play, line);
        return;
    }
    play.at = to;
    arrive(play, story);
}

fn take(play: &mut Play, story: &Story, thing: usize) {
    let Some(found) = story.things.get(thing) else {
        return;
    };
    if place_of(play, thing) != Place::Room(play.at) {
        say(play, "That is not here.");
        return;
    }
    let word = word_for(story, thing);
    if !found.portable {
        say(play, format!("The {word} will not move."));
        return;
    }
    let line = if found.taking.is_empty() {
        format!("You take the {word}.")
    } else {
        found.taking.clone()
    };
    move_thing(play, thing, Place::Held);
    say(play, line);
}

fn put_down(play: &mut Play, story: &Story, thing: usize) {
    if place_of(play, thing) != Place::Held {
        say(play, "You are not holding that.");
        return;
    }
    let word = word_for(story, thing);
    let room = play.at;
    move_thing(play, thing, Place::Room(room));
    say(play, format!("You put down the {word}."));
}

fn study(play: &mut Play, story: &Story, thing: usize) {
    let Some(found) = story.things.get(thing) else {
        return;
    };
    let line = if found.look.is_empty() {
        format!("The {} tells you nothing more.", word_for(story, thing))
    } else {
        found.look.clone()
    };
    say(play, line);
}

fn follow(play: &mut Play, story: &Story, rule: usize) {
    let Some(found) = story.rules.get(rule) else {
        return;
    };
    let allowed = match &found.when {
        Some(test) => passes(play, test),
        None => true,
    };
    if !allowed {
        let line = if found.barred.is_empty() {
            "Nothing happens.".to_string()
        } else {
            found.barred.clone()
        };
        say(play, line);
        return;
    }
    let line = found.say.clone();
    let changes = found.changes.clone();
    say(play, line);
    for change in changes {
        apply(play, story, change);
    }
}

fn apply(play: &mut Play, story: &Story, change: Change) {
    match change {
        Change::Move { thing, place } => move_thing(play, thing, place),
        Change::Value { name, step, amount } => set_value(play, &name, step, amount),
        Change::Go(room) => {
            play.at = room;
            arrive(play, story);
        }
        Change::End(text) => {
            say(play, text);
            play.ended = true;
        }
    }
}
