use crate::components::{Aim, Hook, Speed};
use crate::contract::TetherEvent;
use crate::queries::{meter_fraction, reticle_color, speed_color};
use crate::resources::{TetherResources, UiHandles};
use crate::theme::{
    CLEAR, LABEL_SIZE, LABEL_TOP, MARGIN, MENU_BACKGROUND, MENU_BORDER_WIDTH, MENU_CORNER,
    MENU_HEADING_SIZE, MENU_PADDING, MENU_ROW_SIZE, MENU_SPACING, METER_HEIGHT, METER_TOP,
    METER_TRACK, METER_WIDTH, PADDING, PANEL_BACKGROUND, PANEL_BORDER, PANEL_HEIGHT, PANEL_WIDTH,
    RETICLE_EDGE, RETICLE_EDGE_WIDTH, RETICLE_IDLE, RETICLE_SIZE, SPEED_ROW_HEIGHT, SPEED_SIZE,
    TEXT_BRIGHT, TEXT_DIM,
};
use nightshade::prelude::*;
use nightshade_ui::components::layout_types::WindowLayout;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::MenuStyle;

pub fn build(tree: &mut UiTreeBuilder) -> UiHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = UiHandles {
        root: Some(root),
        ..Default::default()
    };

    tree.in_parent(root, |tree| {
        handles.reticle = Some(
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 50.0)),
                    Ab(vec2(RETICLE_SIZE, RETICLE_SIZE)),
                    Anchor::Center,
                )
                .with_rect(RETICLE_SIZE * 0.5, RETICLE_EDGE_WIDTH, RETICLE_EDGE)
                .color_raw::<UiBase>(RETICLE_IDLE)
                .without_pointer_events()
                .entity(),
        );

        let panel = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(MARGIN, -MARGIN)),
                Ab(vec2(PANEL_WIDTH, PANEL_HEIGHT)),
                Anchor::BottomLeft,
            )
            .with_rect(MENU_CORNER, MENU_BORDER_WIDTH, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .without_pointer_events()
            .entity();

        tree.in_parent(panel, |tree| {
            handles.speed_readout = Some(
                tree.add_node()
                    .window(
                        Ab(vec2(PADDING, 6.0)),
                        Ab(vec2(PANEL_WIDTH - PADDING * 2.0, SPEED_ROW_HEIGHT)),
                        Anchor::TopLeft,
                    )
                    .with_text("0 m/s", SPEED_SIZE)
                    .text_left()
                    .color_raw::<UiBase>(TEXT_BRIGHT)
                    .entity(),
            );

            let track = tree
                .add_node()
                .window(
                    Ab(vec2(PADDING, METER_TOP)),
                    Ab(vec2(METER_WIDTH, METER_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_rect(METER_HEIGHT * 0.5, 0.0, CLEAR)
                .color_raw::<UiBase>(METER_TRACK)
                .entity();

            tree.in_parent(track, |tree| {
                handles.meter_fill = Some(
                    tree.add_node()
                        .window(
                            Ab(vec2(0.0, 0.0)),
                            Ab(vec2(0.0, METER_HEIGHT)),
                            Anchor::TopLeft,
                        )
                        .with_rect(METER_HEIGHT * 0.5, 0.0, CLEAR)
                        .color_raw::<UiBase>(TEXT_BRIGHT)
                        .entity(),
                );
            });

            handles.best_readout = Some(
                tree.add_node()
                    .window(
                        Ab(vec2(PADDING, LABEL_TOP)),
                        Ab(vec2(PANEL_WIDTH - PADDING * 2.0, LABEL_SIZE + 6.0)),
                        Anchor::TopLeft,
                    )
                    .with_text("kept 0    best 0", LABEL_SIZE)
                    .text_left()
                    .color_raw::<UiBase>(TEXT_DIM)
                    .entity(),
            );
        });
    });

    handles
}

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: MENU_BACKGROUND,
        border: PANEL_BORDER,
        corner_radius: MENU_CORNER,
        border_width: MENU_BORDER_WIDTH,
        padding: MENU_PADDING,
        spacing: MENU_SPACING,
        visible: false,
        heading_size: MENU_HEADING_SIZE,
        heading_color: TEXT_BRIGHT,
        centered: true,
        row_size: MENU_ROW_SIZE,
        row_color: TEXT_DIM,
        ..Default::default()
    }
}

pub fn show_the_speed(
    mut resources: ResMut<TetherResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut runners: Query<(&Speed,)>,
) {
    let Some(runner) = resources.scene.runner else {
        return;
    };
    let mut moving = 0.0;
    runners.for_entity(runner, |(speed,)| moving = speed.0);

    let resources = &mut *resources;
    let handles = resources.ui;
    let speed = moving.round().max(0.0) as i32;
    let kept = resources.run.last_release.round().max(0.0) as i32;
    let best = resources.run.best_speed.round().max(0.0) as i32;

    if speed != resources.view.shown_speed {
        resources.view.shown_speed = speed;
        let color = speed_color(moving);
        if let Some(readout) = handles.speed_readout {
            ui_set_text(&mut commands, readout, &format!("{speed} m/s"));
            ui_set_color::<UiBase>(&mut commands, readout, color);
        }
        if let Some(fill) = handles.meter_fill {
            ui_set_color::<UiBase>(&mut commands, fill, color);
            ui_set_window_layout(
                &mut dirty,
                &mut commands,
                fill,
                WindowLayout {
                    position: Ab(vec2(0.0, 0.0)).into(),
                    size: Ab(vec2(METER_WIDTH * meter_fraction(moving), METER_HEIGHT)).into(),
                    anchor: Anchor::TopLeft,
                },
            );
        }
    }

    if kept != resources.view.shown_kept || best != resources.view.shown_best {
        resources.view.shown_kept = kept;
        resources.view.shown_best = best;
        if let Some(readout) = handles.best_readout {
            ui_set_text(
                &mut commands,
                readout,
                &format!("kept {kept}    best {best}"),
            );
        }
    }
}

pub fn read_the_releases(events: EventReader<TetherEvent>, mut resources: ResMut<TetherResources>) {
    for event in &events {
        match event {
            TetherEvent::Released { speed } => {
                crate::commands::record_release(&mut resources.run, *speed)
            }
            TetherEvent::Fell { .. } => crate::commands::record_release(&mut resources.run, 0.0),
        }
    }
}

pub fn show_the_reticle(
    resources: Res<TetherResources>,
    mut commands: Commands,
    mut runners: Query<(&Speed, Option<&Hook>, Option<&Aim>)>,
) {
    let Some(reticle) = resources.ui.reticle else {
        return;
    };
    let Some(runner) = resources.scene.runner else {
        return;
    };

    let mut attached = false;
    let mut aiming = false;
    runners.for_entity(runner, |(_, anchor, aim)| {
        attached = anchor.is_some();
        aiming = aim.is_some();
    });

    ui_set_color::<UiBase>(&mut commands, reticle, reticle_color(attached, aiming));
}
