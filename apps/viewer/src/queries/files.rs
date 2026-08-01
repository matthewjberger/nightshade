pub fn is_picture(name: &str) -> bool {
    let lowered = name.to_lowercase();
    [
        "png", "jpg", "jpeg", "webp", "tga", "bmp", "tif", "tiff", "ktx2",
    ]
    .iter()
    .any(|extension| lowered.ends_with(&format!(".{extension}")))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_model_or_sky(name: &str) -> bool {
    let lowered = name.to_lowercase();
    ["glb", "gltf", "hdr", "zip"]
        .iter()
        .any(|extension| lowered.ends_with(&format!(".{extension}")))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_path(path: &std::path::Path) -> Option<(String, Vec<u8>)> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let data = std::fs::read(path).ok()?;
    Some((name, data))
}

pub fn unzip(bytes: &[u8]) -> Option<Vec<(String, Vec<u8>)>> {
    use std::io::Read;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let mut files = Vec::new();
    for index in 0..archive.len() {
        let Ok(mut entry) = archive.by_index(index) else {
            continue;
        };
        if !entry.is_file() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        let mut data = Vec::new();
        if entry.read_to_end(&mut data).is_ok() {
            files.push((name, data));
        }
    }
    (!files.is_empty()).then_some(files)
}

pub fn is_fbx(name: &str) -> bool {
    name.to_lowercase().ends_with(".fbx")
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn find_blender() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    if let Ok(named) = std::env::var("NIGHTSHADE_BLENDER") {
        let path = PathBuf::from(named);
        if path.is_file() {
            return Some(path);
        }
    }

    let executable = if cfg!(windows) {
        "blender.exe"
    } else {
        "blender"
    };
    if let Ok(paths) = std::env::var("PATH") {
        for directory in std::env::split_paths(&paths) {
            let candidate = directory.join(executable);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE") {
        roots.push(PathBuf::from(&home).join("scoop/apps/blender/current"));
    }
    if let Ok(home) = std::env::var("HOME") {
        roots.push(PathBuf::from(&home).join("scoop/apps/blender/current"));
    }
    roots.push(PathBuf::from("/Applications/Blender.app/Contents/MacOS"));
    roots.push(PathBuf::from("/usr/bin"));
    roots.push(PathBuf::from("/usr/local/bin"));
    roots.push(PathBuf::from("/snap/bin"));
    for root in roots {
        let candidate = root.join(executable);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    for base in [
        "C:/Program Files/Blender Foundation",
        "C:/Program Files (x86)/Steam/steamapps/common/Blender",
    ] {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let candidate = entry.path().join(executable);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        let direct = PathBuf::from(base).join(executable);
        if direct.is_file() {
            return Some(direct);
        }
    }

    None
}
