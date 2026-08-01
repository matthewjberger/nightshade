use super::*;

pub(super) fn resolve_uri_bytes(
    uri: &str,
    resources: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if let Some(rest) = uri.strip_prefix("data:") {
        let comma = rest
            .find(',')
            .ok_or("malformed data URI: missing comma separator")?;
        let header = &rest[..comma];
        let payload = &rest[comma + 1..];
        if header.split(';').any(|piece| piece == "base64") {
            use base64::Engine;
            return Ok(base64::engine::general_purpose::STANDARD.decode(payload)?);
        }
        return Ok(percent_decode(payload).into_bytes());
    }

    let decoded = normalize_relative_path(&percent_decode(uri));
    if let Some(bytes) = resources.get(&decoded) {
        return Ok(bytes.clone());
    }
    Err(format!("missing external resource for URI '{}'", uri).into())
}

pub(super) fn normalize_relative_path(input: &str) -> String {
    let normalized = input.replace('\\', "/");
    let mut parts: Vec<&str> = Vec::new();
    for piece in normalized.split('/') {
        match piece {
            "" | "." => continue,
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

pub(super) fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
        {
            out.push((high << 4) | low);
            index += 3;
            continue;
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

pub(super) fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
