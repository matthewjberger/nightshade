# nightshade-save

A save file is a slot, a header and a version around whatever a game must still have the next time it starts. A game declares one type as its save document by implementing `Save` on it, and the crate writes a header line in front of the payload, reads a slot back, lists the slots with their headers for a load screen, and takes one away. It also holds the clock that decides when to write. The payload stays the game's own, written as JSON or as bincode, and the bytes go through `nightshade-store`.

## Composing it

The crate carries no plugin. A game calls the functions directly and composes nothing.

## Using it

```rust
use nightshade_save::prelude::{save_read, save_slots, save_write};

impl nightshade_save::Save for Save {
    const FOLDER: &'static str = SAVE_FOLDER;
    const NAME: &'static str = SAVE_NAME;
    const VERSION: u32 = SAVE_VERSION;
}

save_write(SAVE_SLOT, &label, &sheet);
let restored: Option<Save> = save_read(SAVE_SLOT);
let listed = save_slots::<Save>();
```

`FOLDER` names the folder under the platform's own data location, `NAME` names the document and prefixes the file, and `VERSION` is what a later build compares against. `SLOTS` defaults to one and `CODEC` defaults to `SaveCodec::Json`, with `SaveCodec::Bincode` the other choice. `save_write` returns false when the slot is past `SLOTS` or the value fails to encode. `save_read` returns nothing when the name does not match or the file's version is ahead of the program's, so a file the program cannot read reports nothing rather than a value that is wrong. `save_header` and `save_slots` hand back the `SaveHeader` with its name, version and label, which is what a load screen draws.

## Reading a file an older build wrote

A file whose header names a version below `VERSION` goes to `Save::migrate` with that version and the raw payload, and what it returns is the answer. The default body decodes the payload as the current shape, which covers a document that only gained fields carrying `#[serde(default)]`. A game whose shape moved further overrides it:

```rust
fn migrate(from: u32, payload: &[u8]) -> Option<Self> {
    if from < 2 {
        return None;
    }
    Self::CODEC.decode(payload)
}
```

`SaveCodec::encode` and `SaveCodec::decode` are the whole format, so a migration body reaches the same pair that `save_write` and `save_read` use.

## The autosave clock

`Autosave` holds the period between writes, the wait that is left, and whether a person asked for one. A game keeps it in its own save resource.

```rust
let leaving = window.should_exit || window.close_requested;
let Some(due) = save.autosave.due(time.delta_time.min(0.05), leaving) else {
    return;
};
```

`ask` marks the next check, `due` advances the wait and reports `SaveDue::Asked`, `SaveDue::Leaving` or `SaveDue::Period`, and the value resets itself when it fires. An ask outranks a leave and a leave outranks the period, so a game that says something after a write can say it only for `SaveDue::Asked`.

## Taking it from another crate

The crate does not split at a `contract` feature. Nothing asks it for types, so a game takes it whole and names `nightshade-save` in its own dependencies.
