use crate::components::{Needs, Route, Using};
use crate::data;
use crate::queries;
use crate::resources::{Aim, Interface, Lot, Models};
use nightshade::prelude::*;
use nightshade_text::prelude::{TextAlignment, VerticalAlignment};
use nightshade_ui::components::layout_types::{FlowDirection, WindowLayout};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn build(mut lot: ResMut<Lot>, mut build: nightshade_ui::commands::builder::UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut interface = Interface::default();
    tree.in_parent(root, |tree| {
        build_money(tree, &mut interface);
        build_needs(tree, &mut interface);
        build_status(tree, &mut interface);
        build_palette(tree, &mut interface);
    });
    tree.finish();
    lot.interface = interface;
}

fn build_money(tree: &mut UiTreeBuilder, interface: &mut Interface) {
    let panel = tree
        .add_node()
        .window(
            Ab(vec2(data::MARGIN, data::MARGIN)),
            Ab(vec2(data::PANEL_WIDTH, data::MONEY_PANEL_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(8.0, 1.0, data::PANEL_BORDER)
        .color_raw::<UiBase>(data::PANEL_BACKGROUND)
        .entity();

    tree.in_parent(panel, |tree| {
        interface.money_readout = tree
            .add_node()
            .window(
                Ab(vec2(14.0, 10.0)),
                Ab(vec2(data::PANEL_WIDTH - 28.0, 32.0)),
                Anchor::TopLeft,
            )
            .with_text("", data::MONEY_SIZE)
            .text_left()
            .color_raw::<UiBase>(data::TEXT_PRIMARY)
            .entity();

        interface.wage_readout = tree
            .add_node()
            .window(
                Ab(vec2(14.0, 46.0)),
                Ab(vec2(data::PANEL_WIDTH - 28.0, 22.0)),
                Anchor::TopLeft,
            )
            .with_text("", data::LABEL_SIZE)
            .text_left()
            .color_raw::<UiBase>(data::TEXT_DIM)
            .entity();
    });
}

fn build_needs(tree: &mut UiTreeBuilder, interface: &mut Interface) {
    let top = data::MARGIN + data::MONEY_PANEL_HEIGHT + 8.0;
    let panel = tree
        .add_node()
        .window(
            Ab(vec2(data::MARGIN, top)),
            Ab(vec2(data::PANEL_WIDTH, data::NEEDS_PANEL_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(8.0, 1.0, data::PANEL_BORDER)
        .color_raw::<UiBase>(data::PANEL_BACKGROUND)
        .entity();

    tree.in_parent(panel, |tree| {
        tree.add_node()
            .window(
                Ab(vec2(14.0, 10.0)),
                Ab(vec2(data::PANEL_WIDTH - 28.0, 18.0)),
                Anchor::TopLeft,
            )
            .with_text("NEEDS", 12.0)
            .text_left()
            .color_raw::<UiBase>(data::TEXT_DIM);

        for (index, need) in data::NEEDS.iter().enumerate() {
            let row = 34.0 + index as f32 * data::ROW_HEIGHT;

            tree.add_node()
                .window(
                    Ab(vec2(14.0, row + 2.0)),
                    Ab(vec2(58.0, data::BAR_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_text(data::need_name(*need), data::LABEL_SIZE)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_PRIMARY);

            let track = tree
                .add_node()
                .window(
                    Ab(vec2(data::BAR_LEFT, row + 3.0)),
                    Ab(vec2(data::BAR_WIDTH, data::BAR_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_rect(4.0, 0.0, data::PANEL_BORDER)
                .color_raw::<UiBase>(data::TRACK_COLOR)
                .entity();

            let fill = tree.in_parent(track, |tree| {
                tree.add_node()
                    .window(
                        Ab(vec2(0.0, 0.0)),
                        Ab(vec2(data::BAR_WIDTH, data::BAR_HEIGHT)),
                        Anchor::TopLeft,
                    )
                    .with_rect(4.0, 0.0, data::PANEL_BORDER)
                    .color_raw::<UiBase>(data::NEED_COLOR[index])
                    .entity()
            });
            interface.need_fills.push(fill);

            let readout = tree
                .add_node()
                .window(
                    Ab(vec2(data::READOUT_LEFT, row + 2.0)),
                    Ab(vec2(52.0, data::BAR_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_text("", data::LABEL_SIZE)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_DIM)
                .entity();
            interface.need_readouts.push(readout);
        }
    });
}

fn build_status(tree: &mut UiTreeBuilder, interface: &mut Interface) {
    interface.status_readout = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, data::MARGIN)),
            Ab(vec2(data::STATUS_WIDTH, data::STATUS_HEIGHT)),
            Anchor::TopCenter,
        )
        .with_text("", data::LABEL_SIZE)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_text_outline(Vec4::new(0.0, 0.0, 0.0, 0.85), 1.0)
        .color_raw::<UiBase>(data::TEXT_PRIMARY)
        .without_pointer_events()
        .entity();
}

fn build_palette(tree: &mut UiTreeBuilder, interface: &mut Interface) {
    let width = queries::palette_width();
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -data::MARGIN)),
            Ab(vec2(width, data::PALETTE_HEIGHT)),
            Anchor::BottomCenter,
        )
        .with_rect(8.0, 1.0, data::PANEL_BORDER)
        .color_raw::<UiBase>(data::PANEL_BACKGROUND)
        .entity();

    tree.in_parent(panel, |tree| {
        for (index, category) in data::CATEGORIES.iter().enumerate() {
            let left = data::PALETTE_GAP + index as f32 * (data::TAB_WIDTH + data::PALETTE_GAP);
            let holder = tree
                .add_node()
                .window(
                    Ab(vec2(left, 8.0)),
                    Ab(vec2(data::TAB_WIDTH, data::TAB_HEIGHT)),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();
            let tab = tree.in_parent(holder, |tree| {
                tree.add_button(data::category_name(*category))
            });
            interface.tabs.push(tab);
        }

        let widest = data::CATEGORIES
            .iter()
            .map(|category| {
                data::CATALOG
                    .iter()
                    .filter(|piece| piece.category == *category)
                    .count()
            })
            .max()
            .unwrap_or(1);
        let row = data::TAB_HEIGHT + 16.0;
        for slot in 0..widest {
            let left =
                data::PALETTE_GAP + slot as f32 * (data::PALETTE_SLOT_WIDTH + data::PALETTE_GAP);
            let holder = tree
                .add_node()
                .window(
                    Ab(vec2(left, row)),
                    Ab(vec2(data::PALETTE_SLOT_WIDTH, data::PALETTE_SLOT_HEIGHT)),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();
            let button = tree.in_parent(holder, |tree| tree.add_button(""));
            interface.palette_slots.push(button);
        }
    });
}

pub fn show_money(mut lot: ResMut<Lot>, mut commands: Commands) {
    let money = lot.money.floor() as i32;
    let wage = lot.wage.round() as i32;
    if money != lot.last_money {
        lot.last_money = money;
        let readout = lot.interface.money_readout;
        ui_set_text(&mut commands, readout, &format!("{money}"));
    }
    if wage != lot.last_wage {
        lot.last_wage = wage;
        let readout = lot.interface.wage_readout;
        ui_set_text(&mut commands, readout, &format!("{wage} an hour"));
    }
}

pub fn show_needs(
    mut lot: ResMut<Lot>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    dwellers: Query<(&Needs,)>,
) {
    let mut levels = None;
    dwellers.for_each(|_entity, (needs,)| levels = Some(needs.0));
    let Some(levels) = levels else {
        return;
    };
    if lot.interface.need_fills.len() != levels.len() {
        return;
    }

    for (index, level) in levels.iter().enumerate() {
        let shown = level.round() as i32;
        if shown == lot.last_needs[index] {
            continue;
        }
        lot.last_needs[index] = shown;

        let fraction = (level / data::NEED_FULL).clamp(0.0, 1.0);
        let fill = lot.interface.need_fills[index];
        ui_set_window_layout(
            &mut dirty,
            &mut commands,
            fill,
            WindowLayout {
                position: Ab(vec2(0.0, 0.0)).into(),
                size: Ab(vec2(data::BAR_WIDTH * fraction, data::BAR_HEIGHT)).into(),
                anchor: Anchor::TopLeft,
            },
        );

        let readout = lot.interface.need_readouts[index];
        ui_set_text(&mut commands, readout, &format!("{shown}"));
    }
}

pub fn show_palette(mut lot: ResMut<Lot>, mut commands: Commands) {
    let slots = queries::slots(&lot);
    let count = lot.interface.palette_slots.len();
    if count == 0 {
        return;
    }
    if lot.last_labels.len() != count {
        lot.last_labels = vec![String::new(); count];
        lot.last_disabled = vec![false; count];
    }
    if lot.last_tabs.len() != lot.interface.tabs.len() {
        lot.last_tabs = vec![String::new(); lot.interface.tabs.len()];
    }

    for (index, category) in data::CATEGORIES.iter().enumerate() {
        let marker = if *category == lot.category { "> " } else { "" };
        let label = format!("{}{}", marker, data::category_name(*category));
        if label != lot.last_tabs[index] {
            lot.last_tabs[index] = label.clone();
            let tab = lot.interface.tabs[index];
            ui_set_text(&mut commands, tab, &label);
        }
    }

    for slot in 0..count {
        let button = lot.interface.palette_slots[slot];
        let (label, blocked) = match slots.get(slot) {
            Some(index) => {
                let piece = &data::CATALOG[*index];
                let marker = if *index == lot.selected { "> " } else { "" };
                (
                    format!("{}{} {}", marker, piece.label, piece.price),
                    !queries::affordable(&lot, *index),
                )
            }
            None => (String::new(), true),
        };
        if label != lot.last_labels[slot] {
            lot.last_labels[slot] = label.clone();
            ui_set_text(&mut commands, button, &label);
        }
        if blocked != lot.last_disabled[slot] {
            lot.last_disabled[slot] = blocked;
            ui_set_disabled(&mut commands, button, blocked);
        }
    }
}

pub fn show_status(
    mut lot: ResMut<Lot>,
    models: Res<Models>,
    mut commands: Commands,
    dwellers: Query<(&Needs, &Route, &Using)>,
) {
    let mut doing = String::from("waiting");
    dwellers.for_each(|_entity, (needs, route, using)| {
        doing = if !route.0.is_empty() {
            format!(
                "walking to {}",
                data::need_name(queries::lowest_need(needs))
            )
        } else if using.0.is_some() {
            format!("seeing to {}", data::need_name(queries::lowest_need(needs)))
        } else {
            "idle, with nothing to use".to_string()
        };
    });

    let piece = &data::CATALOG[lot.selected];
    let status = if !models.ready {
        "loading the kits".to_string()
    } else if lot.aim == Aim::Nowhere {
        format!("{} {}   the dweller is {}", piece.label, piece.price, doing)
    } else if !lot.clear {
        match piece.placement {
            data::Placement::Floor => "there is already a floor there".to_string(),
            data::Placement::Cell => format!("{} needs an empty floor tile", piece.label),
            data::Placement::Edge => "that wall space is taken".to_string(),
        }
    } else if !queries::affordable(&lot, lot.selected) {
        format!("{} costs {}", piece.label, piece.price)
    } else {
        format!("place {}   the dweller is {}", piece.label, doing)
    };

    if status == lot.last_status {
        return;
    }
    lot.last_status = status.clone();
    let readout = lot.interface.status_readout;
    ui_set_text(&mut commands, readout, &status);
}
