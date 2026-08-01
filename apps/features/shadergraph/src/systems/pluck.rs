use crate::commands::{keep_a_picture, place_a_kind, pluck_a_picture, write_the_wgsl};
use crate::components::{NodeKind, NodeSlot};
use crate::data::Kind;
use crate::resources::{Chosen, Compiled, PaletteRows, Pictures, Tapping};
use crate::shader::whole_shader;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_ui::prelude::UiButtonData;
use nightshade_ui_nodegraph::prelude::NodeNote;

fn was_clicked(storage: &Storage, button: Option<nightshade::prelude::Entity>) -> bool {
    button.is_some_and(|button| {
        storage
            .get::<UiButtonData>(button)
            .is_some_and(|data| data.clicked)
    })
}

fn is_a_picture(storage: &Storage, node: nightshade::prelude::Entity) -> bool {
    storage
        .get::<NodeKind>(node)
        .is_some_and(|kind| kind.0 == Kind::Picture)
}

pub fn pluck_a_file(
    chosen: Res<Chosen>,
    rows: Res<PaletteRows>,
    tapping: Res<Tapping>,
    mut pictures: ResMut<Pictures>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let tapped = tapping.tapped.filter(|node| is_a_picture(storage, *node));
    if !was_clicked(storage, rows.pluck) && tapped.is_none() {
        return;
    }
    let plucked = match pluck_a_picture() {
        Ok(plucked) => plucked,
        Err(trouble) => {
            pictures.trouble = if trouble.is_empty() {
                None
            } else {
                Some(trouble)
            };
            return;
        }
    };

    let named = plucked.name.clone();
    let slot = keep_a_picture(&mut pictures, plucked);
    pictures.trouble = None;

    let standing = tapped.or_else(|| chosen.0.filter(|node| is_a_picture(storage, *node)));
    match standing {
        Some(node) => {
            if let Some(held) = storage.get_mut::<NodeSlot>(node) {
                held.0 = slot;
            }
            if let Some(note) = storage.get_mut::<NodeNote>(node) {
                note.0 = named;
            }
        }
        None => {
            let node = place_a_kind(&mut commands, Kind::Picture, theme::FRESH_PLACE);
            commands.set(node, NodeSlot(slot));
            commands.set(node, NodeNote(named));
        }
    }
}

pub fn write_the_source(
    rows: Res<PaletteRows>,
    compiled: Res<Compiled>,
    mut pictures: ResMut<Pictures>,
    storage: &mut Storage,
) {
    if !was_clicked(storage, rows.write) {
        return;
    }
    match write_the_wgsl(&whole_shader(&compiled.source)) {
        Ok(path) => pictures.trouble = Some(format!("wrote {path}")),
        Err(trouble) => {
            pictures.trouble = if trouble.is_empty() {
                None
            } else {
                Some(trouble)
            };
        }
    }
}
