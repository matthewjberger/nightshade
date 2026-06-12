use crate::systems::shell::state::{SandboxShellContext, ShellAction};
use nightshade::prelude::*;
use nightshade::shell::Command;

pub const PLAYER_POS: Command<SandboxShellContext> = Command {
    name: "player_pos",
    description: "Print player position",
    usage: "player_pos",
    execute: player_pos,
};

pub const PLAYER_TP: Command<SandboxShellContext> = Command {
    name: "player_tp",
    description: "Teleport player",
    usage: "player_tp <x> <y> <z>",
    execute: player_tp,
};

pub const RESET: Command<SandboxShellContext> = Command {
    name: "reset",
    description: "Reset player to spawn",
    usage: "reset",
    execute: reset,
};

pub const FLY: Command<SandboxShellContext> = Command {
    name: "fly",
    description: "Toggle fly mode",
    usage: "fly [on|off]",
    execute: fly,
};

pub const THIRD_PERSON: Command<SandboxShellContext> = Command {
    name: "third_person",
    description: "Toggle third person mode",
    usage: "third_person [on|off]",
    execute: third_person,
};

pub const STRIDE_WHEEL: Command<SandboxShellContext> = Command {
    name: "stride_wheel",
    description: "Toggle the stride wheel visualization",
    usage: "stride_wheel [on|off]",
    execute: stride_wheel,
};

pub const SPHERES: Command<SandboxShellContext> = Command {
    name: "spheres",
    description: "Toggle the bumper and lifter visualization",
    usage: "spheres [on|off]",
    execute: spheres,
};

pub const POSE: Command<SandboxShellContext> = Command {
    name: "pose",
    description: "List poses or set the active character pose",
    usage: "pose [name]",
    execute: pose,
};

fn player_pos(_args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    context.queue.push(ShellAction::PrintPlayerPosition);
    "see log".to_string()
}

fn player_tp(args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    if args.len() != 3 {
        return PLAYER_TP.usage.to_string();
    }
    let parsed: Result<Vec<f32>, _> = args.iter().map(|arg| arg.parse::<f32>()).collect();
    let Ok(values) = parsed else {
        return "invalid coordinates".to_string();
    };
    context.queue.push(ShellAction::TeleportPlayer(Vec3::new(
        values[0], values[1], values[2],
    )));
    format!("teleport ({}, {}, {})", values[0], values[1], values[2])
}

fn reset(_args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    context.queue.push(ShellAction::ResetPlayer);
    "reset".to_string()
}

fn fly(args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    let Some(value) = parse_toggle(args) else {
        return FLY.usage.to_string();
    };
    context.queue.push(ShellAction::SetFly(value));
    "OK".to_string()
}

fn third_person(args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    let Some(value) = parse_toggle(args) else {
        return THIRD_PERSON.usage.to_string();
    };
    context.queue.push(ShellAction::SetThirdPerson(value));
    "OK".to_string()
}

fn stride_wheel(args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    let Some(value) = parse_toggle(args) else {
        return STRIDE_WHEEL.usage.to_string();
    };
    context.queue.push(ShellAction::SetStrideWheel(value));
    "OK".to_string()
}

fn spheres(args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    let Some(value) = parse_toggle(args) else {
        return SPHERES.usage.to_string();
    };
    context.queue.push(ShellAction::SetSpheres(value));
    "OK".to_string()
}

fn pose(args: &[&str], _world: &mut World, context: &mut SandboxShellContext) -> String {
    match args {
        [] => format!("poses: clip, {}", context.pose_names.join(", ")),
        [name] => {
            if *name != "clip" && !context.pose_names.iter().any(|pose| pose == name) {
                return format!(
                    "unknown pose, available: clip, {}",
                    context.pose_names.join(", ")
                );
            }
            context.queue.push(ShellAction::SetPose(name.to_string()));
            format!("pose {name}")
        }
        _ => POSE.usage.to_string(),
    }
}

fn parse_toggle(args: &[&str]) -> Option<Option<bool>> {
    match args {
        [] => Some(None),
        ["on"] => Some(Some(true)),
        ["off"] => Some(Some(false)),
        _ => None,
    }
}
