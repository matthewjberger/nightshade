use crate::commands::{
    clear_the_gallery, flip_the_mode, halt_the_crew, step_the_hands, throw_the_work_away,
};
use crate::components::Statue;
use crate::data::{SculptJob, Stage};
use crate::resources::{Gallery, Panel};
use crate::theme;
use nalgebra_glm::Vec4;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_render::prelude::MeshCache;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;
use nightshade_worker::{Workers, workers_hands, workers_hire, workers_waiting};
use std::collections::HashSet;

pub fn build(tree: &mut UiTreeBuilder, gallery: &Gallery) -> Panel {
    let mut panel = Panel::default();
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    tree.in_parent(root, |tree| {
        let holder = tree
            .add_node()
            .window(
                Ab(vec2(theme::PANEL_MARGIN, theme::PANEL_MARGIN)),
                Ab(vec2(theme::PANEL_WIDTH, theme::PANEL_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(8.0, 1.0, theme::PANEL_BORDER)
            .color_raw::<UiBase>(theme::PANEL_BACKGROUND)
            .flow(
                FlowDirection::Vertical,
                theme::PANEL_PADDING,
                theme::PANEL_SPACING,
            )
            .entity();

        tree.in_parent(holder, |tree| {
            heading(tree, theme::TITLE);
            panel.rate = readout(tree);
            panel.frame_lamps = strip(
                tree,
                theme::FRAME_LAMPS,
                theme::FRAME_LAMP_WIDTH,
                theme::LAMP_GAP,
                theme::LAMP_DARK,
            );
            panel.hands_line = readout(tree);
            panel.slot_lamps = strip(
                tree,
                theme::STANDS,
                theme::SLOT_LAMP_WIDTH,
                theme::SLOT_LAMP_GAP,
                theme::LAMP_DARK,
            );
            panel.stands_line = readout(tree);
            panel.thrown_line = readout(tree);
            panel.mode = tree.add_button(&mode_label(gallery.on_the_frame));
            panel.crew = tree.add_button(&hands_label(gallery.hands));
            panel.halt = tree.add_button(&halt_label(gallery.paused));
            panel.again = tree.add_button("Carve them again");
        });

        notes(tree);
    });

    panel.root = root;
    panel.last_rate = u32::MAX;
    panel.last_busy = usize::MAX;
    panel.last_standing = usize::MAX;
    panel.last_thrown = u32::MAX;
    panel.last_paused = !gallery.paused;
    panel.last_stage = vec![u8::MAX; theme::STANDS];
    panel
}

pub fn drive_the_panel(
    frame: Res<RetainedUiFrame>,
    mut gallery: ResMut<Gallery>,
    mut crew: ResMut<Workers<SculptJob>>,
    mut panel: ResMut<Panel>,
    mut mesh_cache: ResMut<MeshCache>,
    mut commands: Commands,
    mut statues: Query<(&Statue,)>,
) {
    let root = panel.root;
    let mode = panel.mode;
    let crew_button = panel.crew;
    let halt = panel.halt;
    let again = panel.again;
    if root == Entity::default() {
        return;
    }
    for entity in ui_button_clicks(&frame) {
        if entity == mode {
            flip_the_mode(&mut gallery);
            ui_set_text(&mut commands, mode, &mode_label(gallery.on_the_frame));
            start_again(
                &mut gallery,
                &mut crew,
                &mut mesh_cache,
                &mut commands,
                &mut statues,
            );
        } else if entity == halt {
            halt_the_crew(&mut gallery);
            if gallery.paused {
                throw_the_work_away(&mut gallery, &mut crew);
            }
        } else if entity == again {
            start_again(
                &mut gallery,
                &mut crew,
                &mut mesh_cache,
                &mut commands,
                &mut statues,
            );
        } else if entity == crew_button {
            step_the_hands(&mut gallery);
            start_again(
                &mut gallery,
                &mut crew,
                &mut mesh_cache,
                &mut commands,
                &mut statues,
            );
            *crew = workers_hire::<SculptJob>(gallery.hands);
            panel.last_hands = usize::MAX;
        }
    }
}

pub fn refresh_the_panel(
    time: Res<Time>,
    gallery: Res<Gallery>,
    crew: Res<Workers<SculptJob>>,
    mut panel: ResMut<Panel>,
    mut commands: Commands,
) {
    if panel.root == Entity::default() {
        return;
    }
    paint_the_frame(&mut panel, &mut commands, time.delta_time);

    let queued: HashSet<usize> = workers_waiting(&crew).into_iter().collect();
    let hands = workers_hands(&crew);
    let mut busy = 0;
    let mut standing = 0;
    for slot in 0..theme::STANDS {
        let mark = match gallery.stage[slot] {
            Stage::Standing => {
                standing += 1;
                3
            }
            Stage::Asked if queued.contains(&slot) => 1,
            Stage::Asked => {
                busy += 1;
                2
            }
            Stage::Empty => 0,
        };
        if panel.last_stage[slot] == mark {
            continue;
        }
        panel.last_stage[slot] = mark;
        let color = match mark {
            1 => theme::LAMP_QUEUED,
            2 => theme::LAMP_HAND,
            3 => theme::LAMP_STANDING,
            _ => theme::LAMP_DARK,
        };
        let lamp = panel.slot_lamps[slot];
        ui_set_color::<UiBase>(&mut commands, lamp, color);
    }

    if panel.last_hands != hands || panel.last_busy != busy {
        panel.last_hands = hands;
        panel.last_busy = busy;
        ui_set_text(
            &mut commands,
            panel.hands_line,
            &format!("{hands} hands hired, {busy} carving now"),
        );
        ui_set_text(&mut commands, panel.crew, &hands_label(hands));
    }
    if panel.last_paused != gallery.paused {
        panel.last_paused = gallery.paused;
        ui_set_text(&mut commands, panel.halt, &halt_label(gallery.paused));
    }
    if panel.last_thrown != gallery.thrown {
        panel.last_thrown = gallery.thrown;
        ui_set_text(
            &mut commands,
            panel.thrown_line,
            &format!("{} stones thrown away", gallery.thrown),
        );
    }
    if panel.last_standing != standing {
        panel.last_standing = standing;
        ui_set_text(
            &mut commands,
            panel.stands_line,
            &format!("{standing} of {} stones done", theme::STANDS),
        );
    }
    let rate = if panel.smooth > 0.0 {
        (1.0 / panel.smooth).round().max(0.0) as u32
    } else {
        0
    };
    if panel.last_rate != rate {
        panel.last_rate = rate;
        ui_set_text(
            &mut commands,
            panel.rate,
            &format!("{rate} frames a second"),
        );
        let color = if rate >= theme::GOOD_RATE {
            theme::GOOD_COLOR
        } else {
            theme::POOR_COLOR
        };
        ui_set_color::<UiBase>(&mut commands, panel.rate, color);
    }
}

fn paint_the_frame(panel: &mut Panel, commands: &mut Commands, delta: f32) {
    panel.smooth = if panel.smooth > 0.0 {
        panel.smooth * 0.9 + delta * 0.1
    } else {
        delta
    };
    let count = panel.frame_lamps.len();
    if count == 0 {
        return;
    }
    let spent = delta * 1000.0;
    let color = if spent >= theme::FRAME_POOR {
        theme::LAMP_POOR
    } else if spent >= theme::FRAME_FAIR {
        theme::LAMP_FAIR
    } else {
        theme::LAMP_GOOD
    };
    let width = ((spent / theme::FRAME_STEP).ceil() as usize).clamp(1, count / 3);
    for _ in 0..width {
        let lamp = panel.frame_lamps[panel.stripe];
        ui_set_color::<UiBase>(commands, lamp, color);
        panel.stripe = (panel.stripe + 1) % count;
    }
    let head = panel.frame_lamps[panel.stripe];
    ui_set_color::<UiBase>(commands, head, theme::LAMP_HEAD);
}

fn start_again(
    gallery: &mut Gallery,
    crew: &mut Workers<SculptJob>,
    mesh_cache: &mut MeshCache,
    commands: &mut Commands,
    statues: &mut Query<(&Statue,)>,
) {
    gallery.paused = false;
    throw_the_work_away(gallery, crew);
    let mut standing = Vec::new();
    statues.for_each_mut(|entity, (statue,)| standing.push((entity, statue.slot)));
    clear_the_gallery(gallery, mesh_cache, commands, standing);
}

fn notes(tree: &mut UiTreeBuilder) {
    let holder = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-theme::PANEL_MARGIN, theme::PANEL_MARGIN)),
            Ab(vec2(theme::NOTES_WIDTH, theme::NOTES_HEIGHT)),
            Anchor::TopRight,
        )
        .with_rect(8.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_BACKGROUND)
        .flow(
            FlowDirection::Vertical,
            theme::PANEL_PADDING,
            theme::NOTE_SPACING,
        )
        .entity();

    tree.in_parent(holder, |tree| {
        heading(tree, "What to look at");
        line(tree, "Each stand wants a carved stone. Cutting");
        line(tree, "one takes about a third of a second, far");
        line(tree, "too long to do while the picture draws.");
        gap(tree);
        line(tree, "A spinning gold block sits over the stand");
        line(tree, "a thread is cutting right now. Count them:");
        line(tree, "there are never more than there are hands.");
        gap(tree);
        label(tree, "Hands on the crew");
        line(tree, "One thread, or every one the machine has.");
        line(tree, "With one, the stones arrive one at a time.");
        line(tree, "Take them all and nothing is left for the");
        line(tree, "frame, which the strip shows at once.");
        gap(tree);
        label(tree, "Stop the crew");
        line(tree, "Throws away every stone still in the queue,");
        line(tree, "so the amber lamps go dark. The green ones");
        line(tree, "finish anyway: a hand already holds them and");
        line(tree, "nothing can take that back.");
        gap(tree);
        label(tree, "Work runs");
        line(tree, "Moves the cutting onto the update system,");
        line(tree, "one stone per frame. The picture crawls and");
        line(tree, "the frame strip turns red.");
    });
}

fn strip(tree: &mut UiTreeBuilder, count: usize, width: f32, gap: f32, dark: Vec4) -> Vec<Entity> {
    let mut lamps = Vec::with_capacity(count);
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, theme::LAMP_HEIGHT)))
        .flow(FlowDirection::Horizontal, 0.0, gap)
        .entity();
    tree.in_parent(row, |tree| {
        for _ in 0..count {
            let lamp = tree
                .add_node()
                .flow_child(Ab(vec2(width, theme::LAMP_HEIGHT)))
                .with_rect(2.0, 0.0, theme::CLEAR)
                .color_raw::<UiBase>(dark)
                .entity();
            lamps.push(lamp);
        }
    });
    lamps
}

fn heading(tree: &mut UiTreeBuilder, text: &str) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0)))
        .with_text(text, theme::HEADING_SIZE)
        .text_left()
        .color_raw::<UiBase>(theme::HEADING_COLOR)
        .entity()
}

fn readout(tree: &mut UiTreeBuilder) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, theme::READOUT_HEIGHT)))
        .with_text("", theme::READOUT_SIZE)
        .text_left()
        .color_raw::<UiBase>(theme::READOUT_COLOR)
        .entity()
}

fn line(tree: &mut UiTreeBuilder, text: &str) {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, theme::NOTE_HEIGHT)))
        .with_text(text, theme::NOTE_SIZE)
        .text_left()
        .color_raw::<UiBase>(theme::NOTE_COLOR);
}

fn label(tree: &mut UiTreeBuilder, text: &str) {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, theme::NOTE_HEIGHT)))
        .with_text(text, theme::NOTE_SIZE)
        .text_left()
        .color_raw::<UiBase>(theme::LABEL_COLOR);
}

fn gap(tree: &mut UiTreeBuilder) {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 6.0)));
}

fn mode_label(on_the_frame: bool) -> String {
    let where_it_runs = if on_the_frame {
        "on the frame"
    } else {
        "on the crew"
    };
    format!("Work runs   {where_it_runs}")
}

fn hands_label(hands: usize) -> String {
    format!("Hands on the crew   {hands}")
}

fn halt_label(paused: bool) -> String {
    if paused {
        "Let the crew work".to_string()
    } else {
        "Stop the crew".to_string()
    }
}
