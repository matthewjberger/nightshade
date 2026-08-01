use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::ui_is_text_editor;
use nightshade_ui::resources::{RetainedUiInteraction, RetainedUiRuntime};

use crate::commands::files::{offer_texture, route_drop};
#[cfg(not(target_arch = "wasm32"))]
use crate::commands::load::remember;
use crate::data::{GizmoMode, ViewerAction};
use crate::queries::files::is_picture;
#[cfg(not(target_arch = "wasm32"))]
use crate::queries::files::{is_model_or_sky, read_path};
use crate::resources::{TextureDrop, ViewerResources};

const SETTLE_FRAMES: u32 = 3;

pub fn handle_events(
    mut viewer: ResMut<ViewerResources>,
    reader: EventReader<AppEvent>,
    input: Res<Input>,
    interaction: Res<RetainedUiInteraction>,
    runtime: Res<RetainedUiRuntime>,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;
    viewer.history.frame += 1;
    let under_pointer = nightshade_ui::ui_pointer_target_of(&runtime, &interaction, storage);
    let texture_row = under_pointer.and_then(|target| {
        viewer
            .handles
            .material
            .texture_rows
            .iter()
            .find(|row| row.nodes.contains(&target))
            .map(|row| row.slot)
    });
    let typing = interaction
        .focused_entity
        .is_some_and(|focused| ui_is_text_editor(storage, focused));
    let control = input.keyboard.is_key_pressed(KeyCode::ControlLeft)
        || input.keyboard.is_key_pressed(KeyCode::ControlRight)
        || input.keyboard.is_key_pressed(KeyCode::SuperLeft)
        || input.keyboard.is_key_pressed(KeyCode::SuperRight);
    let shift = input.keyboard.is_key_pressed(KeyCode::ShiftLeft)
        || input.keyboard.is_key_pressed(KeyCode::ShiftRight);

    for event in reader.iter() {
        match event {
            AppEvent::Keyboard {
                key,
                state: KeyState::Pressed,
            } if !typing => match (key, control) {
                (KeyCode::KeyZ, true) if shift => viewer.actions.push(ViewerAction::Redo),
                (KeyCode::KeyZ, true) => viewer.actions.push(ViewerAction::Undo),
                (KeyCode::KeyY, true) => viewer.actions.push(ViewerAction::Redo),
                (KeyCode::KeyO, true) => viewer.actions.push(ViewerAction::ImportFile),
                (KeyCode::KeyS, true) if shift => viewer.actions.push(ViewerAction::ExportModel),
                (KeyCode::KeyS, true) => viewer.actions.push(ViewerAction::Screenshot),
                (KeyCode::KeyH, false) => viewer.actions.push(ViewerAction::ToggleInterface),
                (KeyCode::KeyX, false) | (KeyCode::Delete, false) => {
                    viewer.actions.push(ViewerAction::DeleteSelected)
                }
                (KeyCode::KeyF, false) => viewer.actions.push(ViewerAction::FrameModel),
                (KeyCode::KeyW, false) => viewer
                    .actions
                    .push(ViewerAction::SetGizmoMode(GizmoMode::Move)),
                (KeyCode::KeyE, false) => viewer
                    .actions
                    .push(ViewerAction::SetGizmoMode(GizmoMode::Rotate)),
                (KeyCode::KeyR, false) => viewer
                    .actions
                    .push(ViewerAction::SetGizmoMode(GizmoMode::Scale)),
                (KeyCode::Space, false) if !viewer.model.clips.is_empty() => {
                    viewer.actions.push(ViewerAction::TogglePlayback)
                }
                (KeyCode::Escape, _) => {
                    if viewer.view.browser.is_some() {
                        viewer.actions.push(ViewerAction::CloseBrowser);
                    } else if viewer.view.add_open {
                        viewer.actions.push(ViewerAction::CloseAddMenu);
                    } else if viewer.view.recent_open {
                        viewer.actions.push(ViewerAction::CloseRecent);
                    } else {
                        viewer.actions.push(ViewerAction::ShowInterface);
                    }
                }
                _ => {}
            },
            AppEvent::FileDropped(file) => {
                if let Some(slot) = texture_row
                    && is_picture(&file.name)
                {
                    offer_texture(
                        &viewer.incoming,
                        TextureDrop {
                            slot,
                            name: file.name.clone(),
                            bytes: file.data.clone(),
                        },
                    );
                    continue;
                }
                viewer
                    .dropped
                    .files
                    .push((file.name.clone(), file.data.clone()));
                viewer.dropped.settle = SETTLE_FRAMES;
            }
            #[cfg(not(target_arch = "wasm32"))]
            AppEvent::FileDroppedPath(path) => {
                if let Some(file) = read_path(path) {
                    if let Some(slot) = texture_row
                        && is_picture(&file.0)
                    {
                        offer_texture(
                            &viewer.incoming,
                            TextureDrop {
                                slot,
                                name: file.0,
                                bytes: file.1,
                            },
                        );
                        continue;
                    }
                    if is_model_or_sky(&file.0) {
                        remember(viewer, &path.to_string_lossy());
                    }
                    viewer.dropped.files.push(file);
                    viewer.dropped.settle = SETTLE_FRAMES;
                }
            }
            _ => {}
        }
    }

    if viewer.dropped.settle > 0 {
        viewer.dropped.settle -= 1;
        if viewer.dropped.settle == 0 {
            let files = std::mem::take(&mut viewer.dropped.files);
            route_drop(&viewer.incoming, files);
        }
    }
}
