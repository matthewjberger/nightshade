---
name: project_save_crate
description: nightshade-save is the slot, the header, the version and the autosave clock around a game's own save payload, over nightshade-store
type: project
---

Six games wrote a save of their own and each wrote the same parts around the
payload. `apps/games/farm`, `apps/games/srpg` and `apps/games/thornmere` kept
theirs as JSON. `apps/games/voxels` and `apps/games/hollowmere` kept theirs as
bytes behind a four-byte magic and a version of their own.
`apps/games/bulwark` kept one number as text. What repeated was the slot, the
header, the check of the version, the list of the slots a load screen wants,
the removal of one slot, and the clock that decides when to write. What did not
repeat, and must not, is the payload.

`nightshade-save` takes the parts that repeat. `nightshade-store` is its only
engine dependency, because the store names no format and a header belongs above
it. Every game that saves is on it, and `nightshade-store` is now named only by
`apps/viewer`, which keeps settings rather than a save.

## What a game declares

```rust
impl nightshade_save::Save for Save {
    const FOLDER: &'static str = crate::theme::SAVE_FOLDER;
    const NAME: &'static str = crate::theme::SAVE_NAME;
    const VERSION: u32 = crate::theme::SAVE_VERSION;
}
```

`SLOTS` is one unless the game says more, and `CODEC` is JSON unless the game
says `SaveCodec::Bincode`. The payload is whatever the game's own type
serializes to, so the crate imposes no shape on it.

Then four calls, and no plugin to compose:

```rust
save_write(slot, "Day 4", &document);
let document: Option<Save> = save_read::<Save>(slot);
let found: Vec<(u32, SaveHeader)> = save_slots::<Save>();
save_remove::<Save>(slot);
```

## What the file looks like

One header line, then the payload:

```
nightshade-save<TAB>farm<TAB>1<TAB>Day 4
{
  "day": 4,
```

The line is text so a JSON save stays readable and a bincode save still has a
header that costs nothing to read. The label is the game's own, with any
control character replaced by a space so it cannot break the line. The slot is
in the file name, `<name>-<slot>.save`, because the store lists no folder: that
is also why `save_slots` probes zero up to `SLOTS` rather than reading a
directory.

## The version check is loud

`save_read` reports nothing when the name in the header is not the game's, when
the version is newer than the program knows, or when the payload does not
decode. A file the program cannot read is never a value that is wrong.
`save_header` reports the header even then, so a load screen can still show
that a slot holds something.

## An older file goes through migrate

A header whose version is below the program's does not reach the ordinary
decode. `save_read` hands the version and the raw payload to `Save::migrate`,
and whatever that returns is the answer. The default body decodes the payload
as the current shape, which is what `#[serde(default)]` on the document covers,
so a game that adds a field writes nothing. A game whose shape moved enough
that serde cannot follow overrides `migrate`, reads the old payload itself, and
builds the current document from it. `SaveCodec::encode` and
`SaveCodec::decode` hold the whole format and sit in `data`, so the trait's
default body, `save_write` and `save_read` all reach the same pair.

## The clock is the crate's too

`Autosave` holds the period, the wait that is left and whether a person asked
for a write. A game keeps one in its own save resource and calls `ask` where a
menu button or a command asks for a write.

```rust
let leaving = window.should_exit || window.close_requested;
let Some(due) = save.autosave.due(time.delta_time.min(0.05), leaving) else {
    return;
};
```

`due` advances the wait and reports `SaveDue::Asked`, `SaveDue::Leaving` or
`SaveDue::Period`, and resets itself when it fires. An ask outranks a leave and
a leave outranks the period, so a game says "the world is saved" only on
`SaveDue::Asked` and stays quiet on the other two.

## What a game loses when it moves onto the crate

The file name and the framing both change, so the first run after the move
starts a new game. `apps/games/voxels` and `apps/games/hollowmere` each dropped
a hand-rolled byte format of about three hundred lines for one bincode
document, and their content enums keep the codes they already had through
`#[serde(into = "u8", try_from = "u8")]` over the existing `code` and
`from_code`, so the numbers on disk still mean what they meant.
