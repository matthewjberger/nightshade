# nightshade-ui-dialogue

The card a conversation is spoken on, a speaker name over the line being typed out. It draws the `Conversation` that `nightshade-dialogue` advances, so a game opens a conversation and never touches the card again. The card hides itself whenever nobody is talking.

## Composing it

```rust
app.add_plugin(nightshade_dialogue::DialoguePlugin {
    settings: nightshade_dialogue::DialogueSettings {
        letters_a_second: 34.0,
    },
});
app.add_plugin(nightshade_ui::UiPlugin::new());
app.add_plugin(nightshade_ui_dialogue::UiDialoguePlugin::default());
```

`UiDialoguePlugin` carries a `DialogueStyle` field and inserts that style beside an empty `DialogueCard`. Its one system runs on `Stage::LateUpdate`, after the conversation has advanced for the frame.

## Using it

```rust
fn build_the_card(
    style: Res<DialogueStyle>,
    mut card: ResMut<DialogueCard>,
    mut build: UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    nightshade_ui_dialogue::commands::build(&mut tree, &style, &mut card);
    tree.finish();
}

fn keep_talking(mut conversation: ResMut<Conversation>) {
    open(&mut conversation, "Warden", LINES);
}
```

`build` places the panel above the bottom of the screen and fills the `DialogueCard` with the panel, the speaker node and the line node. The frame system then writes the speaker only when the name changes and the line only when another letter has arrived, and sets the panel visible for as long as the conversation is talking. A game changes what is said by calling into `nightshade-dialogue`.

## Driving it

```
just example ui_dialogue
```

Press R to open a conversation of three lines and F to close it. Space fills the rest of the line in while it is still typing and moves to the next line once it has arrived, and the card hides itself when the last one is past. The readout gives the speaker, the letters shown against the whole line, and the lines still waiting.
