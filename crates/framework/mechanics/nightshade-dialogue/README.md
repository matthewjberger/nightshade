# nightshade-dialogue

A queue of lines revealed one letter at a time, and the replies waiting at the end. The crate holds who is speaking, the line on screen, how much of it has been revealed, the lines still queued behind it, and the replies offered to the player. A game opens a conversation with a speaker and a list of lines, steps through them, and reads the reply that was chosen.

## Composing it

```rust
use nightshade_dialogue::{DialoguePlugin, DialogueSettings};

app.add_plugin(DialoguePlugin {
    settings: DialogueSettings {
        letters_a_second: 45.0,
    },
});
```

The plugin inserts `DialogueSettings` and `Conversation` and registers `reveal_the_line` on `Stage::Update`, which walks the revealed count up to the length of the line. `Conversation` is what the mechanic announces, and a panel reads it to draw.

## Using it

```rust
use nightshade::prelude::*;
use nightshade_dialogue::Conversation;
use nightshade_dialogue::commands::{choose, close, offer, open, reveal_all, step};
use nightshade_dialogue::queries::{fully_shown, last_line, shown, talking};

fn talk(mut conversation: ResMut<Conversation>) {
    open(&mut conversation, "The Warden", &["You came back.", "That was unwise."]);
    offer(&mut conversation, &["I had to.", "I am leaving."]);

    if !fully_shown(&conversation) {
        reveal_all(&mut conversation);
    } else if last_line(&conversation) {
        choose(&mut conversation, 0);
        close(&mut conversation);
    } else {
        step(&mut conversation);
    }

    let on_screen = shown(&conversation);
    let busy = talking(&conversation);
}
```

`open` takes a speaker and the lines and starts the first one, and `step` moves to the next line or closes the conversation when the queue is empty. `reveal_all` finishes the current line at once, which is what a key press does while text is still crawling. `shown` gives the letters revealed so far and `fully_shown` says whether the line is complete, `last_line` says whether the queue behind it is empty, and `talking` says whether a conversation is running at all. `choose` records a reply index once `offer` has put the replies up.

## Driving it

```
just example dialogue
```

Space finishes the crawling line, steps to the next one, and starts the conversation over once an answer is given, and 1 and 2 take the two replies the crate is offering. The warden leans in while `fully_shown` is false and steps forward or turns away on the answer `choose` recorded, and the card shows the letters `shown` has released.
