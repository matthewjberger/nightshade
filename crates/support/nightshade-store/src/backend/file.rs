use std::path::PathBuf;

pub fn write(folder: &str, name: &str, bytes: &[u8]) -> bool {
    let Some(path) = path(folder, name) else {
        return false;
    };
    std::fs::write(path, bytes).is_ok()
}

pub fn write_text(folder: &str, name: &str, text: &str) -> bool {
    write(folder, name, text.as_bytes())
}

pub fn read(folder: &str, name: &str) -> Option<Vec<u8>> {
    std::fs::read(path(folder, name)?).ok()
}

pub fn read_text(folder: &str, name: &str) -> Option<String> {
    std::fs::read_to_string(path(folder, name)?).ok()
}

pub fn remove(folder: &str, name: &str) -> bool {
    let Some(path) = path(folder, name) else {
        return false;
    };
    !path.exists() || std::fs::remove_file(path).is_ok()
}

fn path(folder: &str, name: &str) -> Option<PathBuf> {
    let mut path = root()?;
    path.push(folder);
    std::fs::create_dir_all(&path).ok()?;
    path.push(name);
    Some(path)
}

#[cfg(not(target_os = "android"))]
fn root() -> Option<PathBuf> {
    dirs::data_dir()
}

#[cfg(target_os = "android")]
fn root() -> Option<PathBuf> {
    nightshade_platform::android::android_app()?.internal_data_path()
}
