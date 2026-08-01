use crate::commands::{import_gltf_from_bytes, import_gltf_from_path, import_gltf_with_resources};
use crate::data::{Gltf, LOAD_CREW};
use nalgebra_glm::Vec3;
use nightshade_load::{Reader, Source};
use nightshade_prefab::data::ModelImport;

impl Reader for Gltf {
    type Item = ModelImport;
    type Note = Option<Vec3>;

    fn hands() -> usize {
        LOAD_CREW
    }

    fn read(source: Source) -> Result<ModelImport, String> {
        let outcome = match source {
            Source::Path(path) => import_gltf_from_path(&path),
            Source::Bytes(bytes) => import_gltf_from_bytes(&bytes),
            Source::Parts { head, files } => import_gltf_with_resources(&head, &files),
            #[cfg(feature = "fetch")]
            Source::Url(url) => Err(format!("{url} was never read").into()),
        };
        outcome.map_err(|error| error.to_string())
    }

    fn dependencies(head: &[u8]) -> Vec<String> {
        let Ok(document) = gltf::Gltf::from_slice(head) else {
            return Vec::new();
        };
        let mut uris: Vec<String> = Vec::new();
        for buffer in document.buffers() {
            if let gltf::buffer::Source::Uri(uri) = buffer.source()
                && !uri.starts_with("data:")
                && !uris.iter().any(|existing| existing == uri)
            {
                uris.push(uri.to_string());
            }
        }
        for image in document.images() {
            if let gltf::image::Source::Uri { uri, .. } = image.source()
                && !uri.starts_with("data:")
                && !uris.iter().any(|existing| existing == uri)
            {
                uris.push(uri.to_string());
            }
        }
        uris
    }
}
