use crate::commands::loads::ask_for_everything;
use crate::resources::{Handles, Pictures, Progress, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_audio::resources::{AudioEngine, Sounds};
use nightshade_load::clear;
use nightshade_scene::prelude::LocalTransform;
use nightshade_state::prelude::NextState;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;

pub fn reload_on_click(
    mut handles: ResMut<Handles>,
    mut progress: ResMut<Progress>,
    mut models: ResMut<nightshade_gltf::Loading>,
    mut pictures: ResMut<Pictures>,
    mut sounds: ResMut<Sounds>,
    mut engine: ResMut<AudioEngine>,
    mut next: ResMut<NextState<Screen>>,
    frame: Res<RetainedUiFrame>,
    mut commands: Commands,
    mut scene: Query<(&LocalTransform,)>,
) {
    if handles.reload == Entity::default() {
        return;
    }
    if !ui_button_clicks(&frame).any(|entity| entity == handles.reload) {
        return;
    }

    let mut stale = Vec::new();
    scene.for_each_mut(|entity, _| {
        if entity != handles.camera && entity != handles.sun {
            stale.push(entity);
        }
    });
    for entity in stale {
        commands.despawn_recursive(entity);
    }
    if handles.picture_tree != Entity::default() {
        commands.despawn_recursive(handles.picture_tree);
        handles.picture_tree = Entity::default();
    }

    clear(&mut models);
    clear(&mut pictures);
    clear(&mut sounds);
    engine.sound_cache.remove(theme::FOOTSTEP_NAME);

    ask_for_everything(
        &mut progress,
        &mut handles,
        &mut models,
        &mut pictures,
        &mut sounds,
    );
    next.pending = Some(Screen::Loading);
}
