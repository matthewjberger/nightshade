use crate::components::{Focus, Kind, Level, Tally};
use crate::contract::Tower;
use crate::data;
use crate::queries;
use crate::resources::{Bulwark, Chips, Hud, Inspect, Slot};
use crate::theme;
use nightshade::prelude::*;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::data::icons::{IconGlyph, material};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_waves::WaveProgress;

fn chip(
    tree: &mut UiTreeBuilder,
    index: usize,
    label: &str,
    mark: IconGlyph,
    color: [f32; 4],
) -> Entity {
    let step = theme::CHIP_WIDTH + theme::CHIP_GAP;
    let panel = tree
        .add_node()
        .window(
            Ab(vec2(
                theme::HUD_MARGIN + step * index as f32,
                theme::HUD_MARGIN,
            )),
            Ab(vec2(theme::CHIP_WIDTH, theme::CHIP_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(9.0, 1.0, Vec4::from(theme::HUD_BORDER))
        .color_raw::<UiBase>(Vec4::from(theme::HUD_PANEL))
        .entity();

    let mut value = Entity::default();
    tree.in_parent(panel, |tree| {
        tree.add_node()
            .window(
                Ab(vec2(12.0, 7.0)),
                Ab(vec2(theme::CHIP_WIDTH - 24.0, 16.0)),
                Anchor::TopLeft,
            )
            .with_text(label, theme::LABEL_SIZE)
            .text_left()
            .color_raw::<UiBase>(Vec4::from(theme::HUD_LABEL));
        tree.add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-9.0, 8.0)),
                Ab(vec2(22.0, 22.0)),
                Anchor::TopRight,
            )
            .with_icon(mark, 19.0)
            .color_raw::<UiBase>(Vec4::from(color));
        value = tree
            .add_node()
            .window(
                Ab(vec2(12.0, 23.0)),
                Ab(vec2(theme::CHIP_WIDTH - 24.0, 30.0)),
                Anchor::TopLeft,
            )
            .with_text("", theme::VALUE_SIZE)
            .text_left()
            .color_raw::<UiBase>(Vec4::from(color))
            .entity();
    });
    value
}

fn hotbar_slot(tree: &mut UiTreeBuilder, index: usize, tower: Tower) -> Slot {
    let step = theme::HOTBAR_WIDTH + theme::HOTBAR_GAP;
    let middle = (Tower::ALL.len() as f32 - 1.0) * 0.5;
    let across = (index as f32 - middle) * step;
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(across, -theme::HUD_MARGIN)),
            Ab(vec2(theme::HOTBAR_WIDTH, theme::HOTBAR_HEIGHT)),
            Anchor::BottomCenter,
        )
        .with_rect(9.0, 1.0, Vec4::from(theme::HUD_BORDER))
        .color_raw::<UiBase>(Vec4::from(theme::HOTBAR_IDLE))
        .flow(FlowDirection::Vertical, 11.0, 5.0)
        .entity();

    let mut slot = Slot {
        panel,
        ..Default::default()
    };
    tree.in_parent(panel, |tree| {
        slot.name = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 22.0)))
            .with_text(tower.name(), theme::HOTBAR_NAME_SIZE)
            .text_left()
            .color_raw::<UiBase>(Vec4::from(theme::HUD_VALUE))
            .entity();
        slot.blurb = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 16.0)))
            .with_text(data::TOWER_BLURB[tower.slot()], theme::HOTBAR_BLURB_SIZE)
            .text_left()
            .color_raw::<UiBase>(Vec4::from(theme::HUD_DIM))
            .entity();
        slot.cost = tree.add_button("");
    });
    slot
}

fn inspect_panel(tree: &mut UiTreeBuilder) -> Inspect {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-theme::HUD_MARGIN, -theme::HUD_MARGIN)),
            Ab(vec2(theme::INSPECT_WIDTH, theme::INSPECT_HEIGHT)),
            Anchor::BottomRight,
        )
        .with_rect(9.0, 1.0, Vec4::from(theme::HUD_BORDER))
        .color_raw::<UiBase>(Vec4::from(theme::HUD_PANEL))
        .flow(FlowDirection::Vertical, 14.0, 8.0)
        .with_visible(false)
        .entity();

    let mut inspect = Inspect {
        panel,
        ..Default::default()
    };
    tree.in_parent(panel, |tree| {
        inspect.name = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)))
            .with_text("", 21.0)
            .text_left()
            .color_raw::<UiBase>(Vec4::from(theme::HUD_VALUE))
            .entity();
        inspect.stats = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 18.0)))
            .with_text("", 13.0)
            .text_left()
            .color_raw::<UiBase>(Vec4::from(theme::HUD_DIM))
            .entity();
        inspect.reach = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 18.0)))
            .with_text("", 13.0)
            .text_left()
            .color_raw::<UiBase>(Vec4::from(theme::HUD_DIM))
            .entity();
        inspect.upgrade = tree.add_button("Raise");
        inspect.aim = tree.add_button("Aim");
        inspect.sell = tree.add_button("Pull down");
    });
    inspect
}

pub fn build(tree: &mut UiTreeBuilder) -> Hud {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut hud = Hud {
        root,
        ..Default::default()
    };

    tree.in_parent(root, |tree| {
        hud.chips = Chips {
            coin: chip(tree, 0, "COIN", material::TOKEN, theme::HUD_COIN),
            lives: chip(tree, 1, "LIVES", material::HOME, theme::HUD_LIVES),
            wave: chip(tree, 2, "WAVE", material::TIMER, theme::HUD_WAVE),
            kills: chip(tree, 3, "BROKEN", material::AUTO_AWESOME, theme::HUD_GOOD),
        };

        for (index, tower) in Tower::ALL.into_iter().enumerate() {
            hud.slots[index] = hotbar_slot(tree, index, tower);
        }

        hud.inspect = inspect_panel(tree);

        hud.banner = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-theme::HUD_MARGIN, theme::HUD_MARGIN)),
                Ab(vec2(theme::BANNER_WIDTH, theme::BANNER_HEIGHT)),
                Anchor::TopRight,
            )
            .with_rect(9.0, 1.0, Vec4::from(theme::HUD_BORDER))
            .color_raw::<UiBase>(Vec4::from(theme::HUD_PANEL))
            .entity();

        tree.in_parent(hud.banner, |tree| {
            hud.banner_text = tree
                .add_node()
                .window(
                    Ab(vec2(18.0, 16.0)),
                    Ab(vec2(theme::BANNER_WIDTH - 36.0, 28.0)),
                    Anchor::TopLeft,
                )
                .with_text("", theme::BANNER_TEXT_SIZE)
                .text_left()
                .color_raw::<UiBase>(Vec4::from(theme::HUD_VALUE))
                .entity();
            hud.banner_note = tree
                .add_node()
                .window(
                    Ab(vec2(18.0, 50.0)),
                    Ab(vec2(theme::BANNER_WIDTH - 36.0, 20.0)),
                    Anchor::TopLeft,
                )
                .with_text("", theme::BANNER_NOTE_SIZE)
                .text_left()
                .color_raw::<UiBase>(Vec4::from(theme::HUD_LABEL))
                .entity();
        });

        let send_panel = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0))
                    + Ab(vec2(
                        -theme::HUD_MARGIN,
                        theme::HUD_MARGIN + theme::BANNER_HEIGHT + 10.0,
                    )),
                Ab(vec2(theme::SEND_WIDTH, theme::SEND_HEIGHT)),
                Anchor::TopRight,
            )
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        tree.in_parent(send_panel, |tree| {
            hud.send = tree.add_button("Send now");
        });

        let pace_panel = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0))
                    + Ab(vec2(
                        -theme::HUD_MARGIN - theme::SEND_WIDTH - 10.0,
                        theme::HUD_MARGIN + theme::BANNER_HEIGHT + 10.0,
                    )),
                Ab(vec2(theme::PACE_WIDTH, theme::SEND_HEIGHT)),
                Anchor::TopRight,
            )
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        tree.in_parent(pace_panel, |tree| {
            hud.pace = tree.add_button("x1");
        });

        let mend_panel = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0))
                    + Ab(vec2(
                        -theme::HUD_MARGIN - theme::SEND_WIDTH - theme::PACE_WIDTH - 20.0,
                        theme::HUD_MARGIN + theme::BANNER_HEIGHT + 10.0,
                    )),
                Ab(vec2(theme::MEND_WIDTH, theme::SEND_HEIGHT)),
                Anchor::TopRight,
            )
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        tree.in_parent(mend_panel, |tree| {
            hud.mend = tree.add_button("");
        });

        let hold_panel = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0))
                    + Ab(vec2(
                        -theme::HUD_MARGIN
                            - theme::SEND_WIDTH
                            - theme::PACE_WIDTH
                            - theme::MEND_WIDTH
                            - 30.0,
                        theme::HUD_MARGIN + theme::BANNER_HEIGHT + 10.0,
                    )),
                Ab(vec2(theme::HOLD_WIDTH, theme::SEND_HEIGHT)),
                Anchor::TopRight,
            )
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        tree.in_parent(hold_panel, |tree| {
            hud.hold = tree.add_button("Hold");
        });
    });

    hud
}

pub fn show_the_chips(
    mut bulwark: ResMut<Bulwark>,
    progress: Res<WaveProgress>,
    mut commands: Commands,
) {
    if bulwark.hud.root == Entity::default() {
        return;
    }
    let lines = [
        bulwark.coin.to_string(),
        bulwark.lives.to_string(),
        queries::wave_label(&progress),
        bulwark.kills.to_string(),
    ];
    let handles = [
        bulwark.hud.chips.coin,
        bulwark.hud.chips.lives,
        bulwark.hud.chips.wave,
        bulwark.hud.chips.kills,
    ];
    for slot in 0..lines.len() {
        if bulwark.last_chips[slot] == lines[slot] {
            continue;
        }
        ui_set_text(&mut commands, handles[slot], &lines[slot]);
        bulwark.last_chips[slot] = lines[slot].clone();
    }
}

pub fn show_the_hotbar(mut bulwark: ResMut<Bulwark>, mut commands: Commands) {
    if bulwark.hud.root == Entity::default() {
        return;
    }
    let stamp = Some((bulwark.chosen, bulwark.coin));
    if stamp == bulwark.last_hotbar {
        return;
    }
    bulwark.last_hotbar = stamp;
    for (index, tower) in Tower::ALL.into_iter().enumerate() {
        let slot = bulwark.hud.slots[index];
        let cost = queries::build_cost(tower);
        let chosen = bulwark.chosen == tower;
        let color = if chosen {
            theme::HOTBAR_CHOSEN
        } else {
            theme::HOTBAR_IDLE
        };
        let blurb_color = if queries::affordable(&bulwark, cost) {
            theme::HUD_DIM
        } else {
            theme::HUD_LIVES
        };
        ui_set_color::<UiBase>(&mut commands, slot.panel, Vec4::from(color));
        ui_set_color::<UiBase>(&mut commands, slot.blurb, Vec4::from(blurb_color));
        ui_set_text(&mut commands, slot.cost, &format!("{cost} coin"));
    }
}

pub fn show_the_banner(
    mut bulwark: ResMut<Bulwark>,
    progress: Res<WaveProgress>,
    mut commands: Commands,
) {
    if bulwark.hud.root == Entity::default() {
        return;
    }
    let line = queries::banner_line(&progress);
    if line != bulwark.last_banner {
        ui_set_text(&mut commands, bulwark.hud.banner_text, &line);
        bulwark.last_banner = line;
    }
    let note = queries::banner_note(&bulwark, &progress);
    if note != bulwark.last_note {
        ui_set_text(&mut commands, bulwark.hud.banner_note, &note);
        bulwark.last_note = note;
    }
}

pub fn show_the_inspection(
    mut bulwark: ResMut<Bulwark>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    towers: Query<(&Kind, &Level, &Focus, &Tally)>,
) {
    if bulwark.hud.root == Entity::default() {
        return;
    }
    let chosen = bulwark.selected;
    let mut found = None;
    towers.for_each(|entity, (kind, level, focus, tally)| {
        if Some(entity) == chosen {
            found = Some((kind.0, level.0, focus.0, tally.0));
        }
    });

    let panel = bulwark.hud.inspect.panel;
    let showing = found.is_some();
    if showing != bulwark.last_inspect {
        ui_set_visible(&mut dirty, &mut commands, panel, showing);
        bulwark.last_inspect = showing;
    }
    let Some((kind, level, focus, tally)) = found else {
        bulwark.last_shown = None;
        return;
    };

    let stamp = chosen.map(|entity| (entity, level, focus, tally, bulwark.coin));
    if stamp == bulwark.last_shown {
        return;
    }
    bulwark.last_shown = stamp;

    let inspect = bulwark.hud.inspect;
    ui_set_text(
        &mut commands,
        inspect.name,
        &queries::inspect_name(kind, tally),
    );
    ui_set_text(
        &mut commands,
        inspect.stats,
        &queries::inspect_stats(kind, level),
    );
    ui_set_text(
        &mut commands,
        inspect.reach,
        &queries::inspect_reach(kind, level),
    );
    let raise = match queries::upgrade_cost(kind, level) {
        Some(cost) => format!("Raise   {cost}"),
        None => "Fully raised".to_string(),
    };
    ui_set_text(&mut commands, inspect.upgrade, &raise);
    ui_set_text(&mut commands, inspect.aim, queries::focus_label(focus));
    ui_set_text(
        &mut commands,
        inspect.sell,
        &format!("Pull down   {}", queries::sell_value(kind, level)),
    );
}

pub fn show_the_send(
    mut bulwark: ResMut<Bulwark>,
    progress: Res<WaveProgress>,
    mut commands: Commands,
) {
    if bulwark.hud.root == Entity::default() {
        return;
    }
    let label = match progress.resting {
        true => "Send now",
        false => "Marching",
    };
    if bulwark.last_send != label {
        ui_set_text(&mut commands, bulwark.hud.send, label);
        bulwark.last_send = label.to_string();
    }
    if bulwark.last_pace != bulwark.pace_step {
        let pace = queries::pace_label(&bulwark);
        ui_set_text(&mut commands, bulwark.hud.pace, &pace);
        bulwark.last_pace = bulwark.pace_step;
    }
    let mend = queries::mend_label(&bulwark);
    if bulwark.last_mend != mend {
        ui_set_text(&mut commands, bulwark.hud.mend, &mend);
        bulwark.last_mend = mend;
    }
}
