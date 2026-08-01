use super::*;

pub(super) fn import_lenient_from_path(
    path: &Path,
) -> Result<GltfImport, Box<dyn std::error::Error>> {
    let base = path.parent();
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let Gltf { document, blob } = Gltf::from_reader_without_validation(reader)?;
    let buffers = import_buffers(&document, base, blob)?;
    let encoded_images = extract_encoded_images_from_path(&document, base, &buffers)?;
    Ok((document, buffers, encoded_images))
}

pub(super) fn import_lenient_from_slice(
    bytes: &[u8],
) -> Result<GltfImport, Box<dyn std::error::Error>> {
    let Gltf { document, blob } = Gltf::from_slice_without_validation(bytes)?;
    let buffers = import_buffers(&document, None, blob)?;
    let encoded_images = extract_encoded_images_from_path(&document, None, &buffers)?;
    Ok((document, buffers, encoded_images))
}

pub(super) fn extract_encoded_images_from_path(
    doc: &Document,
    base: Option<&Path>,
    buffers: &[buffer::Data],
) -> Result<EncodedImages, Box<dyn std::error::Error>> {
    let mut out = Vec::with_capacity(doc.images().count());
    for img in doc.images() {
        let bytes = match img.source() {
            image::Source::Uri { uri, .. } => {
                if let Some(rest) = uri.strip_prefix("data:") {
                    decode_data_uri_payload(rest)?
                } else if let Some(base) = base {
                    let decoded = percent_decode(uri);
                    let path = base.join(decoded);
                    std::fs::read(path)?
                } else {
                    return Err(format!("no base path for image URI '{}'", uri).into());
                }
            }
            image::Source::View { view, .. } => {
                let buffer = &buffers[view.buffer().index()];
                let start = view.offset();
                let end = start + view.length();
                buffer.0[start..end].to_vec()
            }
        };
        out.push(Some(bytes));
    }
    Ok(out)
}

pub(super) fn decode_data_uri_payload(rest: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let comma = rest
        .find(',')
        .ok_or("malformed data URI: missing comma separator")?;
    let header = &rest[..comma];
    let payload = &rest[comma + 1..];
    if header.split(';').any(|piece| piece == "base64") {
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.decode(payload)?)
    } else {
        Ok(percent_decode(payload).into_bytes())
    }
}
