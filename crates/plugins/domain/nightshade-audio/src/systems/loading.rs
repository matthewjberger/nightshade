use crate::data::LOAD_LIMIT;
use crate::resources::{AudioEngine, Sounds};
use nightshade::prelude::ResMut;
use nightshade_load::{gather, take};

pub fn collect_the_sounds(mut sounds: ResMut<Sounds>, mut engine: ResMut<AudioEngine>) {
    for (handle, name) in gather(&mut sounds, LOAD_LIMIT) {
        let Some(name) = name else {
            continue;
        };
        let Some(outcome) = take(&mut sounds, handle) else {
            continue;
        };
        match outcome {
            Ok(data) => {
                engine.sound_cache.insert(name, data);
            }
            Err(error) => tracing::error!("the sound {name} did not load: {error}"),
        }
    }
}
