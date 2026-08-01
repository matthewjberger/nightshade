# nightshade-ui-toast

The card near the top of the screen that says one thing and then goes away. It draws the `Toast` that `nightshade-toast` counts down, so a game says a line and the card appears, holds and hides on its own. `apps/features/composed` carries one.

## Composing it

```rust
app.add_plugin(nightshade_toast::ToastPlugin {
    settings: nightshade_toast::ToastSettings {
        showing: 2.4,
        fading: 0.6,
    },
});
app.add_plugin(nightshade_ui::UiPlugin::new());
app.add_plugin(nightshade_ui_toast::UiToastPlugin::default());
```

`UiToastPlugin` carries a `ToastStyle` field and inserts that style beside an empty `ToastCard`. Its one system runs on `Stage::LateUpdate`, after the countdown has moved for the frame.

## Using it

```rust
fn build_the_card(style: Res<ToastStyle>, mut card: ResMut<ToastCard>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    nightshade_ui_toast::commands::build(&mut tree, &style, &mut card);
    tree.finish();
}

fn say_something(mut toast: ResMut<Toast>, settings: Res<ToastSettings>) {
    say(&mut toast, &settings, "the gate is open");
}
```

`build` places the card below the top of the screen and fills the `ToastCard` with the panel and the line node. A game then calls into `nightshade-toast` to say a line, which resets the countdown. The frame system sets the panel visible while anything remains on the clock and writes the text only when the line changes.

## Driving it

```
just example ui_toast
```

Press 1 through 4 to raise the card with one of four lines and Space to hush it at once. The card drops on its own when the countdown runs out, and the post behind the body rises and sinks with the fade.
