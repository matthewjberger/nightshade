use crate::systems::shell::state::{SandboxShellContext, ShellAction};
use nightshade::prelude::*;
use nightshade::shell::Command;

pub const TIME: Command<SandboxShellContext> = Command {
    name: "time",
    description: "Get or set time of day",
    usage: "time [hour]",
    execute: time,
};

pub const TIME_PAUSE: Command<SandboxShellContext> = Command {
    name: "time_pause",
    description: "Pause or resume the day night cycle",
    usage: "time_pause [on|off]",
    execute: time_pause,
};

pub const TIME_SPEED: Command<SandboxShellContext> = Command {
    name: "time_speed",
    description: "Set day night cycle speed",
    usage: "time_speed <value>",
    execute: time_speed,
};

fn time(args: &[&str], world: &mut World, context: &mut SandboxShellContext) -> String {
    match args {
        [] => format!(
            "time of day: {:.2}",
            world.expect_resource_mut::<RendererState>().day_night.hour
        ),
        [hour] => match hour.parse::<f32>() {
            Ok(value) => {
                context.queue.push(ShellAction::SetTimeOfDay(value));
                format!("time of day set to {value:.2}")
            }
            Err(_) => "invalid hour".to_string(),
        },
        _ => TIME.usage.to_string(),
    }
}

fn time_pause(args: &[&str], world: &mut World, context: &mut SandboxShellContext) -> String {
    let paused = match args {
        [] => {
            world
                .expect_resource_mut::<RendererState>()
                .day_night
                .auto_cycle
        }
        ["on"] => true,
        ["off"] => false,
        _ => return TIME_PAUSE.usage.to_string(),
    };
    context.queue.push(ShellAction::PauseTimeOfDay(paused));
    if paused {
        "time paused".to_string()
    } else {
        "time resumed".to_string()
    }
}

fn time_speed(args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    let [value] = args else {
        return TIME_SPEED.usage.to_string();
    };
    match value.parse::<f32>() {
        Ok(speed) => {
            context.queue.push(ShellAction::SetTimeSpeed(speed));
            format!("time speed set to {speed}")
        }
        Err(_) => "invalid speed".to_string(),
    }
}
