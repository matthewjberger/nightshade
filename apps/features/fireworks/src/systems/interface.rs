use crate::commands::launch;
use crate::data::ExplosionType;
use crate::queries;
use crate::resources::{Interface, Launcher, Show, Toggles};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::*;

pub fn poll_ui_events(
    panel: Res<Interface>,
    mut launcher: ResMut<Launcher>,
    mut toggles: ResMut<Toggles>,
    mut show: ResMut<Show>,
    mut render_settings: ResMut<RenderSettings>,
    mut debug_draw: ResMut<DebugDraw>,
    time: Res<Time>,
    mut build: UiBuild,
    storage: &mut nightshade::ecs::Storage,
) {
    let uptime = time.uptime_milliseconds as f32 / 1000.0;

    let mut launch_single = false;
    let mut launch_salvo = false;
    let mut launch_finale = false;
    let mut show_clicked = false;
    let mut explosion_request: Option<ExplosionType> = None;

    for event in ui_events(build.frame()) {
        let UiEvent::ButtonClicked(entity) = event else {
            continue;
        };
        if *entity == panel.launch_single {
            launch_single = true;
        } else if *entity == panel.launch_salvo {
            launch_salvo = true;
        } else if *entity == panel.finale {
            launch_finale = true;
        } else if *entity == panel.show_play {
            show_clicked = true;
        } else if let Some(explosion_type) = panel.type_buttons.get(entity).copied() {
            explosion_request = Some(explosion_type);
        }
    }

    if let Some(value) = ui_checkbox_value(storage, panel.auto_launch) {
        launcher.auto = value;
    }
    if let Some(value) = ui_slider_value(storage, panel.launch_interval) {
        launcher.interval = value;
    }
    if let Some(value) = ui_checkbox_value(storage, panel.salvo_mode) {
        launcher.salvo = value;
    }
    if let Some(value) = ui_slider_value(storage, panel.salvo_count) {
        launcher.salvo_count = value.round().clamp(3.0, 15.0) as u32;
    }
    if let Some(value) = ui_slider_value(storage, panel.bloom_intensity) {
        render_settings.bloom_intensity = value;
    }
    if let Some(value) = ui_checkbox_value(storage, panel.show_grid) {
        toggles.grid = value;
    }
    debug_draw.show_grid = toggles.grid;

    let salvo_count = launcher.salvo_count;
    let commands = build.context().commands;
    if launch_single {
        launch::launch_firework(commands);
    }
    if launch_salvo {
        launch::launch_salvo(commands, salvo_count);
    }
    if launch_finale {
        launch::launch_finale(commands);
    }
    if let Some(explosion_type) = explosion_request {
        launch::launch_specific_firework(commands, explosion_type, None, None);
    }
    if show_clicked {
        if show.active {
            show.active = false;
        } else {
            launch::start_show(&mut show, &mut launcher, uptime);
        }
    }
}

pub fn update_ui_state(
    mut panel: ResMut<Interface>,
    launcher: Res<Launcher>,
    show: Res<Show>,
    mut build: UiBuild,
    storage: &mut nightshade::ecs::Storage,
) {
    let stats = queries::count_stats(storage);

    let salvo_change = (panel.last_salvo_mode != launcher.salvo).then(|| {
        panel.last_salvo_mode = launcher.salvo;
        launcher.salvo
    });
    let show_change = (panel.last_show_active != show.active).then(|| {
        panel.last_show_active = show.active;
        show.active
    });
    let stats_change = (panel.last_stats != Some(stats)).then(|| {
        panel.last_stats = Some(stats);
        stats
    });

    let salvo_count_label = panel.salvo_count_label;
    let salvo_count = panel.salvo_count;

    let context = build.context();
    let dirty = context.dirty;
    let commands = context.commands;

    if let Some(salvo_mode) = salvo_change {
        ui_set_visible(dirty, commands, salvo_count_label, salvo_mode);
        ui_set_visible(dirty, commands, salvo_count, salvo_mode);
    }

    if let Some(show_active) = show_change {
        if show_active {
            ui_set_text(commands, panel.show_status_label, "Show in progress...");
            ui_set_text(commands, panel.show_play, "Stop Show");
        } else {
            ui_set_text(commands, panel.show_status_label, "");
            ui_set_text(commands, panel.show_play, "Play Fireworks Show");
        }
    }

    let Some(stats) = stats_change else {
        return;
    };
    ui_set_text(
        commands,
        panel.shells_label,
        &format!("Active shells: {}", stats.shells),
    );
    ui_set_text(
        commands,
        panel.sub_shells_label,
        &format!("Sub-shells: {}", stats.sub_shells),
    );
    ui_set_text(
        commands,
        panel.fire_label,
        &format!("Fire effects: {}", stats.fire),
    );
    ui_set_text(
        commands,
        panel.sprite_label,
        &format!(
            "Sprite effects: {} ({} entities)",
            stats.sprite_total, stats.sprite_active
        ),
    );
    ui_set_text(
        commands,
        panel.smoke_label,
        &format!("Smoke effects: {}", stats.smoke),
    );
    ui_set_text(
        commands,
        panel.emitters_label,
        &format!("Active emitters: {}", stats.emitters),
    );
}
