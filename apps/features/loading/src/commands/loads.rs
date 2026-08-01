use crate::data::{MODELS, Picture};
use crate::resources::{Handles, Pictures, Progress};
use crate::theme;
use nightshade_audio::resources::Sounds;
use nightshade_load::{Reader, Source, bytes_of, load, load_noted};
use nightshade_render::data::decoded_image::{BlockFamily, DecodedImage, decode_image};

impl Reader for Picture {
    type Item = DecodedImage;
    type Note = ();

    fn read(source: Source) -> Result<DecodedImage, String> {
        let bytes = bytes_of(source)?;
        decode_image(&bytes, BlockFamily::Texels)
            .map_err(|error| format!("a picture did not decode: {error}"))
    }
}

pub fn ask_for_everything(
    progress: &mut Progress,
    handles: &mut Handles,
    models: &mut nightshade_gltf::Loading,
    pictures: &mut Pictures,
    sounds: &mut Sounds,
) {
    for (name, position) in MODELS {
        load_noted(
            models,
            Source::Url(nightshade_gltf::queries::sample_model_url(name)),
            Some(position),
        );
    }
    load(pictures, Source::Bytes(theme::SHEET.to_vec()));
    load_noted(
        sounds,
        Source::Bytes(theme::FOOTSTEP.to_vec()),
        Some(theme::FOOTSTEP_NAME.to_string()),
    );
    progress.models_landed = 0;
    progress.pictures_landed = 0;
    progress.sounds_landed = 0;
    progress.failed = 0;
    progress.last_count.clear();
    progress.last_reading.clear();
    handles.played = false;
}
