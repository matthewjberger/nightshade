use crate::data::assets::{MODELS, kit_textures};
use nightshade_gltf::Loading;
use nightshade_load::{LoadHandle, load_model_with_files};

pub fn request_model(loading: &mut Loading, index: usize) -> LoadHandle {
    let file = &MODELS[index];
    load_model_with_files(
        loading,
        file.bytes.to_vec(),
        kit_textures(file.kit)
            .iter()
            .map(|(path, bytes)| ((*path).to_string(), bytes.to_vec())),
    )
}
