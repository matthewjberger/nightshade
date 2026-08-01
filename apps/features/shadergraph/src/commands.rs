use crate::components::{NodeKind, NodeSlot, NodeValue};
use crate::data::{
    Kind, Preset, inlet_names_of, inlets_of, note_of, outlet_names_of, outlets_of, takes_a_value,
    tint_of, title_of, value_span_of,
};
use crate::resources::{KEPT, Pictures, Plucked, SLOTS};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::Commands;
use nightshade::ecs::{Entity, Storage};
use nightshade_ui_nodegraph::prelude::{
    NodeField, NodeInletNames, NodeInlets, NodeNote, NodeOutletNames, NodeOutlets, NodePlace,
    NodeTint, NodeTitle, Socket, WireFrom, WireTo,
};

const FACE: &[u8] = include_bytes!("../../../../assets/textures/awesomeface.png");
const STONE: &[u8] =
    include_bytes!("../../../../assets/kits/models/retro/Textures/cobblestone.png");

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

fn decode(name: &str, bytes: &[u8]) -> Option<Plucked> {
    let decoded = nightshade_render::data::decoded_image::decode_to_rgba8(bytes).ok()?;
    Some(Plucked {
        name: name.to_string(),
        rgba: decoded.rgba,
        width: decoded.width,
        height: decoded.height,
    })
}

pub fn stand_the_pictures(pictures: &mut Pictures) {
    pictures.held[theme::FACE_SLOT as usize] = decode(theme::FACE_NAME, FACE);
    pictures.held[theme::STONE_SLOT as usize] = decode(theme::STONE_NAME, STONE);
    pictures.revision += 1;
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
const PICTURES: [&str; 6] = ["png", "jpg", "jpeg", "bmp", "tga", "webp"];

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn pluck_a_picture() -> Result<Plucked, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Pictures", &PICTURES)
        .set_title("Open a picture for the graph")
        .pick_file()
    else {
        return Err(String::new());
    };
    let name = path
        .file_name()
        .map(|leaf| leaf.to_string_lossy().to_string())
        .unwrap_or_else(|| "picture".to_string());
    let bytes = std::fs::read(&path).map_err(|error| format!("{name} did not read: {error}"))?;
    let decoded = nightshade_render::data::decoded_image::decode_to_rgba8(&bytes)
        .map_err(|error| format!("{name} did not decode: {error}"))?;
    Ok(Plucked {
        name,
        rgba: decoded.rgba,
        width: decoded.width,
        height: decoded.height,
    })
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn pluck_a_picture() -> Result<Plucked, String> {
    Err("there is no file dialog here".to_string())
}

pub fn keep_a_picture(pictures: &mut Pictures, plucked: Plucked) -> u32 {
    let free = SLOTS - KEPT;
    let slot = KEPT + pictures.next % free;
    pictures.next = (pictures.next + 1) % free;
    pictures.held[slot] = Some(plucked);
    pictures.revision += 1;
    slot as u32
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn write_the_wgsl(source: &str) -> Result<String, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Shader", &["wgsl"])
        .set_file_name("graph.wgsl")
        .set_title("Write the shader to")
        .save_file()
    else {
        return Err(String::new());
    };
    let path = if path.extension().is_some() {
        path
    } else {
        path.with_extension("wgsl")
    };
    std::fs::write(&path, source).map_err(|error| format!("the shader did not write: {error}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn write_the_wgsl(_source: &str) -> Result<String, String> {
    Err("there is nowhere to write a file here".to_string())
}

pub fn place_a_kind(commands: &mut Commands, kind: Kind, place: Vec2) -> Entity {
    let node = commands.spawn((
        NodePlace(place),
        NodeTitle(title_of(kind).to_string()),
        NodeInlets(inlets_of(kind)),
        NodeOutlets(outlets_of(kind)),
        NodeTint(tint_of(kind)),
    ));
    commands.set(node, NodeKind(kind));
    commands.set(node, NodeInletNames(owned(inlet_names_of(kind))));
    commands.set(node, NodeOutletNames(owned(outlet_names_of(kind))));
    let (_low, _high, start) = value_span_of(kind);
    commands.set(node, NodeNote(note_of(kind, start)));
    commands.set(
        node,
        NodeField(takes_a_value(kind) || kind == Kind::Picture),
    );
    if takes_a_value(kind) {
        commands.set(node, NodeValue(start));
    }
    if kind == Kind::Picture {
        commands.set(node, NodeSlot(theme::FACE_SLOT));
    }
    node
}

pub fn string_a_wire(commands: &mut Commands, from: Socket, to: Socket) {
    commands.spawn((WireFrom(from), WireTo(to)));
}

pub fn clear_the_graph(commands: &mut Commands, storage: &Storage) {
    let nodes: Vec<Entity> = storage.query_entities::<(NodePlace,)>().collect();
    for node in nodes {
        commands.despawn(node);
    }
    let wires: Vec<Entity> = storage.query_entities::<(WireFrom,)>().collect();
    for wire in wires {
        commands.despawn(wire);
    }
}

pub fn build_a_preset(commands: &mut Commands, preset: &Preset, pictures: &Pictures) {
    let mut placed = Vec::with_capacity(preset.nodes.len());
    for (kind, across, down, value) in preset.nodes {
        let node = place_a_kind(
            commands,
            *kind,
            theme::PRESET_CORNER + Vec2::new(*across, *down),
        );
        if takes_a_value(*kind) {
            commands.set(node, NodeValue(*value));
            commands.set(node, NodeNote(note_of(*kind, *value)));
        }
        if *kind == Kind::Picture {
            let slot = (*value as u32).min(SLOTS as u32 - 1);
            commands.set(node, NodeSlot(slot));
            let named = pictures.held[slot as usize]
                .as_ref()
                .map(|held| held.name.clone())
                .unwrap_or_else(|| "no picture".to_string());
            commands.set(node, NodeNote(named));
        }
        placed.push(node);
    }
    for (from, outlet, to, inlet) in preset.wires {
        let Some(tail) = placed.get(*from).copied() else {
            continue;
        };
        let Some(head) = placed.get(*to).copied() else {
            continue;
        };
        string_a_wire(
            commands,
            Socket {
                node: tail,
                index: *outlet,
            },
            Socket {
                node: head,
                index: *inlet,
            },
        );
    }
}
