use crate::resources::{Script, Vault};
use nightshade::prelude::*;
use nightshade_audio::resources::Sounds;
use nightshade_load::{Source, load_noted};

pub fn fetch(script: Res<Script>, mut vault: ResMut<Vault>, mut sounds: ResMut<Sounds>) {
    for path in &script.sounds {
        if vault.heard.contains(path) {
            continue;
        }
        vault.heard.insert(path.clone());
        load_noted(
            &mut sounds,
            Source::Path(std::path::PathBuf::from(path)),
            Some(path.clone()),
        );
    }
}
