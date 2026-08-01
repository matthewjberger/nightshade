use crate::resources::BoidsGpuResources;
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::*;

const REPORT_INTERVAL: f32 = 0.5;

const RATE_BANDS: [(f32, Vec4); 3] = [
    (50.0, Vec4::new(0.45, 0.92, 0.50, 1.0)),
    (25.0, Vec4::new(1.0, 0.68, 0.25, 1.0)),
    (0.0, Vec4::new(1.0, 0.36, 0.36, 1.0)),
];

pub fn build(tree: &mut UiTreeBuilder, resources: &mut BoidsGpuResources) {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_intro(UiAnimationType::Fade, 0.25)
        .entity();

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
            tree.add_node()
                .window(
                    Ab(vec2(MARGIN, 14.0)),
                    Ab(vec2(PANEL_WIDTH - MARGIN * 2.0, 30.0)),
                    Anchor::TopLeft,
                )
                .with_text("BOIDS GPU", 22.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_PRIMARY);

            resources.frame_label = Some(
                tree.add_node()
                    .window(
                        Ab(vec2(MARGIN, 12.0)),
                        Ab(vec2(PANEL_WIDTH - MARGIN * 2.0, 34.0)),
                        Anchor::TopLeft,
                    )
                    .with_text("", 30.0)
                    .text_right()
                    .color_raw::<UiBase>(RATE_BANDS[0].1)
                    .entity(),
            );

            let mut y = 58.0;
            tree.add_stat_header(&stat_panel(), y, "FLOCK");
            y += 22.0;

            resources.count_label = Some(tree.add_stat_row(&stat_panel(), y, "boids", ""));
            y += ROW_HEIGHT;

            tree.add_stat_row(&stat_panel(), y, "stepped on", "the gpu");
            y += ROW_HEIGHT;

            tree.add_stat_row(&stat_panel(), y, "read back", "never");
            y += ROW_HEIGHT + 10.0;

            tree.add_stat_header(&stat_panel(), y, "KEYS");
            y += 22.0;

            tree.add_stat_row(&stat_panel(), y, "left, right", "count");
            y += ROW_HEIGHT;

            resources.visible_label = Some(tree.add_stat_row(&stat_panel(), y, "4  boids", "on"));
        });
    });
}

pub fn update(time: Res<Time>, mut resources: ResMut<BoidsGpuResources>, mut commands: Commands) {
    let delta_time = time.delta_time;

    let (count_label, frame_label, visible_label, count, visible, due) = {
        let resources = &mut *resources;
        resources.frame_seconds += f64::from(delta_time);
        resources.frame_count += 1;
        resources.report_countdown -= delta_time;

        let due = resources.report_countdown <= 0.0 && resources.frame_count > 0;
        if due {
            resources.reported_frame_milliseconds =
                resources.frame_seconds * 1000.0 / f64::from(resources.frame_count);
            resources.frame_seconds = 0.0;
            resources.frame_count = 0;
            resources.report_countdown = REPORT_INTERVAL;
        }

        (
            resources.count_label,
            resources.frame_label,
            resources.visible_label,
            resources.count,
            resources.visible,
            due,
        )
    };

    if let Some(label) = count_label {
        ui_set_text(
            &mut commands,
            label,
            &nightshade_ui::queries::text::thousands(count as u64),
        );
    }
    if let Some(label) = visible_label {
        ui_set_text(&mut commands, label, if visible { "on" } else { "off" });
    }

    if !due {
        return;
    }

    let milliseconds = resources.reported_frame_milliseconds;
    tracing::info!(
        target: "boids_gpu",
        "{count} boids, frame {milliseconds:.2} ms"
    );

    if let Some(label) = frame_label {
        let rate = if milliseconds > 0.0 {
            1000.0 / milliseconds
        } else {
            0.0
        };
        ui_set_text(&mut commands, label, &format!("{rate:.0} fps"));
        let color = RATE_BANDS
            .iter()
            .find(|(floor, _)| rate as f32 >= *floor)
            .map(|(_, color)| *color)
            .unwrap_or(RATE_BANDS[2].1);
        ui_set_color::<UiBase>(&mut commands, label, color);
    }
}
