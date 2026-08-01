use crate::commands::interface::show_the_picture;
use crate::commands::interface::write_the_card;
use crate::data::{MODELS, SOUNDS, TOTAL};
use crate::queries::landed;
use crate::resources::{Handles, Pictures, Progress, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_audio::contract::PendingSound;
use nightshade_audio::resources::Sounds;
use nightshade_load::{gather, in_flight, take};
use nightshade_state::prelude::NextState;

pub fn watch_the_loads(
    mut progress: ResMut<Progress>,
    mut handles: ResMut<Handles>,
    mut pictures: ResMut<Pictures>,
    mut next: ResMut<NextState<Screen>>,
    models: Res<nightshade_gltf::Loading>,
    sounds: Res<Sounds>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    for (handle, _) in gather(&mut pictures, 1) {
        progress.pictures_landed += 1;
        let Some(outcome) = take(&mut pictures, handle) else {
            continue;
        };
        match outcome {
            Ok(picture) => handles.picture_tree = show_the_picture(&mut build, &picture),
            Err(error) => {
                progress.failed += 1;
                tracing::error!("the picture did not load: {error}");
            }
        }
    }

    progress.models_landed = MODELS.len() - in_flight(&models).min(MODELS.len());
    progress.sounds_landed = SOUNDS - in_flight(&sounds).min(SOUNDS);

    if !handles.played && landed(&progress) >= TOTAL {
        handles.played = true;
        commands.spawn((PendingSound {
            clip: theme::FOOTSTEP_NAME.to_string(),
            volume: 1.0,
            ..Default::default()
        },));
        next.pending = Some(Screen::Playing);
    }

    let context = build.context();
    write_the_card(&mut progress, &handles, context.dirty, context.commands);
}
