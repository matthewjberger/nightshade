use nalgebra_glm;
use nalgebra_glm::Vec3;
use nightshade::prelude::Commands;
use nightshade_graphics::commands::builtins::{
    spawn_cone_at, spawn_cube_at, spawn_cylinder_at, spawn_plane_at, spawn_sphere_at,
    spawn_torus_at,
};
use nightshade_render::data::config::{AntiAliasing, Atmosphere, Fog, PbrDebugMode};

use crate::systems::shell::state::{SandboxShellContext, ShellAction};
use nightshade_shell::data::Command;
use nightshade_shell::resources::state::ShellState;

pub fn engine_commands() -> [Command<SandboxShellContext>; 36] {
    [
        Command {
            name: "spawn_cube",
            description: "Spawn a cube at the specified position",
            usage: "spawn_cube <x> <y> <z>",
            execute: spawn_cube,
        },
        Command {
            name: "spawn_sphere",
            description: "Spawn a sphere at the specified position",
            usage: "spawn_sphere <x> <y> <z>",
            execute: spawn_sphere,
        },
        Command {
            name: "spawn_cylinder",
            description: "Spawn a cylinder at the specified position",
            usage: "spawn_cylinder <x> <y> <z>",
            execute: spawn_cylinder,
        },
        Command {
            name: "spawn_cone",
            description: "Spawn a cone at the specified position",
            usage: "spawn_cone <x> <y> <z>",
            execute: spawn_cone,
        },
        Command {
            name: "spawn_torus",
            description: "Spawn a torus at the specified position",
            usage: "spawn_torus <x> <y> <z>",
            execute: spawn_torus,
        },
        Command {
            name: "spawn_plane",
            description: "Spawn a plane at the specified position",
            usage: "spawn_plane <x> <y> <z>",
            execute: spawn_plane,
        },
        Command {
            name: "grid",
            description: "Toggle grid visibility (on/off)",
            usage: "grid <on|off>",
            execute: grid,
        },
        Command {
            name: "atmosphere",
            description: "Set the atmosphere type",
            usage: "atmosphere <none|sky|cloudy|space|nebula|sunset>",
            execute: atmosphere,
        },
        Command {
            name: "fov",
            description: "Get or set camera field of view in degrees",
            usage: "fov [degrees]",
            execute: fov,
        },
        Command {
            name: "timescale",
            description: "Get or set time scale (1.0 = normal, 0.5 = slow-mo, 2.0 = fast)",
            usage: "timescale [value]",
            execute: timescale,
        },
        Command {
            name: "fps",
            description: "Show current frames per second",
            usage: "fps",
            execute: fps,
        },
        Command {
            name: "quit",
            description: "Exit the application",
            usage: "quit",
            execute: quit,
        },
        Command {
            name: "pos",
            description: "Show current camera position",
            usage: "pos",
            execute: pos,
        },
        Command {
            name: "tp",
            description: "Teleport camera to specified position",
            usage: "tp <x> <y> <z>",
            execute: teleport,
        },
        Command {
            name: "unlit",
            description: "Toggle unlit rendering mode",
            usage: "unlit [on|off]",
            execute: unlit,
        },
        Command {
            name: "bloom",
            description: "Toggle bloom effect, set intensity, or set threshold",
            usage: "bloom [on|off|<intensity>|threshold <value>]",
            execute: bloom,
        },
        Command {
            name: "rotate",
            description: "Rotate an entity by degrees on each axis",
            usage: "rotate <entity> <x> <y> <z>",
            execute: rotate,
        },
        Command {
            name: "scale",
            description: "Scale an entity uniformly or on each axis",
            usage: "scale <entity> <factor> OR scale <entity> <x> <y> <z>",
            execute: scale,
        },
        Command {
            name: "move",
            description: "Move an entity by offset or to absolute position",
            usage: "move <entity> <x> <y> <z>",
            execute: move_entity,
        },
        Command {
            name: "delete",
            description: "Delete an entity from the world",
            usage: "delete <entity>",
            execute: delete,
        },
        Command {
            name: "inspect",
            description: "Show information about an entity",
            usage: "inspect <entity>",
            execute: inspect,
        },
        Command {
            name: "cull",
            description: "Toggle GPU frustum and occlusion culling",
            usage: "cull [on|off|gpu on|off|occlusion on|off]",
            execute: cull,
        },
        Command {
            name: "bounds",
            description: "Toggle bounding volume debug overlay",
            usage: "bounds [on|off]",
            execute: bounds,
        },
        Command {
            name: "normals",
            description: "Toggle vertex normal debug lines",
            usage: "normals [on|off]",
            execute: normals,
        },
        Command {
            name: "pbr",
            description: "Set PBR debug visualization mode",
            usage: "pbr [none|basecolor|normal|metallic|roughness|occlusion|emissive|f|g|d|diffuse|specular|miprainbow]",
            execute: pbr,
        },
        Command {
            name: "ssao",
            description: "Toggle screen-space ambient occlusion",
            usage: "ssao [on|off]",
            execute: ssao,
        },
        Command {
            name: "ssr",
            description: "Toggle screen-space reflections",
            usage: "ssr [on|off]",
            execute: ssr,
        },
        Command {
            name: "ssgi",
            description: "Toggle screen-space global illumination",
            usage: "ssgi [on|off]",
            execute: ssgi,
        },
        Command {
            name: "aa",
            description: "Get or set which technique smooths the edges",
            usage: "aa [off|fxaa|taa]",
            execute: anti_aliasing,
        },
        Command {
            name: "fog",
            description: "Toggle distance fog or set start/end distance",
            usage: "fog [on|off|<start> <end>]",
            execute: fog,
        },
        Command {
            name: "exposure",
            description: "Get or set exposure multiplier",
            usage: "exposure [value]",
            execute: exposure,
        },
        Command {
            name: "gamma",
            description: "Get or set gamma correction",
            usage: "gamma [value]",
            execute: gamma,
        },
        Command {
            name: "saturation",
            description: "Get or set color saturation",
            usage: "saturation [value]",
            execute: saturation,
        },
        Command {
            name: "contrast",
            description: "Get or set color contrast",
            usage: "contrast [value]",
            execute: contrast,
        },
        Command {
            name: "fullscreen",
            description: "Toggle fullscreen mode",
            usage: "fullscreen [on|off]",
            execute: fullscreen,
        },
        Command {
            name: "cursor",
            description: "Toggle mouse cursor visibility",
            usage: "cursor [on|off]",
            execute: cursor,
        },
    ]
}

pub fn register(shell: &mut ShellState<SandboxShellContext>) {
    for command in engine_commands() {
        shell.registry.register_builtin(command);
    }
}

fn spawn_cube(
    args: &[&str],
    commands: &mut Commands,
    _context: &mut SandboxShellContext,
) -> String {
    if args.len() != 3 {
        return "Usage: spawn_cube <x> <y> <z>".to_string();
    }
    let x: f32 = match args[0].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid x coordinate".to_string(),
    };
    let y: f32 = match args[1].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid y coordinate".to_string(),
    };
    let z: f32 = match args[2].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid z coordinate".to_string(),
    };
    spawn_cube_at(commands, Vec3::new(x, y, z));
    format!("Spawned cube at ({}, {}, {})", x, y, z)
}

fn spawn_sphere(
    args: &[&str],
    commands: &mut Commands,
    _context: &mut SandboxShellContext,
) -> String {
    if args.len() != 3 {
        return "Usage: spawn_sphere <x> <y> <z>".to_string();
    }
    let x: f32 = match args[0].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid x coordinate".to_string(),
    };
    let y: f32 = match args[1].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid y coordinate".to_string(),
    };
    let z: f32 = match args[2].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid z coordinate".to_string(),
    };
    spawn_sphere_at(commands, Vec3::new(x, y, z));
    format!("Spawned sphere at ({}, {}, {})", x, y, z)
}

fn spawn_cylinder(
    args: &[&str],
    commands: &mut Commands,
    _context: &mut SandboxShellContext,
) -> String {
    if args.len() != 3 {
        return "Usage: spawn_cylinder <x> <y> <z>".to_string();
    }
    let x: f32 = match args[0].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid x coordinate".to_string(),
    };
    let y: f32 = match args[1].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid y coordinate".to_string(),
    };
    let z: f32 = match args[2].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid z coordinate".to_string(),
    };
    spawn_cylinder_at(commands, Vec3::new(x, y, z));
    format!("Spawned cylinder at ({}, {}, {})", x, y, z)
}

fn spawn_cone(
    args: &[&str],
    commands: &mut Commands,
    _context: &mut SandboxShellContext,
) -> String {
    if args.len() != 3 {
        return "Usage: spawn_cone <x> <y> <z>".to_string();
    }
    let x: f32 = match args[0].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid x coordinate".to_string(),
    };
    let y: f32 = match args[1].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid y coordinate".to_string(),
    };
    let z: f32 = match args[2].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid z coordinate".to_string(),
    };
    spawn_cone_at(commands, Vec3::new(x, y, z));
    format!("Spawned cone at ({}, {}, {})", x, y, z)
}

fn spawn_torus(
    args: &[&str],
    commands: &mut Commands,
    _context: &mut SandboxShellContext,
) -> String {
    if args.len() != 3 {
        return "Usage: spawn_torus <x> <y> <z>".to_string();
    }
    let x: f32 = match args[0].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid x coordinate".to_string(),
    };
    let y: f32 = match args[1].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid y coordinate".to_string(),
    };
    let z: f32 = match args[2].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid z coordinate".to_string(),
    };
    spawn_torus_at(commands, Vec3::new(x, y, z));
    format!("Spawned torus at ({}, {}, {})", x, y, z)
}

fn spawn_plane(
    args: &[&str],
    commands: &mut Commands,
    _context: &mut SandboxShellContext,
) -> String {
    if args.len() != 3 {
        return "Usage: spawn_plane <x> <y> <z>".to_string();
    }
    let x: f32 = match args[0].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid x coordinate".to_string(),
    };
    let y: f32 = match args[1].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid y coordinate".to_string(),
    };
    let z: f32 = match args[2].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid z coordinate".to_string(),
    };
    spawn_plane_at(commands, Vec3::new(x, y, z));
    format!("Spawned plane at ({}, {}, {})", x, y, z)
}

fn grid(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.is_empty() {
        let state = if context.readout.debug_draw.show_grid {
            "on"
        } else {
            "off"
        };
        return format!("Grid is currently {}", state);
    }

    match args[0].to_lowercase().as_str() {
        "on" | "true" | "1" => {
            context.debug_draw().show_grid = true;
            "Grid enabled".to_string()
        }
        "off" | "false" | "0" => {
            context.debug_draw().show_grid = false;
            "Grid disabled".to_string()
        }
        _ => "Usage: grid <on|off>".to_string(),
    }
}

fn atmosphere(
    args: &[&str],
    _commands: &mut Commands,
    context: &mut SandboxShellContext,
) -> String {
    if args.is_empty() {
        let current = match context.readout.settings.atmosphere {
            Atmosphere::None => "none",
            Atmosphere::Sky => "sky",
            Atmosphere::CloudySky => "cloudy",
            Atmosphere::Space => "space",
            Atmosphere::Nebula => "nebula",
            Atmosphere::Sunset => "sunset",
            _ => "other",
        };
        return format!("Current atmosphere: {}", current);
    }

    let atmosphere = match args[0].to_lowercase().as_str() {
        "none" => Atmosphere::None,
        "sky" => Atmosphere::Sky,
        "cloudy" | "cloudysky" => Atmosphere::CloudySky,
        "space" => Atmosphere::Space,
        "nebula" => Atmosphere::Nebula,
        "sunset" => Atmosphere::Sunset,
        _ => {
            return "Unknown atmosphere type. Usage: atmosphere <none|sky|cloudy|space|nebula|sunset>".to_string();
        }
    };

    context.settings().atmosphere = atmosphere;
    format!("Atmosphere set to {}", args[0].to_lowercase())
}

fn fov(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let requested = if args.is_empty() {
        None
    } else {
        match args[0].parse::<f32>() {
            Ok(value) if (1.0..=179.0).contains(&value) => Some(value),
            Ok(_) => return "FOV must be between 1 and 179 degrees".to_string(),
            Err(_) => return "Invalid FOV value".to_string(),
        }
    };
    context.queue.push(ShellAction::SetFieldOfView(requested));
    String::new()
}

fn parse_vector(args: &[&str]) -> Result<Vec3, String> {
    let x: f32 = args[0].parse().map_err(|_| "Invalid x value".to_string())?;
    let y: f32 = args[1].parse().map_err(|_| "Invalid y value".to_string())?;
    let z: f32 = args[2].parse().map_err(|_| "Invalid z value".to_string())?;
    Ok(Vec3::new(x, y, z))
}

fn timescale(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.is_empty() {
        return format!("Current timescale: {:.2}", context.readout.time_speed);
    }

    let scale: f32 = match args[0].parse() {
        Ok(value) => value,
        Err(_) => return "Invalid timescale value".to_string(),
    };

    if !(0.0..=10.0).contains(&scale) {
        return "Timescale must be between 0.0 and 10.0".to_string();
    }

    context.wanted.time_speed = Some(scale);
    format!("Timescale set to {:.2}", scale)
}

fn fps(_args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let timing = &context.readout;
    format!(
        "FPS: {:.1} | Frame time: {:.2}ms | Uptime: {:.1}s",
        timing.frames_per_second,
        timing.delta_time * 1000.0,
        timing.uptime_milliseconds as f32 / 1000.0
    )
}

fn quit(_args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    context.wanted.exit = true;
    "Exiting...".to_string()
}

fn pos(_args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    context.queue.push(ShellAction::ReadCameraPosition);
    String::new()
}

fn teleport(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.len() != 3 {
        return "Usage: tp <x> <y> <z>".to_string();
    }
    match parse_vector(args) {
        Ok(position) => {
            context.queue.push(ShellAction::TeleportCamera(position));
            String::new()
        }
        Err(message) => message,
    }
}

fn unlit(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.is_empty() {
        let state = if context.readout.settings.unlit_mode {
            "on"
        } else {
            "off"
        };
        return format!("Unlit mode is currently {}", state);
    }

    match args[0].to_lowercase().as_str() {
        "on" | "true" | "1" => {
            context.settings().unlit_mode = true;
            "Unlit mode enabled".to_string()
        }
        "off" | "false" | "0" => {
            context.settings().unlit_mode = false;
            "Unlit mode disabled".to_string()
        }
        _ => "Usage: unlit [on|off]".to_string(),
    }
}

fn bloom(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.is_empty() {
        let state = if context.readout.settings.bloom_enabled {
            format!(
                "on (intensity: {:.2}, threshold: {:.2})",
                context.readout.settings.bloom_intensity, context.readout.settings.bloom_threshold
            )
        } else {
            "off".to_string()
        };
        return format!("Bloom is currently {}", state);
    }

    match args[0].to_lowercase().as_str() {
        "on" | "true" | "1" => {
            context.settings().bloom_enabled = true;
            "Bloom enabled".to_string()
        }
        "off" | "false" | "0" => {
            context.settings().bloom_enabled = false;
            "Bloom disabled".to_string()
        }
        "threshold" => {
            if args.len() < 2 {
                return format!(
                    "Bloom threshold is {:.2}",
                    context.readout.settings.bloom_threshold
                );
            }
            if let Ok(threshold) = args[1].parse::<f32>() {
                if (0.0..=10.0).contains(&threshold) {
                    context.settings().bloom_threshold = threshold;
                    return format!("Bloom threshold set to {:.2}", threshold);
                }
                return "Bloom threshold must be between 0.0 and 10.0".to_string();
            }
            "Usage: bloom threshold <value>".to_string()
        }
        _ => {
            if let Ok(intensity) = args[0].parse::<f32>() {
                if (0.0..=5.0).contains(&intensity) {
                    context.settings().bloom_enabled = true;
                    context.settings().bloom_intensity = intensity;
                    return format!("Bloom intensity set to {:.2}", intensity);
                }
                return "Bloom intensity must be between 0.0 and 5.0".to_string();
            }
            "Usage: bloom [on|off|<intensity>|threshold <value>]".to_string()
        }
    }
}

fn rotate(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.len() != 4 {
        return "Usage: rotate <entity> <x> <y> <z>".to_string();
    }
    let Some(entity) = nightshade_shell::data::parse_entity(args[0]) else {
        return format!("Invalid entity format: {}", args[0]);
    };
    match parse_vector(&args[1..]) {
        Ok(degrees) => {
            context.queue.push(ShellAction::RotateEntity(
                entity,
                args[0].to_string(),
                degrees,
            ));
            String::new()
        }
        Err(message) => message,
    }
}

fn scale(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.len() < 2 {
        return "Usage: scale <entity> <factor> OR scale <entity> <x> <y> <z>".to_string();
    }
    let Some(entity) = nightshade_shell::data::parse_entity(args[0]) else {
        return format!("Invalid entity format: {}", args[0]);
    };
    let factor = if args.len() == 2 {
        match args[1].parse::<f32>() {
            Ok(value) => Vec3::new(value, value, value),
            Err(_) => return "Invalid scale factor".to_string(),
        }
    } else if args.len() == 4 {
        match parse_vector(&args[1..]) {
            Ok(value) => value,
            Err(message) => return message,
        }
    } else {
        return "Usage: scale <entity> <factor> OR scale <entity> <x> <y> <z>".to_string();
    };
    context.queue.push(ShellAction::ScaleEntity(
        entity,
        args[0].to_string(),
        factor,
    ));
    String::new()
}

fn move_entity(
    args: &[&str],
    _commands: &mut Commands,
    context: &mut SandboxShellContext,
) -> String {
    if args.len() != 4 {
        return "Usage: move <entity> <x> <y> <z>".to_string();
    }
    let Some(entity) = nightshade_shell::data::parse_entity(args[0]) else {
        return format!("Invalid entity format: {}", args[0]);
    };
    match parse_vector(&args[1..]) {
        Ok(position) => {
            context.queue.push(ShellAction::MoveEntity(
                entity,
                args[0].to_string(),
                position,
            ));
            String::new()
        }
        Err(message) => message,
    }
}

fn delete(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.is_empty() {
        return "Usage: delete <entity>".to_string();
    }
    let Some(entity) = nightshade_shell::data::parse_entity(args[0]) else {
        return format!("Invalid entity format: {}", args[0]);
    };
    context
        .queue
        .push(ShellAction::DeleteEntity(entity, args[0].to_string()));
    String::new()
}

fn inspect(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.is_empty() {
        return "Usage: inspect <entity>".to_string();
    }
    let Some(entity) = nightshade_shell::data::parse_entity(args[0]) else {
        return format!("Invalid entity format: {}", args[0]);
    };
    context
        .queue
        .push(ShellAction::InspectEntity(entity, args[0].to_string()));
    String::new()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "on" | "true" | "1" => Some(true),
        "off" | "false" | "0" => Some(false),
        _ => None,
    }
}

fn cull(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let graphics = context.renderer_state();
    if args.is_empty() {
        return format!(
            "GPU culling: {}",
            if graphics.gpu_culling_enabled {
                "on"
            } else {
                "off"
            }
        );
    }
    match args[0].to_lowercase().as_str() {
        "gpu" => {
            let Some(value) = args.get(1).and_then(|raw| parse_bool(raw)) else {
                return "Usage: cull gpu <on|off>".to_string();
            };
            graphics.gpu_culling_enabled = value;
            format!("GPU culling {}", if value { "enabled" } else { "disabled" })
        }
        other => {
            let Some(value) = parse_bool(other) else {
                return "Usage: cull [on|off|gpu on|off]".to_string();
            };
            graphics.gpu_culling_enabled = value;
            format!("Culling {}", if value { "enabled" } else { "disabled" })
        }
    }
}

fn bounds(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let graphics = context.debug_draw();
    if args.is_empty() {
        return format!(
            "Bounding volumes: {}",
            if graphics.show_bounding_volumes {
                "on"
            } else {
                "off"
            }
        );
    }
    let Some(value) = parse_bool(args[0]) else {
        return "Usage: bounds [on|off]".to_string();
    };
    graphics.show_bounding_volumes = value;
    format!(
        "Bounding volumes {}",
        if value { "enabled" } else { "disabled" }
    )
}

fn normals(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let graphics = context.debug_draw();
    if args.is_empty() {
        return format!(
            "Normals: {}",
            if graphics.show_normals { "on" } else { "off" }
        );
    }
    let Some(value) = parse_bool(args[0]) else {
        return "Usage: normals [on|off]".to_string();
    };
    graphics.show_normals = value;
    format!(
        "Normal debug {}",
        if value { "enabled" } else { "disabled" }
    )
}

fn pbr(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.is_empty() {
        return format!(
            "PBR debug mode: {}",
            context.readout.debug_draw.pbr_debug_mode.name()
        );
    }
    let mode = match args[0].to_lowercase().as_str() {
        "none" | "off" => PbrDebugMode::None,
        "basecolor" | "base" | "color" => PbrDebugMode::BaseColor,
        "normal" | "normals" => PbrDebugMode::Normal,
        "metallic" | "metal" => PbrDebugMode::Metallic,
        "roughness" | "rough" => PbrDebugMode::Roughness,
        "occlusion" | "ao" => PbrDebugMode::Occlusion,
        "emissive" | "emit" => PbrDebugMode::Emissive,
        "f" | "fresnel" => PbrDebugMode::F,
        "g" | "geometry" => PbrDebugMode::G,
        "d" | "distribution" => PbrDebugMode::D,
        "diffuse" => PbrDebugMode::Diffuse,
        "specular" | "spec" => PbrDebugMode::Specular,
        "miprainbow" | "mip" | "rainbow" => PbrDebugMode::MipRainbow,
        _ => {
            return "Usage: pbr [none|basecolor|normal|metallic|roughness|occlusion|emissive|f|g|d|diffuse|specular|miprainbow]".to_string();
        }
    };
    context.debug_draw().pbr_debug_mode = mode;
    format!("PBR debug mode: {}", mode.name())
}

fn ssao(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let graphics = context.settings();
    if args.is_empty() {
        return format!("SSAO: {}", if graphics.ssao_enabled { "on" } else { "off" });
    }
    let Some(value) = parse_bool(args[0]) else {
        return "Usage: ssao [on|off]".to_string();
    };
    graphics.ssao_enabled = value;
    format!("SSAO {}", if value { "enabled" } else { "disabled" })
}

fn ssr(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let graphics = context.settings();
    if args.is_empty() {
        return format!("SSR: {}", if graphics.ssr_enabled { "on" } else { "off" });
    }
    let Some(value) = parse_bool(args[0]) else {
        return "Usage: ssr [on|off]".to_string();
    };
    graphics.ssr_enabled = value;
    format!("SSR {}", if value { "enabled" } else { "disabled" })
}

fn ssgi(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let graphics = context.settings();
    if args.is_empty() {
        return format!("SSGI: {}", if graphics.ssgi_enabled { "on" } else { "off" });
    }
    let Some(value) = parse_bool(args[0]) else {
        return "Usage: ssgi [on|off]".to_string();
    };
    graphics.ssgi_enabled = value;
    format!("SSGI {}", if value { "enabled" } else { "disabled" })
}

fn anti_aliasing(
    args: &[&str],
    _commands: &mut Commands,
    context: &mut SandboxShellContext,
) -> String {
    let graphics = context.settings();
    if args.is_empty() {
        return format!("Antialiasing: {}", graphics.anti_aliasing.name());
    }
    let asked = args[0].to_lowercase();
    let Some(technique) = AntiAliasing::ALL
        .iter()
        .find(|technique| technique.name().to_lowercase() == asked)
    else {
        return "Usage: aa [off|fxaa|taa]".to_string();
    };
    graphics.anti_aliasing = *technique;
    format!("Antialiasing: {}", technique.name())
}

fn fog(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    if args.is_empty() {
        return match context.readout.settings.fog {
            Some(fog) => format!("Fog: on (start {:.1}, end {:.1})", fog.start, fog.end),
            None => "Fog: off".to_string(),
        };
    }
    if args.len() >= 2
        && let (Ok(start), Ok(end)) = (args[0].parse::<f32>(), args[1].parse::<f32>())
    {
        let mut fog = context.readout.settings.fog.unwrap_or_default();
        fog.start = start;
        fog.end = end.max(start + 0.01);
        context.settings().fog = Some(fog);
        return format!("Fog start {:.1}, end {:.1}", fog.start, fog.end);
    }
    let Some(value) = parse_bool(args[0]) else {
        return "Usage: fog [on|off|<start> <end>]".to_string();
    };
    if value {
        if context.readout.settings.fog.is_none() {
            context.settings().fog = Some(Fog::default());
        }
        "Fog enabled".to_string()
    } else {
        context.settings().fog = None;
        "Fog disabled".to_string()
    }
}

fn exposure(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let grading = &mut context.settings().color_grading;
    if args.is_empty() {
        return format!("Exposure: {:.3}", grading.exposure);
    }
    let Ok(value) = args[0].parse::<f32>() else {
        return "Usage: exposure [value]".to_string();
    };
    grading.exposure = value.max(0.0);
    format!("Exposure set to {:.3}", grading.exposure)
}

fn gamma(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let grading = &mut context.settings().color_grading;
    if args.is_empty() {
        return format!("Gamma: {:.2}", grading.gamma);
    }
    let Ok(value) = args[0].parse::<f32>() else {
        return "Usage: gamma [value]".to_string();
    };
    grading.gamma = value.clamp(0.1, 10.0);
    format!("Gamma set to {:.2}", grading.gamma)
}

fn saturation(
    args: &[&str],
    _commands: &mut Commands,
    context: &mut SandboxShellContext,
) -> String {
    let grading = &mut context.settings().color_grading;
    if args.is_empty() {
        return format!("Saturation: {:.2}", grading.saturation);
    }
    let Ok(value) = args[0].parse::<f32>() else {
        return "Usage: saturation [value]".to_string();
    };
    grading.saturation = value.clamp(0.0, 4.0);
    format!("Saturation set to {:.2}", grading.saturation)
}

fn contrast(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let grading = &mut context.settings().color_grading;
    if args.is_empty() {
        return format!("Contrast: {:.2}", grading.contrast);
    }
    let Ok(value) = args[0].parse::<f32>() else {
        return "Usage: contrast [value]".to_string();
    };
    grading.contrast = value.clamp(0.0, 4.0);
    format!("Contrast set to {:.2}", grading.contrast)
}

fn fullscreen(
    args: &[&str],
    _commands: &mut Commands,
    context: &mut SandboxShellContext,
) -> String {
    let window = context.window();
    if args.is_empty() {
        return format!(
            "Fullscreen: {}",
            if window.use_fullscreen { "on" } else { "off" }
        );
    }
    let Some(value) = parse_bool(args[0]) else {
        return "Usage: fullscreen [on|off]".to_string();
    };
    window.use_fullscreen = value;
    format!("Fullscreen {}", if value { "enabled" } else { "disabled" })
}

fn cursor(args: &[&str], _commands: &mut Commands, context: &mut SandboxShellContext) -> String {
    let window = context.window();
    if args.is_empty() {
        return format!("Cursor: {}", if window.show_cursor { "on" } else { "off" });
    }
    let Some(value) = parse_bool(args[0]) else {
        return "Usage: cursor [on|off]".to_string();
    };
    window.show_cursor = value;
    format!("Cursor {}", if value { "shown" } else { "hidden" })
}
