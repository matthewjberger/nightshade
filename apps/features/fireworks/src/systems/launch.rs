use crate::commands::launch;
use crate::data::ShowAction;
use crate::resources::{Launcher, Show};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn advance_show(
    mut launcher: ResMut<Launcher>,
    mut show: ResMut<Show>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let uptime = time.uptime_milliseconds as f32 / 1000.0;

    if show.active {
        run_show(&mut show, &mut commands, uptime);
        return;
    }

    if !launcher.auto || uptime < launcher.next_launch_time {
        return;
    }

    if launcher.salvo {
        launch::launch_salvo(&mut commands, launcher.salvo_count);
    } else {
        launch::launch_firework(&mut commands);
    }
    launcher.next_launch_time = uptime + launcher.interval;
}

fn run_show(show: &mut Show, commands: &mut Commands<'_>, uptime: f32) {
    let show_time = uptime - show.start_time;

    while show.index < show.events.len() {
        let action = show.events[show.index].action.clone();
        if show_time < show.events[show.index].time {
            break;
        }
        match action {
            ShowAction::Type(explosion_type) => {
                launch::launch_specific_firework(commands, explosion_type, None, None);
            }
            ShowAction::TypeWithColor(explosion_type, color) => {
                launch::launch_specific_firework(commands, explosion_type, Some(color), None);
            }
            ShowAction::Random => launch::launch_firework(commands),
            ShowAction::Salvo(count) => launch::launch_salvo(commands, count),
            ShowAction::Symmetric(explosion_type, count) => {
                launch::launch_symmetric_fireworks(commands, explosion_type, count);
            }
        }
        show.index += 1;
    }

    if show.index < show.events.len() {
        return;
    }
    let last_event_time = show.events.last().map(|event| event.time).unwrap_or(0.0);
    if show_time > last_event_time + 5.0 {
        show.active = false;
    }
}
