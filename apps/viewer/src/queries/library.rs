use crate::data::PREVIEW_MARK;
#[cfg(not(target_arch = "wasm32"))]
use crate::data::{LIBRARY_DEPTH, LIBRARY_LIMIT, LibraryKind, PREVIEW_EXTENSIONS};
use crate::resources::{LibraryEntry, ViewerResources};

#[cfg(not(target_arch = "wasm32"))]
pub fn default_folder() -> Option<String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()?;
    let folder = std::path::Path::new(&home).join("assets");
    folder
        .is_dir()
        .then(|| folder.to_string_lossy().to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn default_folder() -> Option<String> {
    None
}

pub fn preview_key(path: &str) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    stamp_of(path).hash(&mut hasher);
    format!("{}{:016x}.png", PREVIEW_MARK, hasher.finish())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn preview_name(key: &str) -> Option<&str> {
    key.strip_prefix(PREVIEW_MARK)
}

#[cfg(not(target_arch = "wasm32"))]
fn stamp_of(path: &str) -> (u64, u64) {
    let Ok(data) = std::fs::metadata(path) else {
        return (0, 0);
    };
    let modified = data
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|span| span.as_secs())
        .unwrap_or(0);
    (data.len(), modified)
}

#[cfg(target_arch = "wasm32")]
fn stamp_of(_path: &str) -> (u64, u64) {
    (0, 0)
}

pub fn is_remote(key: &str) -> bool {
    key.starts_with("http://") || key.starts_with("https://")
}

pub fn display_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn scan(folders: &[String], files: &[String]) -> Vec<LibraryEntry> {
    let mut entries = Vec::new();
    for path in files {
        if let Some(entry) = entry_of(std::path::Path::new(path)) {
            entries.push(entry);
        }
    }
    for folder in folders {
        walk(std::path::Path::new(folder), 0, &mut entries);
    }
    entries.sort_by_cached_key(|entry| (entry.name.to_lowercase(), entry.path.clone()));
    entries.dedup_by(|left, right| left.path == right.path);
    entries.truncate(LIBRARY_LIMIT);
    entries
}

#[cfg(target_arch = "wasm32")]
pub fn scan(_folders: &[String], _files: &[String]) -> Vec<LibraryEntry> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn walk(folder: &std::path::Path, depth: u32, entries: &mut Vec<LibraryEntry>) {
    use std::collections::HashSet;

    if depth > LIBRARY_DEPTH || entries.len() >= LIBRARY_LIMIT {
        return;
    }
    let Ok(listing) = std::fs::read_dir(folder) else {
        return;
    };
    let mut below = Vec::new();
    let mut files: Vec<(String, std::path::PathBuf)> = Vec::new();
    for item in listing.flatten() {
        let Ok(kind) = item.file_type() else {
            continue;
        };
        if kind.is_dir() {
            below.push(item.path());
            continue;
        }
        files.push((item.file_name().to_string_lossy().to_string(), item.path()));
    }
    let beside: HashSet<String> = files.iter().map(|(name, _)| name.to_lowercase()).collect();
    for (name, path) in &files {
        let Some(kind) = LibraryKind::of(name) else {
            continue;
        };
        entries.push(LibraryEntry {
            thumbnail: picture_beside(name, path, &beside),
            path: path.to_string_lossy().to_string(),
            name: name.clone(),
            kind,
        });
    }
    below.sort();
    for folder in below {
        walk(&folder, depth + 1, entries);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn picture_beside(
    name: &str,
    path: &std::path::Path,
    beside: &std::collections::HashSet<String>,
) -> Option<String> {
    let stem = name.rsplit_once('.').map(|(head, _)| head).unwrap_or(name);
    for extension in PREVIEW_EXTENSIONS {
        let candidate = format!("{}.{extension}", stem.to_lowercase());
        if beside.contains(&candidate) {
            return Some(
                path.with_file_name(format!("{stem}.{extension}"))
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn entry_of(path: &std::path::Path) -> Option<LibraryEntry> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let kind = LibraryKind::of(&name)?;
    if !path.is_file() {
        return None;
    }
    let mut beside = std::collections::HashSet::new();
    for extension in PREVIEW_EXTENSIONS {
        let candidate = path.with_extension(extension);
        if candidate.is_file()
            && let Some(found) = candidate.file_name()
        {
            beside.insert(found.to_string_lossy().to_lowercase());
        }
    }
    Some(LibraryEntry {
        thumbnail: picture_beside(&name, path, &beside),
        path: path.to_string_lossy().to_string(),
        name,
        kind,
    })
}

pub fn roots(viewer: &ViewerResources) -> Vec<String> {
    let mut folders: Vec<String> = viewer.library.built_in.iter().cloned().collect();
    folders.extend(viewer.library.folders.iter().cloned());
    folders
}
