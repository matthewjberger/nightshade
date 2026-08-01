# nightshade-story

A world told in words, and the text file it is written in. The crate holds what a room is, what a thing is and what a verb does, reads that shape out of a plain source file, and answers what the player can see and reach right now. It names no engine and depends on nothing, so a checker beside it reads the same story the game plays.

A passage is not the model here. The model is the room, the thing and the verb, and the same world answers a link that was clicked and a line that was typed.

## Composing it

```rust
use nightshade_story::prelude::{Play, begin, read};

let story = match read(SOURCE) {
    Ok(story) => story,
    Err(fault) => panic!("the story does not read: {fault}"),
};
let mut play = Play::default();
begin(&mut play, &story);
app.insert_resource(story);
app.insert_resource(play);
```

The crate carries no plugin, no system and no settings, so a game inserts `Story` and `Play` and calls the functions from its own systems. `Story` is what the file said and never changes. `Play` is where the player stands, where each thing lies, the values the story keeps, and every line told so far.

## The format

A story is a plain text file of one keyword to a line. Blank lines are skipped, and a line that starts with two dashes is a note to the writer.

```
story Candlewake
opening The sea has gone quiet, which is worse than the noise.
start shore

value lit 0

room shore
  title The Shingle
  look Grey stones grind under you.
  way north hall

room hall
  title The Long Hall
  look Salt has eaten the plaster off the walls.
  way south shore
  way up stair when lit same 1 barred The stair is black above you.

thing lantern
  words lantern brass
  look A brass lantern with a cold wick.
  at hall
  portable
  taking You lift the lantern.

verb light lantern
  when holds lantern and holds tinderbox
  say You strike the tinderbox, and the wick takes.
  set lit set 1
  barred You want something to strike it with.
```

`story`, `opening`, `start` and `value` stand at the top. A `room` block takes `title`, `look` and `way`, and a `way` reads a direction word, the room it reaches, an optional `when` and an optional `barred` line to say when the test fails. A `thing` block takes `words`, which is every noun the player may type for it, `look`, `at` with a room name or `held` or `gone`, `portable`, and `taking`. A `verb` block names a word and an optional thing, and takes `when`, `say`, `barred` and the changes `set`, `move`, `go` and `end`.

A test is one term, or terms joined by `and` and `or`. A term is `holds <thing>`, `here <thing>`, `at <room>`, `not <term>`, or a value against a number with `above`, `below`, `same`, `apart`, `least` or `most`. Rooms and things may be named before they are declared, because the reader collects every name first.

## Using it

```rust
use nightshade_story::prelude::{Deed, links, look, obey, pack, understand, value, wrap};

let told = look(&story, &play);
let offered = links(&story, &play);

match understand(&story, &play, "take the lantern") {
    Ok(deed) => obey(&mut play, &story, deed),
    Err(refusal) => say(&mut play, refusal),
}

obey(&mut play, &story, offered[0].deed);
let lines = wrap(&told[1], 72);
```

`understand` turns a typed line into a `Deed` or gives back the line to print instead, and `links` gives the same `Deed` values ready made with the words to put on a button. `obey` is the one way a deed reaches the world, so a choice player and a parser player run down the same path. `look` gives the paragraphs for the room the player stands in, `pack` gives what the player holds, `value` reads a value the story keeps, and `wrap` breaks a paragraph at a column width for whatever draws it.

`begin`, `say`, `move_thing` and `set_value` change the play directly, which is what a game reaches for when something outside the story moves a thing or sets a value.

## Driving it

```
just example story
```

Press 1 through 6 to take the link with that number, L to look again, I to see what you hold, and R to start over. The near row of blocks is the rooms, and the one you stand in rises and turns gold. The far row is the things, blue where one lies in the room with you and red where you hold it. The readout gives the room, the last line told, what you hold, and the links on offer.
