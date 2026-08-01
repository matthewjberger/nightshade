# nightshade-ui-menu

The card a game shows when it is not playing, a heading over some rows and a column of buttons. A game builds one panel for each screen and says which screen owns it, and the crate shows the panel whose screen the game is on and hides the rest. Most of the games in `apps/games/` carry their title, pause and end screens this way.

## Composing it

```rust
app.add_plugin(nightshade_state::StatePlugin::new(Screen::Title));
app.add_plugin(nightshade_ui::UiPlugin::new());
app.add_plugin(nightshade_ui_menu::MenuPlugin::<Screen>::new(
    nightshade_ui_menu::data::dark_menu_style(),
));
```

`MenuPlugin` is generic over the screen type and inserts the `MenuStyle` it is given beside an empty `Menus<S>`. Its one system runs on `Stage::Update` and follows the `State<S>` that `nightshade-state` keeps.

## Using it

```rust
fn build_the_menus(style: Res<MenuStyle>, mut menus: ResMut<Menus<Screen>>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let title = add_menu(
        &mut tree,
        &style,
        420.0,
        260.0,
        "The Menu",
        &["A panel with a heading, some rows and a few buttons."],
        &["Play", "Quit"],
    );
    tree.finish();
    menu_show_on(&mut menus, Screen::Title, title.panel);
}
```

`add_menu` builds the panel, the heading, one row for each line and one button for each label, and hands back a `MenuHandles` naming every entity so a game can read the buttons later. `menu_show_on` records which screen the panel belongs to. The pieces are also available on their own as `add_menu_panel`, `add_menu_heading`, `add_menu_row`, `add_menu_text` and `add_menu_text_centered` for a panel that wants a different arrangement. `MenuStyle` carries the card colors, the padding, the text sizes and whether rows are centered, and `dark_menu_style` is the darker card a paused game uses.

## The card furniture

A game that lays a screen out as a card with a title bar, rows and a footer takes the pieces from the same crate rather than drawing them again. `CardStyle` carries the sizes and the colors of that furniture, the card border and radius, the frame corner and tint, the title and footer bands, the row height and its two alternating colors, the text sizes and colors, the action button, the bar and the item cell. A game inserts one `CardStyle` as a resource and hands it to every builder.

```rust
let panel = add_card(&mut tree, &style, frame_layer, 660.0, 400.0, style.card_background);
tree.in_parent(panel, |tree| {
    add_title_bar(tree, &style, 660.0, "Settings");
    let body = add_body_column(tree, &style, 660.0, 294.0);
    tree.in_parent(body, |tree| {
        add_section_label(tree, &style, "How you move", 600.0);
        let strip = add_row_strip(tree, &style, 0, 600.0);
        tree.in_parent(strip, |tree| {
            add_cell_text(tree, &style, "Watch the world", 396.0, style.body_size, style.text_color);
            add_action_button(tree, &style, "Off");
        });
        let cells = add_slot_row(tree, &style, 600.0, 9);
        let (fill, text) = add_bar(tree, &style, 12.0, 34.0, 200.0, style.bright_color);
    });
    let buttons = add_footer_buttons(tree, &style, 660.0, &["Back", "Save"]);
});
```

`add_frame` lays a nine-slice picture over the edges of the node it is called in, from a layer the game uploaded, and does nothing when no layer is given. `add_card` is the centered hidden card with that frame, and `add_corner_card` is the same card pinned at a left and top offset. `add_title_bar` and `add_footer_bar` are the bands at the top and the bottom, `add_footer_buttons` fills the footer with evenly spaced buttons and returns them, and `add_body_column` and `add_scrolling_body` are the column between, plain or scrolling. `add_section_label`, `add_row_strip`, `add_cell_text` and `add_stat_row` build the rows, `add_action_button` and `add_wide_button` the buttons in them, `add_bar` a filled bar with a caption that `set_bar_fill` moves, and `add_slot_row` a row of item cells whose `SlotHandles` name the frame, the icon and the count of each cell.

## Driving it

```
just example menu
```

Click Play on the title card to reach the play screen, Resume or Quit to title on the pause card, and Again on the end card. One panel stands at a time and the crate swaps it as the screen changes.
