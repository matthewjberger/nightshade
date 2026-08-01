use crate::data::{FIELD_SEPARATOR, FILE_SUFFIX, HEADER_FIELDS, HEADER_TAG, Save, SaveHeader};
use nightshade_store::prelude::store_read;

pub(crate) fn save_file<T: Save>(slot: u32) -> String {
    format!("{}-{slot}{FILE_SUFFIX}", T::NAME)
}

pub(crate) fn save_header_line<T: Save>(label: &str) -> String {
    let label: String = label
        .chars()
        .map(|letter| if letter.is_control() { ' ' } else { letter })
        .collect();
    format!(
        "{HEADER_TAG}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{label}\n",
        T::NAME,
        T::VERSION
    )
}

pub fn save_header<T: Save>(slot: u32) -> Option<SaveHeader> {
    let bytes = store_read(T::FOLDER, &save_file::<T>(slot))?;
    let (line, _) = split_the_file(&bytes)?;
    read_the_header(line)
}

pub fn save_read<T: Save>(slot: u32) -> Option<T> {
    let bytes = store_read(T::FOLDER, &save_file::<T>(slot))?;
    let (line, payload) = split_the_file(&bytes)?;
    let header = read_the_header(line)?;
    if header.name != T::NAME || header.version > T::VERSION {
        return None;
    }
    if header.version < T::VERSION {
        return T::migrate(header.version, payload);
    }
    T::CODEC.decode(payload)
}

pub fn save_slots<T: Save>() -> Vec<(u32, SaveHeader)> {
    (0..T::SLOTS)
        .filter_map(|slot| save_header::<T>(slot).map(|header| (slot, header)))
        .collect()
}

fn split_the_file(bytes: &[u8]) -> Option<(&str, &[u8])> {
    let end = bytes.iter().position(|byte| *byte == b'\n')?;
    let line = std::str::from_utf8(&bytes[..end]).ok()?;
    Some((line, &bytes[end + 1..]))
}

fn read_the_header(line: &str) -> Option<SaveHeader> {
    let mut fields = line.splitn(HEADER_FIELDS, FIELD_SEPARATOR);
    if fields.next()? != HEADER_TAG {
        return None;
    }
    let name = fields.next()?.to_string();
    let version = fields.next()?.parse().ok()?;
    let label = fields.next().unwrap_or_default().to_string();
    Some(SaveHeader {
        name,
        version,
        label,
    })
}
