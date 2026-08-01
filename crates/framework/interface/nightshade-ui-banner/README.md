# nightshade-ui-banner

One line of text across the top of the screen, with a card behind it. A game writes the line, the color and whether the banner is shown, and the crate writes the retained interface only when one of those changes. `apps/games/coin_rush` and `apps/games/coinfall` keep their running line this way.

## Composing it

```rust
app.add_plugin(nightshade_ui::UiPlugin::new());
app.add_plugin(nightshade_ui_banner::BannerPlugin::default());
```

`BannerPlugin` carries a `BannerStyle` field, and it inserts that style, the `Banner` a game writes and the handles the builder fills. Its one system runs on `Stage::Update`.

## Using it

```rust
fn build_the_banner(
    style: Res<BannerStyle>,
    mut handles: ResMut<BannerHandles>,
    mut build: UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    *handles = add_banner(&mut tree, &style);
    tree.finish();
}

fn keep_it_current(mut banner: ResMut<Banner>) {
    say(&mut banner, "the gate is open".to_string());
    tint(&mut banner, CALM);
    show(&mut banner, true);
}
```

`add_banner` builds the panel and the text node once and returns a `BannerHandles` the game stores in the resource. After that a game touches only `Banner`, whose `line`, `color` and `shown` fields the three commands write. The frame system compares each field against what it last drew, so an unchanged line costs nothing. `built` reports whether the builder has run yet.

## Driving it

```
just example ui_banner
```

Press 1, 2, 3 or 4 to write one of four lines across the top, R to swap the tint between calm and urgent, and Space to take the banner down and put it back up. The readout gives the line the crate is holding, whether it is up, and which tint it carries.
