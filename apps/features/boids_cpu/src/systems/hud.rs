use crate::resources::{BoidsResources, HudHandles};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;

const OVERLAYS: [(&str, &str); 4] = [
    ("1", "Headings"),
    ("2", "Region colors"),
    ("3", "Neighbors"),
    ("4", "Boids"),
];

const RATE_BANDS: [(f32, Vec4); 3] = [
    (50.0, Vec4::new(0.45, 0.92, 0.50, 1.0)),
    (25.0, Vec4::new(1.0, 0.68, 0.25, 1.0)),
    (0.0, Vec4::new(1.0, 0.36, 0.36, 1.0)),
];

pub fn build(tree: &mut UiTreeBuilder) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_intro(UiAnimationType::Fade, 0.25)
        .entity();

    let mut handles = HudHandles::default();

    tree.in_parent(root, |tree| {
        let panel = tree
            .add_node()
            .window(
                Ab(vec2(16.0, 16.0)),
                Ab(vec2(PANEL_WIDTH, PANEL_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG)
            .with_shadow(vec4(0.0, 0.0, 0.0, 0.7), vec2(0.0, 12.0), 36.0, 4.0)
            .entity();

        tree.in_parent(panel, |tree| {
            add_title(tree);

            handles.rate_readout = tree
                .add_node()
                .window(
                    Ab(vec2(MARGIN, 12.0)),
                    Ab(vec2(PANEL_WIDTH - MARGIN * 2.0, 34.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 30.0)
                .text_right()
                .color_raw::<UiBase>(RATE_BANDS[0].1)
                .entity();

            let mut y = 58.0;
            tree.add_stat_header(&stat_panel(), y, "FLOCK");
            y += 22.0;

            handles.count_readout = tree.add_stat_row(&stat_panel(), y, "boids", "");
            y += ROW_HEIGHT;

            handles.step_readout = tree.add_stat_row(&stat_panel(), y, "flock step", "measuring");
            y += ROW_HEIGHT;

            handles.sync_readout = tree.add_stat_row(&stat_panel(), y, "sync", "measuring");
            y += ROW_HEIGHT;

            handles.other_readout = tree.add_stat_row(&stat_panel(), y, "elsewhere", "measuring");
            y += ROW_HEIGHT;

            handles.region_readout = tree.add_stat_row(&stat_panel(), y, "regions", "");
            y += ROW_HEIGHT;

            handles.batch_readout = tree.add_stat_row(&stat_panel(), y, "batches", "");
            y += ROW_HEIGHT + 10.0;

            tree.add_stat_header(&stat_panel(), y, "FEATURES");
            y += 22.0;

            tree.add_stat_row(
                &stat_panel(),
                y,
                "parallel",
                on_or_off(cfg!(feature = "parallel")),
            );
            y += ROW_HEIGHT;

            y += 10.0;

            tree.add_stat_header(&stat_panel(), y, "OVERLAYS");
            y += 22.0;

            for (slot, (key, name)) in OVERLAYS.iter().enumerate() {
                handles.overlay_buttons[slot] = add_button_row(tree, y, key, name, false);
                y += BUTTON_HEIGHT + 4.0;
            }
        });
    });

    handles
}

pub fn update(
    mut resources: ResMut<BoidsResources>,
    frame: Res<RetainedUiFrame>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let clicked: Vec<Entity> = ui_button_clicks(&frame).collect();

    let handles = resources.hud;

    for entity in clicked {
        if entity == handles.overlay_buttons[0] {
            resources.show_headings = !resources.show_headings;
        } else if entity == handles.overlay_buttons[1] {
            resources.show_bounds = !resources.show_bounds;
        } else if entity == handles.overlay_buttons[2] {
            resources.show_neighbors = !resources.show_neighbors;
        } else if entity == handles.overlay_buttons[3] {
            resources.show_boids = !resources.show_boids;
        }
    }

    let step = resources.reported_milliseconds;
    let batches = resources.batches.clone();
    let flags = [
        resources.show_headings,
        resources.show_bounds,
        resources.show_neighbors,
        resources.show_boids,
    ];

    if step != resources.hud_step {
        resources.hud_step = step;
        ui_set_text(
            &mut commands,
            handles.step_readout,
            &format_milliseconds(step),
        );
    }

    let rate = time.frames_per_second.round().max(0.0) as u32;
    if rate != resources.hud_rate {
        resources.hud_rate = rate;
        let (_, color) = RATE_BANDS
            .iter()
            .find(|(floor, _)| rate as f32 >= *floor)
            .copied()
            .unwrap_or(RATE_BANDS[RATE_BANDS.len() - 1]);

        ui_set_text(&mut commands, handles.rate_readout, &format!("{rate} fps"));
        ui_set_color::<UiBase>(&mut commands, handles.rate_readout, color);
    }

    let count = resources.boids.len();
    if count != resources.hud_count {
        resources.hud_count = count;
        ui_set_text(
            &mut commands,
            handles.count_readout,
            &nightshade_ui::queries::text::thousands(count as u64),
        );
    }

    let frame_milliseconds = resources.reported_frame_milliseconds;
    if frame_milliseconds != resources.hud_frame {
        resources.hud_frame = frame_milliseconds;
        let elsewhere = frame_milliseconds
            - resources.reported_milliseconds
            - resources.reported_sync_milliseconds;
        ui_set_text(
            &mut commands,
            handles.other_readout,
            &format_milliseconds(elsewhere),
        );
    }

    let imbalance = resources.reported_imbalance;
    if imbalance != resources.hud_imbalance {
        resources.hud_imbalance = imbalance;
        let used = resources.reported_regions;
        ui_set_text(
            &mut commands,
            handles.region_readout,
            &format!("{used} at {imbalance:.2}x"),
        );
    }

    let sync = resources.reported_sync_milliseconds;
    if sync != resources.hud_sync {
        resources.hud_sync = sync;
        ui_set_text(
            &mut commands,
            handles.sync_readout,
            &format_milliseconds(sync),
        );
    }

    if batches != resources.hud_batches {
        resources.hud_batches = batches.clone();
        ui_set_text(
            &mut commands,
            handles.batch_readout,
            &format_batches(&batches),
        );
    }

    for slot in 0..OVERLAYS.len() {
        if flags[slot] == resources.hud_overlays[slot] {
            continue;
        }
        resources.hud_overlays[slot] = flags[slot];
        let (key, name) = OVERLAYS[slot];
        ui_set_text(
            &mut commands,
            handles.overlay_buttons[slot],
            &button_label(key, name, flags[slot]),
        );
    }
}

fn format_milliseconds(milliseconds: f64) -> String {
    if milliseconds <= 0.0 {
        return "measuring".to_string();
    }
    format!("{milliseconds:.1} ms")
}

fn format_batches(batches: &[std::ops::Range<usize>]) -> String {
    if batches.is_empty() {
        return "none".to_string();
    }

    let systems: usize = batches.iter().map(|batch| batch.len()).sum();
    let widest = batches
        .iter()
        .map(|batch| batch.len())
        .max()
        .unwrap_or_default();

    format!("{systems} in {}, {widest} wide", batches.len())
}

fn on_or_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

fn button_label(key: &str, name: &str, enabled: bool) -> String {
    format!("{key}   {name}   {}", on_or_off(enabled))
}

fn add_title(tree: &mut UiTreeBuilder) {
    tree.add_node()
        .window(
            Ab(vec2(MARGIN, 16.0)),
            Ab(vec2(200.0, 26.0)),
            Anchor::TopLeft,
        )
        .with_text("BOIDS", 21.0)
        .text_left()
        .with_text_outline(SECTION_HEADER, 1.5)
        .color_raw::<UiBase>(TEXT_PRIMARY)
        .entity();
}

fn add_button_row(
    tree: &mut UiTreeBuilder,
    y: f32,
    key: &str,
    name: &str,
    enabled: bool,
) -> Entity {
    let holder = tree
        .add_node()
        .window(
            Ab(vec2(MARGIN, y)),
            Ab(vec2(PANEL_WIDTH - MARGIN * 2.0, BUTTON_HEIGHT)),
            Anchor::TopLeft,
        )
        .entity();

    tree.in_parent(holder, |tree| {
        tree.add_button(&button_label(key, name, enabled))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_step_reads_as_measuring_until_the_first_report() {
        assert_eq!(format_milliseconds(0.0), "measuring");
    }

    #[test]
    fn the_batch_layout_reads_as_a_count_and_a_width() {
        assert_eq!(format_batches(&[0..1, 1..9, 9..10]), "10 in 3, 8 wide");
    }

    #[test]
    fn a_schedule_with_no_batches_still_reads() {
        assert_eq!(format_batches(&[]), "none");
    }

    #[test]
    fn a_button_carries_its_key_and_its_state() {
        assert_eq!(button_label("1", "Headings", true), "1   Headings   on");
        assert_eq!(button_label("1", "Headings", false), "1   Headings   off");
    }
}
