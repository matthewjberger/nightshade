# nightshade-readout

The line a program keeps in its window title while it runs: a fixed title, whatever notes a system pushed this frame, and the frame rate. It draws nothing on the screen, so a rendering app can say what it is showing without putting anything over the picture. Every app in `apps/rendering/` and `apps/vulkan/` uses it, and so does every example in `crates/framework/`.

## Composing it

```rust
app.add_plugin(nightshade_readout::ReadoutPlugin::titled("Bounced Light"));
```

`ReadoutPlugin` inserts the `Readout` resource carrying the title it was given and an empty note list. Its one system runs on `Stage::Update`, which is where a window call belongs.

## Using it

```rust
fn write_the_line(mut readout: ResMut<Readout>) {
    clear_notes(&mut readout);
    say(&mut readout, format!("{count} bodies"));
    say(&mut readout, "the sun is turning");
}
```

A system clears the notes and pushes what it wants shown, usually every frame. `say` takes anything that becomes a `String`, and `clear_notes` empties the list so last frame's notes do not stack up. The frame system joins the title, the notes and the current frame rate with `line` and writes the result to the window title.

## Driving it

```
just example readout
```

Press 1 through 5 to add a note and Z to clear them all. Each note joins the window title beside the frame rate, and a post in the row stands up for every note the readout is carrying.
