use crate::ecs::world::World;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_DROPPED_FILES: std::cell::RefCell<Vec<crate::platform::input::resources::DroppedFile>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn collect_directory_files(
    root: &std::path::Path,
    current: &std::path::Path,
    files: &mut Vec<crate::platform::input::resources::DroppedFile>,
) {
    let entries = match std::fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_directory_files(root, &path, files);
        } else if path.is_file() {
            let Ok(data) = std::fs::read(&path) else {
                continue;
            };
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let prefix = root
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| format!("{}/", name))
                .unwrap_or_default();
            let name = format!("{}{}", prefix, relative);
            files.push(crate::platform::input::resources::DroppedFile { name, data });
        }
    }
}

pub(crate) fn process_dropped_files(world: &mut World) {
    #[cfg(target_arch = "wasm32")]
    {
        WASM_DROPPED_FILES.with(|files| {
            world
                .res_mut::<crate::ecs::loading::LoadingState>()
                .dropped_files
                .append(&mut files.borrow_mut());
        });
    }

    let dropped_files: Vec<_> = world
        .res_mut::<crate::ecs::loading::LoadingState>()
        .dropped_files
        .drain(..)
        .collect();
    for file in dropped_files {
        crate::ecs::events::emit_app_event(
            world,
            crate::platform::input::events::AppEvent::FileDropped(file),
        );
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn setup_wasm_file_drop() {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");
    let canvas = document
        .get_element_by_id("canvas")
        .expect("no canvas element");

    let dragover_closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        event.prevent_default();
        event.stop_propagation();
    }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("dragover", dragover_closure.as_ref().unchecked_ref())
        .expect("failed to add dragover listener");
    dragover_closure.forget();

    let drop_closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        event.prevent_default();
        event.stop_propagation();

        let Some(data_transfer) = event.data_transfer() else {
            return;
        };

        let items = data_transfer.items();
        let mut entries: Vec<web_sys::FileSystemEntry> = Vec::new();
        for index in 0..items.length() {
            let Some(item) = items.get(index) else {
                continue;
            };
            if let Ok(Some(entry)) = item.webkit_get_as_entry() {
                entries.push(entry);
            }
        }

        if entries.is_empty() {
            return;
        }

        wasm_bindgen_futures::spawn_local(async move {
            let file_entries = collect_file_entries(entries).await;
            if file_entries.is_empty() {
                return;
            }
            let read_futures = file_entries
                .into_iter()
                .map(|(file_entry, path)| async move { read_file_entry(file_entry, path).await });
            let results = futures::future::join_all(read_futures).await;
            let batch: Vec<crate::platform::input::resources::DroppedFile> =
                results.into_iter().flatten().collect();
            if !batch.is_empty() {
                WASM_DROPPED_FILES.with(|files| files.borrow_mut().extend(batch));
            }
        });
    }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("drop", drop_closure.as_ref().unchecked_ref())
        .expect("failed to add drop listener");
    drop_closure.forget();
}

#[cfg(target_arch = "wasm32")]
async fn collect_file_entries(
    entries: Vec<web_sys::FileSystemEntry>,
) -> Vec<(web_sys::FileSystemFileEntry, String)> {
    use wasm_bindgen::JsCast;

    let mut files: Vec<(web_sys::FileSystemFileEntry, String)> = Vec::new();
    let mut worklist: Vec<(web_sys::FileSystemEntry, String)> =
        entries.into_iter().map(|e| (e, String::new())).collect();
    while let Some((entry, prefix)) = worklist.pop() {
        if entry.is_file() {
            let file_entry: web_sys::FileSystemFileEntry = entry.unchecked_into();
            let path = if prefix.is_empty() {
                file_entry.name()
            } else {
                format!("{}/{}", prefix, file_entry.name())
            };
            files.push((file_entry, path));
        } else if entry.is_directory() {
            let directory_entry: web_sys::FileSystemDirectoryEntry = entry.unchecked_into();
            let next_prefix = if prefix.is_empty() {
                directory_entry.name()
            } else {
                format!("{}/{}", prefix, directory_entry.name())
            };
            let reader = directory_entry.create_reader();
            loop {
                let promise = js_sys::Promise::new(&mut |resolve, reject| {
                    let _ = reader.read_entries_with_callback_and_callback(&resolve, &reject);
                });
                let entries_value = match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(value) => value,
                    Err(_) => break,
                };
                let array = js_sys::Array::from(&entries_value);
                if array.length() == 0 {
                    break;
                }
                for index in 0..array.length() {
                    let raw = array.get(index);
                    let child: web_sys::FileSystemEntry = raw.unchecked_into();
                    worklist.push((child, next_prefix.clone()));
                }
            }
        }
    }
    files
}

#[cfg(target_arch = "wasm32")]
async fn read_file_entry(
    file_entry: web_sys::FileSystemFileEntry,
    path: String,
) -> Option<crate::platform::input::resources::DroppedFile> {
    use wasm_bindgen::JsCast;

    let file_promise = js_sys::Promise::new(&mut |resolve, reject| {
        file_entry.file_with_callback_and_callback(&resolve, &reject);
    });
    let file_value = wasm_bindgen_futures::JsFuture::from(file_promise)
        .await
        .ok()?;
    let file: web_sys::File = file_value.unchecked_into();
    let buffer_value = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .ok()?;
    let array_buffer: js_sys::ArrayBuffer = buffer_value.unchecked_into();
    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();
    Some(crate::platform::input::resources::DroppedFile {
        name: path,
        data: bytes,
    })
}
