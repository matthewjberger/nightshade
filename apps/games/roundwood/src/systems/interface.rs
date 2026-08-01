use crate::data::{self, Item};
use crate::queries::interact::shop_name;
use crate::queries::sky::clock;
use crate::queries::town::place_name;
use crate::resources::{Chat, Hud, Pocket, Prompt, Readouts, Sky, Store, Toast, Town, Village};
use nightshade::prelude::*;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

const PLACE_WIDTH: f32 = 300.0;
const POCKET_WIDTH: f32 = 228.0;
const SHOP_WIDTH: f32 = 460.0;
const SHOP_HEIGHT: f32 = 340.0;
const ROW_HEIGHT: f32 = 24.0;
const MARGIN: f32 = 16.0;

pub fn build(mut hud: ResMut<Hud>, mut builder: nightshade_ui::commands::builder::UiBuild) {
    let mut tree = UiTreeBuilder::new(builder.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    let mut handles = Hud::default();

    tree.in_parent(root, |tree| {
        let panel = tree
            .add_node()
            .window(
                Ab(vec2(MARGIN, MARGIN)),
                Ab(vec2(PLACE_WIDTH, 68.0)),
                Anchor::TopLeft,
            )
            .with_rect(10.0, 1.0, data::PANEL_BORDER)
            .color_raw::<UiBase>(data::PANEL_BACKGROUND)
            .with_visible(false)
            .entity();
        handles.place_panel = panel;
        tree.in_parent(panel, |tree| {
            handles.place_text = tree
                .add_node()
                .window(
                    Ab(vec2(16.0, 10.0)),
                    Ab(vec2(PLACE_WIDTH - 32.0, 26.0)),
                    Anchor::TopLeft,
                )
                .with_text(data::TITLE, 21.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_BRIGHT)
                .entity();
            handles.clock_text = tree
                .add_node()
                .window(
                    Ab(vec2(16.0, 38.0)),
                    Ab(vec2(PLACE_WIDTH - 32.0, 20.0)),
                    Anchor::TopLeft,
                )
                .with_text(data::TAGLINE, 15.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_DIM)
                .entity();
        });

        handles.toast_text = tree
            .add_node()
            .window(
                Ab(vec2(MARGIN + 4.0, MARGIN + 78.0)),
                Ab(vec2(PLACE_WIDTH * 1.4, 24.0)),
                Anchor::TopLeft,
            )
            .with_text("", 18.0)
            .text_left()
            .color_raw::<UiBase>(data::TEXT_WARM)
            .entity();

        let pocket = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN, MARGIN)),
                Ab(vec2(
                    POCKET_WIDTH,
                    46.0 + ROW_HEIGHT * Item::ALL.len() as f32,
                )),
                Anchor::TopRight,
            )
            .with_rect(10.0, 1.0, data::PANEL_BORDER)
            .color_raw::<UiBase>(data::PANEL_BACKGROUND)
            .with_visible(false)
            .entity();
        handles.pocket_panel = pocket;
        tree.in_parent(pocket, |tree| {
            handles.bells_text = tree
                .add_node()
                .window(
                    Ab(vec2(14.0, 10.0)),
                    Ab(vec2(POCKET_WIDTH - 28.0, 24.0)),
                    Anchor::TopLeft,
                )
                .with_text("0 bells", 19.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_WARM)
                .entity();
            for (index, row) in handles.pocket_rows.iter_mut().enumerate() {
                *row = tree
                    .add_node()
                    .window(
                        Ab(vec2(14.0, 38.0 + ROW_HEIGHT * index as f32)),
                        Ab(vec2(POCKET_WIDTH - 28.0, ROW_HEIGHT)),
                        Anchor::TopLeft,
                    )
                    .with_text("", 15.0)
                    .text_left()
                    .color_raw::<UiBase>(data::TEXT_DIM)
                    .entity();
            }
        });

        handles.prompt_panel = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -172.0)),
                Ab(vec2(340.0, 46.0)),
                Anchor::BottomCenter,
            )
            .with_rect(10.0, 1.0, data::PANEL_BORDER)
            .color_raw::<UiBase>(data::PANEL_BACKGROUND)
            .with_visible(false)
            .entity();
        tree.in_parent(handles.prompt_panel, |tree| {
            handles.prompt_text = tree
                .add_node()
                .window(Ab(vec2(12.0, 12.0)), Ab(vec2(316.0, 24.0)), Anchor::TopLeft)
                .with_text("", 18.0)
                .text_center()
                .color_raw::<UiBase>(data::TEXT_WARM)
                .entity();
        });

        handles.speech_panel = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -36.0)),
                Ab(vec2(760.0, 118.0)),
                Anchor::BottomCenter,
            )
            .with_rect(12.0, 1.0, data::PANEL_BORDER)
            .color_raw::<UiBase>(data::PANEL_BACKGROUND)
            .with_visible(false)
            .entity();
        tree.in_parent(handles.speech_panel, |tree| {
            handles.speaker_text = tree
                .add_node()
                .window(Ab(vec2(24.0, 14.0)), Ab(vec2(712.0, 22.0)), Anchor::TopLeft)
                .with_text("", 17.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_WARM)
                .entity();
            handles.speech_text = tree
                .add_node()
                .window(Ab(vec2(24.0, 44.0)), Ab(vec2(712.0, 60.0)), Anchor::TopLeft)
                .with_text("", 19.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_BRIGHT)
                .entity();
        });

        handles.shop_panel = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(SHOP_WIDTH, SHOP_HEIGHT)),
                Anchor::Center,
            )
            .with_rect(14.0, 1.0, data::PANEL_BORDER)
            .color_raw::<UiBase>(data::PANEL_BACKGROUND)
            .with_visible(false)
            .entity();
        tree.in_parent(handles.shop_panel, |tree| {
            handles.shop_title = tree
                .add_node()
                .window(
                    Ab(vec2(26.0, 20.0)),
                    Ab(vec2(SHOP_WIDTH - 52.0, 28.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 22.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_BRIGHT)
                .entity();
            for (index, row) in handles.shop_rows.iter_mut().enumerate() {
                *row = tree
                    .add_node()
                    .window(
                        Ab(vec2(26.0, 62.0 + ROW_HEIGHT * index as f32)),
                        Ab(vec2(SHOP_WIDTH - 52.0, ROW_HEIGHT)),
                        Anchor::TopLeft,
                    )
                    .with_text("", 16.0)
                    .text_left()
                    .color_raw::<UiBase>(data::TEXT_DIM)
                    .entity();
            }
            handles.shop_total = tree
                .add_node()
                .window(
                    Ab(vec2(26.0, 74.0 + ROW_HEIGHT * Item::ALL.len() as f32)),
                    Ab(vec2(SHOP_WIDTH - 52.0, 26.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 18.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_WARM)
                .entity();
            let buttons = tree
                .add_node()
                .window(
                    Ab(vec2(26.0, SHOP_HEIGHT - 96.0)),
                    Ab(vec2(SHOP_WIDTH - 52.0, 84.0)),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Vertical, 0.0, 10.0)
                .entity();
            tree.in_parent(buttons, |tree| {
                handles.sell_button = tree.add_button("Sell everything");
                handles.leave_button = tree.add_button("Leave the shop");
            });
        });

        handles.veil = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, data::VEIL)
            .color_raw::<UiBase>(data::VEIL)
            .with_visible(false)
            .entity();

        handles.title_panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(360.0, 220.0)), Anchor::Center)
            .with_rect(14.0, 1.0, data::PANEL_BORDER)
            .color_raw::<UiBase>(data::PANEL_BACKGROUND)
            .with_visible(false)
            .entity();
        tree.in_parent(handles.title_panel, |tree| {
            tree.add_node()
                .window(Ab(vec2(26.0, 26.0)), Ab(vec2(308.0, 30.0)), Anchor::TopLeft)
                .with_text(data::TITLE, 24.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_BRIGHT);
            tree.add_node()
                .window(Ab(vec2(26.0, 58.0)), Ab(vec2(308.0, 22.0)), Anchor::TopLeft)
                .with_text(data::TAGLINE, 15.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_DIM);
            let buttons = tree
                .add_node()
                .window(
                    Ab(vec2(26.0, 106.0)),
                    Ab(vec2(308.0, 90.0)),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Vertical, 0.0, 10.0)
                .entity();
            tree.in_parent(buttons, |tree| {
                handles.start_button = tree.add_button("Walk into the village");
                handles.title_quit_button = tree.add_button("Leave for now");
            });
        });

        handles.pause_panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(360.0, 194.0)), Anchor::Center)
            .with_rect(14.0, 1.0, data::PANEL_BORDER)
            .color_raw::<UiBase>(data::PANEL_BACKGROUND)
            .with_visible(false)
            .entity();
        tree.in_parent(handles.pause_panel, |tree| {
            tree.add_node()
                .window(Ab(vec2(26.0, 26.0)), Ab(vec2(308.0, 30.0)), Anchor::TopLeft)
                .with_text(data::PAUSED, 24.0)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_BRIGHT);
            let buttons = tree
                .add_node()
                .window(Ab(vec2(26.0, 80.0)), Ab(vec2(308.0, 90.0)), Anchor::TopLeft)
                .flow(FlowDirection::Vertical, 0.0, 10.0)
                .entity();
            tree.in_parent(buttons, |tree| {
                handles.resume_button = tree.add_button("Back to the village");
                handles.quit_button = tree.add_button("Leave the village");
            });
        });
    });

    handles.root = tree.root_entity();
    handles.ready = true;
    tree.finish();
    *hud = handles;
}

pub fn refresh(
    village: Res<Village>,
    town: Res<Town>,
    sky: Res<Sky>,
    prompt: Res<Prompt>,
    chat: Res<Chat>,
    hud: Res<Hud>,
    mut readouts: ResMut<Readouts>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if !hud.ready {
        return;
    }

    let place = format!("{}  ·  {}", data::TITLE, place_name(&town, village.feet));
    write_line(
        &mut commands,
        hud.place_text,
        &mut readouts.place,
        place.clone(),
    );
    write_line(
        &mut commands,
        hud.clock_text,
        &mut readouts.clock,
        clock(sky.hour),
    );

    let wanted = prompt.line.clone();
    if !wanted.is_empty() {
        write_line(
            &mut commands,
            hud.prompt_text,
            &mut readouts.prompt,
            wanted.clone(),
        );
    } else {
        readouts.prompt.clear();
    }
    let prompt_open = !wanted.is_empty();
    if prompt_open != readouts.prompt_open {
        readouts.prompt_open = prompt_open;
        ui_set_visible(&mut dirty, &mut commands, hud.prompt_panel, prompt_open);
    }

    if let Some(index) = chat.villager {
        let neighbor = &data::NEIGHBORS[index];
        let line = neighbor
            .lines
            .get(chat.line)
            .copied()
            .unwrap_or_default()
            .to_string();
        write_line(
            &mut commands,
            hud.speaker_text,
            &mut readouts.speaker,
            neighbor.name.to_string(),
        );
        write_line(&mut commands, hud.speech_text, &mut readouts.speech, line);
    }
    if chat.shown != readouts.speech_open {
        readouts.speech_open = chat.shown;
        ui_set_visible(&mut dirty, &mut commands, hud.speech_panel, chat.shown);
    }
}

pub fn show_the_pocket(
    pocket: Res<Pocket>,
    toast: Res<Toast>,
    store: Res<Store>,
    hud: Res<Hud>,
    mut readouts: ResMut<Readouts>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if !hud.ready {
        return;
    }

    write_line(
        &mut commands,
        hud.toast_text,
        &mut readouts.toast,
        toast.line.clone(),
    );
    write_line(
        &mut commands,
        hud.bells_text,
        &mut readouts.bells,
        format!("{} bells", pocket.bells),
    );
    for (index, item) in Item::ALL.into_iter().enumerate() {
        let held = pocket.counts[item.index()];
        let line = if held == 0 {
            String::new()
        } else {
            format!("{held}  {}", item.name())
        };
        let row = hud.pocket_rows[index];
        write_line(&mut commands, row, &mut readouts.pocket[index], line);
    }

    let open = store.keeper.is_some();
    if open != readouts.shop_open {
        readouts.shop_open = open;
        ui_set_visible(&mut dirty, &mut commands, hud.shop_panel, open);
    }
    let Some(keeper) = store.keeper else {
        return;
    };
    let title = format!("{}  ·  we buy what you find", shop_name(keeper));
    write_line(
        &mut commands,
        hud.shop_title,
        &mut readouts.shop_title,
        title,
    );
    let mut total = 0;
    for (index, item) in Item::ALL.into_iter().enumerate() {
        let held = pocket.counts[item.index()];
        let worth = held * item.price();
        total += worth;
        let line = if held == 0 {
            format!("{}  ·  none", item.name())
        } else {
            format!("{held}  {}  ·  {worth} bells", item.name())
        };
        let row = hud.shop_rows[index];
        write_line(&mut commands, row, &mut readouts.shop[index], line);
    }
    write_line(
        &mut commands,
        hud.shop_total,
        &mut readouts.shop_total,
        format!("Everything comes to {total} bells"),
    );
}

fn write_line(commands: &mut Commands, node: Entity, held: &mut String, wanted: String) {
    if *held == wanted {
        return;
    }
    *held = wanted;
    ui_set_text(commands, node, held);
}
