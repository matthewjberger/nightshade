use crate::contract::{BattleCommand, Job, Ware};
use crate::queries::{foe_row, hero_card, purse_line};
use crate::resources::{Battle, Field, Hud, Party, Prompt, Screen, Toast, UiHandles};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::*;

pub fn build(mut hud: ResMut<Hud>, mut build: nightshade_ui::commands::builder::UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let handles = tree.in_parent(root, |tree| {
        let mut handles = UiHandles::default();
        build_field_bars(tree, &mut handles);
        build_title(tree, &mut handles);
        build_battle(tree, &mut handles);
        build_shop(tree, &mut handles);
        build_jobs(tree, &mut handles);
        build_pause(tree, &mut handles);
        build_defeat(tree, &mut handles);
        handles
    });

    tree.finish();
    hud.handles = handles;
}

fn build_field_bars(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    handles.banner = Some(
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, theme::MARGIN)),
                Ab(vec2(theme::BANNER_WIDTH, theme::BANNER_HEIGHT)),
                Anchor::TopCenter,
            )
            .with_text("", theme::LABEL_SIZE)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .with_text_outline(Vec4::new(0.0, 0.0, 0.0, 1.0), 1.2)
            .color_raw::<UiBase>(theme::TEXT_BRIGHT)
            .without_pointer_events()
            .entity(),
    );

    handles.prompt = Some(
        tree.add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::PARTY_HEIGHT - 30.0)),
                Ab(vec2(theme::BANNER_WIDTH, theme::BANNER_HEIGHT)),
                Anchor::BottomCenter,
            )
            .with_text("", theme::LABEL_SIZE)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .with_text_outline(Vec4::new(0.0, 0.0, 0.0, 1.0), 1.2)
            .color_raw::<UiBase>(theme::TEXT_GOLD)
            .without_pointer_events()
            .entity(),
    );

    let bar = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::MARGIN)),
            Ab(vec2(theme::PARTY_WIDTH, theme::PARTY_HEIGHT)),
            Anchor::BottomCenter,
        )
        .with_rect(8.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_BG)
        .flow(FlowDirection::Vertical, 8.0, 4.0)
        .without_pointer_events()
        .entity();
    handles.party = Some(bar);

    tree.in_parent(bar, |tree| {
        let cards = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 40.0)))
            .flow(FlowDirection::Horizontal, 0.0, 6.0)
            .entity();
        tree.in_parent(cards, |tree| {
            for slot in 0..4 {
                handles.party_cards[slot] = Some(
                    tree.add_node()
                        .flow_child(Ab(vec2(theme::HERO_CARD_WIDTH, 40.0)))
                        .with_text("", theme::SMALL_SIZE)
                        .text_left()
                        .color_raw::<UiBase>(theme::TEXT_BRIGHT)
                        .entity(),
                );
            }
        });
        handles.purse = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 20.0)))
                .with_text("", theme::SMALL_SIZE)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(theme::TEXT_GOLD)
                .entity(),
        );
    });
}

fn build_title(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = add_menu_panel(tree, &menu_style(), theme::TITLE_WIDTH, theme::TITLE_HEIGHT);
    handles.title = Some(panel);
    tree.in_parent(panel, |tree| {
        add_menu_text_centered(tree, "CRYSTALBOUND", theme::TITLE_SIZE, theme::TEXT_BRIGHT);
        add_menu_text_centered(
            tree,
            "Cross the moor, take the vault, carry the shards home.",
            theme::LABEL_SIZE,
            theme::TEXT_DIM,
        );
        add_menu_text_centered(
            tree,
            "WASD to walk. Space swings. E speaks. Escape pauses.",
            theme::SMALL_SIZE,
            theme::TEXT_DIM,
        );
        handles.title_play = Some(tree.add_button("Set out"));
        handles.title_quit = Some(tree.add_button("Quit"));
    });
}

fn build_battle(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let foes = tree
        .add_node()
        .window(
            Ab(vec2(theme::MARGIN, theme::MARGIN + 44.0)),
            Ab(vec2(theme::FOE_WIDTH, theme::FOE_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(8.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_BG)
        .flow(FlowDirection::Vertical, 10.0, 6.0)
        .without_pointer_events()
        .entity();
    handles.foes = Some(foes);
    tree.in_parent(foes, |tree| {
        add_menu_text_centered(tree, "AGAINST YOU", theme::SMALL_SIZE, theme::TEXT_DIM);
        for slot in 0..3 {
            handles.foe_rows[slot] = Some(
                tree.add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, theme::ROW_HEIGHT)))
                    .with_text("", theme::LABEL_SIZE)
                    .text_left()
                    .color_raw::<UiBase>(theme::TEXT_BRIGHT)
                    .entity(),
            );
        }
    });

    handles.log = Some(
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, theme::MARGIN + 48.0)),
                Ab(vec2(theme::BANNER_WIDTH, theme::BANNER_HEIGHT)),
                Anchor::TopCenter,
            )
            .with_text("", theme::LABEL_SIZE)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .with_text_outline(Vec4::new(0.0, 0.0, 0.0, 1.0), 1.2)
            .color_raw::<UiBase>(theme::TEXT_BRIGHT)
            .without_pointer_events()
            .entity(),
    );

    let commands = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-theme::MARGIN, -theme::PARTY_HEIGHT - 28.0)),
            Ab(vec2(theme::COMMAND_WIDTH, theme::COMMAND_HEIGHT)),
            Anchor::BottomRight,
        )
        .with_rect(8.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_BG)
        .flow(FlowDirection::Vertical, 10.0, 6.0)
        .entity();
    handles.commands = Some(commands);
    tree.in_parent(commands, |tree| {
        add_menu_text_centered(tree, "COMMAND", theme::SMALL_SIZE, theme::TEXT_DIM);
        for (slot, command) in BattleCommand::ALL.iter().enumerate() {
            handles.command_buttons[slot] = Some(tree.add_button(command.name()));
        }
    });
}
fn build_shop(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = add_menu_panel(tree, &menu_style(), theme::SHOP_WIDTH, theme::SHOP_HEIGHT);
    handles.shop = Some(panel);
    tree.in_parent(panel, |tree| {
        add_menu_text(tree, "THE SHOP", theme::HEADING_SIZE, theme::TEXT_BRIGHT);
        handles.shop_purse = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, theme::ROW_HEIGHT)))
                .with_text("", theme::LABEL_SIZE)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(theme::TEXT_GOLD)
                .entity(),
        );
        for (slot, ware) in Ware::ALL.iter().enumerate() {
            handles.shop_buttons[slot] =
                Some(tree.add_button(&format!("{}   {} gold", ware.name(), ware.price())));
        }
        handles.shop_leave = Some(tree.add_button("Leave"));
    });
}

fn build_jobs(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = add_menu_panel(tree, &menu_style(), theme::JOBS_WIDTH, theme::JOBS_HEIGHT);
    handles.jobs = Some(panel);
    tree.in_parent(panel, |tree| {
        add_menu_text_centered(tree, "THE CRYSTAL", theme::HEADING_SIZE, theme::TEXT_BRIGHT);
        handles.jobs_note = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, theme::ROW_HEIGHT)))
                .with_text("", theme::LABEL_SIZE)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(theme::TEXT_DIM)
                .entity(),
        );

        let heroes = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 42.0)))
            .flow(FlowDirection::Horizontal, 0.0, 6.0)
            .entity();
        tree.in_parent(heroes, |tree| {
            for slot in 0..4 {
                handles.jobs_heroes[slot] = Some(tree.add_button(crate::data::HERO_NAMES[slot]));
            }
        });

        let ways = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 42.0)))
            .flow(FlowDirection::Horizontal, 0.0, 6.0)
            .entity();
        tree.in_parent(ways, |tree| {
            for (slot, job) in Job::LADDER.iter().enumerate() {
                handles.jobs_list[slot] = Some(tree.add_button(job.name()));
            }
        });

        handles.jobs_unlock = Some(tree.add_button("Feed the crystal a shard"));
        handles.jobs_leave = Some(tree.add_button("Step away"));
    });
}

fn build_pause(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = add_menu_panel(tree, &menu_style(), theme::MENU_WIDTH, theme::MENU_HEIGHT);
    handles.paused = Some(panel);
    tree.in_parent(panel, |tree| {
        add_menu_text_centered(tree, "Paused", theme::HEADING_SIZE, theme::TEXT_BRIGHT);
        handles.paused_resume = Some(tree.add_button("Back to it"));
        handles.paused_title = Some(tree.add_button("Give up"));
    });
}

fn build_defeat(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = add_menu_panel(tree, &menu_style(), theme::MENU_WIDTH, theme::MENU_HEIGHT);
    handles.defeat = Some(panel);
    tree.in_parent(panel, |tree| {
        add_menu_text_centered(
            tree,
            "The party falls",
            theme::HEADING_SIZE,
            theme::TEXT_HURT,
        );
        add_menu_text_centered(
            tree,
            "The moor keeps what it takes.",
            theme::SMALL_SIZE,
            theme::TEXT_DIM,
        );
        handles.defeat_title = Some(tree.add_button("Begin again"));
    });
}

pub fn refresh(
    mut hud: ResMut<Hud>,
    party: Res<Party>,
    field: Res<Field>,
    battle: Res<Battle>,
    prompt: Res<Prompt>,
    toast: Res<Toast>,
    mut commands: Commands,
) {
    let hud = &mut *hud;
    for slot in 0..4 {
        let text = hero_card(&party, slot);
        ui_set_text_cached(
            &mut commands,
            hud.handles.party_cards[slot],
            &mut hud.last.cards[slot],
            text,
        );
    }
    let purse = purse_line(&party);
    ui_set_text_cached(
        &mut commands,
        hud.handles.purse,
        &mut hud.last.purse,
        purse.clone(),
    );
    ui_set_text_cached(
        &mut commands,
        hud.handles.shop_purse,
        &mut hud.last.shop_purse,
        purse,
    );

    let banner = if toast.life > 0.0 {
        toast.text.clone()
    } else {
        field.place.name().to_string()
    };
    ui_set_text_cached(
        &mut commands,
        hud.handles.banner,
        &mut hud.last.banner,
        banner,
    );
    let hint = prompt.text.clone();
    ui_set_text_cached(
        &mut commands,
        hud.handles.prompt,
        &mut hud.last.prompt,
        hint,
    );

    for slot in 0..3 {
        let text = if slot < battle.foes.len() {
            foe_row(&battle, slot)
        } else {
            String::new()
        };
        ui_set_text_cached(
            &mut commands,
            hud.handles.foe_rows[slot],
            &mut hud.last.foes[slot],
            text,
        );
    }
    let log = battle.log.clone();
    ui_set_text_cached(&mut commands, hud.handles.log, &mut hud.last.log, log);

    let chosen = party
        .heroes
        .get(party.chosen)
        .map(|hero| hero.name)
        .unwrap_or("");
    let note = format!(
        "Shards {}.  {} stands before it.  Open ways: {}.",
        party.shards,
        chosen,
        party
            .unlocked
            .iter()
            .map(|job| job.name())
            .collect::<Vec<_>>()
            .join(", ")
    );
    ui_set_text_cached(
        &mut commands,
        hud.handles.jobs_note,
        &mut hud.last.jobs_note,
        note,
    );
}

pub fn fade_the_toast(time: Res<Time>, mut toast: ResMut<Toast>) {
    toast.life = (toast.life - time.delta_time).max(0.0);
}

pub fn show_the_screen(
    mut hud: ResMut<Hud>,
    screen: Res<crate::resources::ShownScreen>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if hud.shown == Some(screen.0) {
        return;
    }
    hud.shown = Some(screen.0);
    let handles = hud.handles;
    let now = screen.0;
    let field_like = matches!(
        now,
        Screen::Field | Screen::Shop | Screen::Jobs | Screen::Paused
    );

    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.title,
        now == Screen::Title,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.paused,
        now == Screen::Paused,
    );
    ui_set_visible(&mut dirty, &mut commands, handles.shop, now == Screen::Shop);
    ui_set_visible(&mut dirty, &mut commands, handles.jobs, now == Screen::Jobs);
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.defeat,
        now == Screen::Defeat,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.party,
        field_like || now == Screen::Battle,
    );
    ui_set_visible(&mut dirty, &mut commands, handles.banner, field_like);
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.prompt,
        now == Screen::Field,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.foes,
        now == Screen::Battle,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.log,
        now == Screen::Battle,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.commands,
        now == Screen::Battle,
    );
}

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: theme::PANEL_BG,
        border: theme::PANEL_BORDER,
        padding: 16.0,
        spacing: 8.0,
        centered: true,
        ..Default::default()
    }
}
