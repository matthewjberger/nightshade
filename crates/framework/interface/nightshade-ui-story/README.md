# nightshade-ui-story

The reading pane a story is told on. It draws the lines that `nightshade-story` has told, puts the links it offers on buttons below them, and keeps a line at the bottom for a player who would rather type. Clicking a link and typing a command reach the world the same way, so a game gets both players from one card.

## Composing it

```rust
app.add_plugin(nightshade_ui::UiPlugin::new());
app.add_plugin(nightshade_ui_story::UiStoryPlugin {
    style: nightshade_ui_story::StoryStyle {
        lines: 18,
        columns: 78,
        ..Default::default()
    },
});
```

`UiStoryPlugin` carries a `StoryStyle` field and inserts that style beside an empty `StoryCard`. It registers `take_the_line` on `Stage::Update`, which reads what the player asked for and answers it, and `show_the_story` on `Stage::LateUpdate`, once the world has settled for the frame. The plugin says it wants `Story` and `Play`, and the game is what inserts them.

## Using it

```rust
fn build_the_card(style: Res<StoryStyle>, mut card: ResMut<StoryCard>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    nightshade_ui_story::commands::build(&mut tree, &style, &mut card);
    tree.finish();
}
```

`build` puts a panel in the middle of the screen and fills the `StoryCard` with one text node for each line the style asks for, one button for each link, and the input at the foot. The frame system wraps every paragraph told to the style's column width, writes the tail of that into the line nodes, and writes a line only when it has changed. It writes the link buttons from what `links` offers and hides the ones left over, and it takes the input away once the story has ended.

`StoryStyle` carries the panel width, the row height a button and the input stand at, the padding, the corner radius, how many lines and links the card holds, the column width the text wraps at, and the colors. The panel height follows from those counts, so a card cannot be asked for more lines than it has room to draw.

## Driving it

```
just example ui_story
```

Click a link to take it, or type a command on the line at the foot and press enter. Take the key, open the gate, and go through to the garden. Press R to start the story over. The readout gives the lines standing on the card, the last of them, and how many links are on offer.
