use std::cell::{Cell, RefCell};
use std::collections::HashMap;

pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

pub enum FileError {
    NotFound(String),
    ReadError(String),
    WriteError(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::NotFound(path) => write!(formatter, "File not found: {}", path),
            FileError::ReadError(message) => write!(formatter, "Read error: {}", message),
            FileError::WriteError(message) => write!(formatter, "Write error: {}", message),
        }
    }
}

pub struct LoadedFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

thread_local! {
    static COMPLETED_LOADS: RefCell<HashMap<u64, LoadedFile>> = RefCell::new(HashMap::new());
    static NEXT_LOAD_ID: Cell<u64> = const { Cell::new(1) };
}

fn allocate_load_id() -> u64 {
    NEXT_LOAD_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

fn store_completed_load(id: u64, file: LoadedFile) {
    COMPLETED_LOADS.with(|cell| {
        cell.borrow_mut().insert(id, file);
    });
}

#[derive(Clone, Copy)]
pub struct PendingFileLoad {
    id: u64,
}

impl PendingFileLoad {
    pub fn empty() -> Self {
        Self {
            id: allocate_load_id(),
        }
    }

    pub fn ready(file: LoadedFile) -> Self {
        let id = allocate_load_id();
        store_completed_load(id, file);
        Self { id }
    }

    pub fn is_ready(&self) -> bool {
        COMPLETED_LOADS.with(|cell| cell.borrow().contains_key(&self.id))
    }

    pub fn take(&self) -> Option<LoadedFile> {
        COMPLETED_LOADS.with(|cell| cell.borrow_mut().remove(&self.id))
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
pub fn pick_file(filters: &[FileFilter]) -> Option<std::path::PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    for filter in filters {
        let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&filter.name, &extensions);
    }
    dialog.pick_file()
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
pub fn pick_folder() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
pub fn save_file_dialog(
    filters: &[FileFilter],
    default_filename: Option<&str>,
) -> Option<std::path::PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(filename) = default_filename {
        dialog = dialog.set_file_name(filename);
    }
    for filter in filters {
        let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&filter.name, &extensions);
    }
    dialog.save_file()
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
pub fn read_file(path: &std::path::Path) -> Result<Vec<u8>, FileError> {
    if !path.exists() {
        return Err(FileError::NotFound(path.display().to_string()));
    }
    std::fs::read(path).map_err(|error| FileError::ReadError(error.to_string()))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
pub fn write_file(path: &std::path::Path, bytes: &[u8]) -> Result<(), FileError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| FileError::WriteError(error.to_string()))?;
    }
    std::fs::write(path, bytes).map_err(|error| FileError::WriteError(error.to_string()))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
pub fn save_file(filename: &str, data: &[u8], filters: &[FileFilter]) -> Result<(), FileError> {
    let mut dialog = rfd::FileDialog::new().set_file_name(filename);
    for filter in filters {
        let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&filter.name, &extensions);
    }
    match dialog.save_file() {
        Some(path) => write_file(&path, data),
        None => Ok(()),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
pub fn request_file_load(filters: &[FileFilter]) -> PendingFileLoad {
    let mut dialog = rfd::FileDialog::new();
    for filter in filters {
        let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&filter.name, &extensions);
    }
    match dialog.pick_file() {
        Some(path) => {
            let name = path
                .file_name()
                .map(|os_name| os_name.to_string_lossy().to_string())
                .unwrap_or_default();
            match std::fs::read(&path) {
                Ok(bytes) => PendingFileLoad::ready(LoadedFile { name, bytes }),
                Err(_) => PendingFileLoad::empty(),
            }
        }
        None => PendingFileLoad::empty(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_directory(path: &std::path::Path) {
    let directory = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(directory)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(directory).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(directory)
            .spawn();
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub fn save_file(filename: &str, data: &[u8], _filters: &[FileFilter]) -> Result<(), FileError> {
    use wasm_bindgen::JsCast;

    let window =
        web_sys::window().ok_or_else(|| FileError::WriteError("No window object".to_string()))?;
    let document = window
        .document()
        .ok_or_else(|| FileError::WriteError("No document object".to_string()))?;

    let uint8_array = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::new();
    parts.push(&uint8_array);

    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/octet-stream");

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .map_err(|_| FileError::WriteError("Failed to create Blob".to_string()))?;

    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| FileError::WriteError("Failed to create object URL".to_string()))?;

    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(|_| FileError::WriteError("Failed to create anchor element".to_string()))?
        .dyn_into()
        .map_err(|_| FileError::WriteError("Failed to cast to anchor".to_string()))?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub fn request_file_load(filters: &[FileFilter]) -> PendingFileLoad {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    let pending = PendingFileLoad::empty();
    let pending_id = pending.id;

    let Some(window) = web_sys::window() else {
        return pending;
    };
    let Some(document) = window.document() else {
        return pending;
    };
    let Some(input) = document
        .create_element("input")
        .ok()
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return pending;
    };

    input.set_type("file");

    let accept: String = filters
        .iter()
        .flat_map(|filter| {
            filter
                .extensions
                .iter()
                .map(|extension| format!(".{}", extension))
        })
        .collect::<Vec<_>>()
        .join(",");
    if !accept.is_empty() {
        input.set_accept(&accept);
    }

    let input_for_closure = input.clone();
    let closure = Closure::once(Box::new(move |_event: web_sys::Event| {
        let captured_input = input_for_closure;

        wasm_bindgen_futures::spawn_local(async move {
            if let Some(files) = captured_input.files()
                && let Some(file) = files.get(0)
            {
                let name = file.name();
                let array_buffer_promise = file.array_buffer();
                if let Ok(array_buffer) =
                    wasm_bindgen_futures::JsFuture::from(array_buffer_promise).await
                {
                    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                    let bytes = uint8_array.to_vec();
                    store_completed_load(pending_id, LoadedFile { name, bytes });
                }
            }
        });
    }) as Box<dyn FnOnce(_)>);

    input.set_onchange(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    input.click();

    pending
}
