use crate::data::{LoadHandle, ReadDone, ReadJob, Reader, Source};
use crate::resources::Loader;
use nightshade_worker::{Job, workers_submit, workers_take};
use std::marker::PhantomData;

impl<R: Reader> Job for ReadJob<R> {
    type Key = u64;
    type Done = ReadDone<R>;
    type Tools = ();

    fn key(&self) -> Self::Key {
        self.key
    }

    fn run(self, _tools: &mut Self::Tools) -> Self::Done {
        ReadDone {
            key: self.key,
            outcome: R::read(self.source),
        }
    }
}

pub fn load<R: Reader>(loader: &mut Loader<R>, source: Source) -> LoadHandle {
    load_noted(loader, source, R::Note::default())
}

pub fn load_noted<R: Reader>(loader: &mut Loader<R>, source: Source, note: R::Note) -> LoadHandle {
    loader.next += 1;
    let key = loader.next;
    loader.notes.insert(key, note);
    #[cfg(feature = "fetch")]
    if let Source::Url(url) = source {
        start_the_web_read(loader, key, url);
        return LoadHandle(key);
    }
    let job = ReadJob {
        key,
        source,
        reader: PhantomData,
    };
    workers_submit(&mut loader.crew, job, 0);
    LoadHandle(key)
}

pub fn take<R: Reader>(
    loader: &mut Loader<R>,
    handle: LoadHandle,
) -> Option<Result<R::Item, String>> {
    loader.ready.remove(&handle.0)
}

pub fn gather<R: Reader>(loader: &mut Loader<R>, limit: usize) -> Vec<(LoadHandle, R::Note)> {
    let mut landed = Vec::new();
    #[cfg(feature = "fetch")]
    for key in hand_the_web_reads_over(loader) {
        let note = loader.notes.remove(&key).unwrap_or_default();
        landed.push((LoadHandle(key), note));
    }
    for done in workers_take(&mut loader.crew, limit) {
        loader.ready.insert(done.key, done.outcome);
        let note = loader.notes.remove(&done.key).unwrap_or_default();
        landed.push((LoadHandle(done.key), note));
    }
    landed
}

pub fn clear<R: Reader>(loader: &mut Loader<R>) {
    nightshade_worker::workers_clear(&mut loader.crew);
    loader.notes.clear();
    #[cfg(feature = "fetch")]
    loader.reading.clear();
}

#[cfg(feature = "fetch")]
fn spawn_fetch(url: String) -> std::sync::Arc<std::sync::Mutex<crate::data::Fetch>> {
    use crate::data::Fetch;
    use std::sync::{Arc, Mutex};
    let slot = Arc::new(Mutex::new(Fetch::Waiting));
    let writer = slot.clone();
    ehttp::fetch(ehttp::Request::get(&url), move |result| {
        let Ok(mut slot) = writer.lock() else {
            return;
        };
        *slot = match result {
            Ok(response) if response.ok => Fetch::Arrived(response.bytes),
            Ok(response) => Fetch::Failed(format!("{url} answered {}", response.status)),
            Err(error) => Fetch::Failed(error),
        };
    });
    slot
}

#[cfg(feature = "fetch")]
fn base_of(url: &str) -> String {
    match url.rfind('/') {
        Some(index) => url[..=index].to_string(),
        None => String::new(),
    }
}

#[cfg(feature = "fetch")]
fn take_fetch(
    slot: &std::sync::Arc<std::sync::Mutex<crate::data::Fetch>>,
) -> Option<Result<Vec<u8>, String>> {
    use crate::data::Fetch;
    let Ok(mut slot) = slot.lock() else {
        return Some(Err("web read was interrupted".to_string()));
    };
    match std::mem::replace(&mut *slot, Fetch::Waiting) {
        Fetch::Waiting => None,
        Fetch::Arrived(bytes) => Some(Ok(bytes)),
        Fetch::Failed(reason) => Some(Err(reason)),
    }
}

#[cfg(feature = "fetch")]
fn start_the_web_read<R: Reader>(loader: &mut Loader<R>, key: u64, url: String) {
    use crate::data::WebRead;
    let base = base_of(&url);
    let slot = spawn_fetch(url);
    loader.reading.insert(key, WebRead::Head { slot, base });
}

#[cfg(feature = "fetch")]
fn hand_the_web_reads_over<R: Reader>(loader: &mut Loader<R>) -> Vec<u64> {
    use crate::data::WebRead;
    use std::collections::HashMap;

    let keys: Vec<u64> = loader.reading.keys().copied().collect();
    let mut submit: Vec<(u64, Source)> = Vec::new();
    let mut failed: Vec<(u64, String)> = Vec::new();
    let mut wanted: Vec<(u64, Vec<u8>, String, Vec<String>)> = Vec::new();

    for key in keys {
        let Some(entry) = loader.reading.get_mut(&key) else {
            continue;
        };
        match entry {
            WebRead::Head { slot, base } => match take_fetch(slot) {
                None => {}
                Some(Err(reason)) => failed.push((key, reason)),
                Some(Ok(head)) => {
                    let deps = R::dependencies(&head);
                    if deps.is_empty() {
                        submit.push((key, Source::Bytes(head)));
                    } else {
                        wanted.push((key, head, base.clone(), deps));
                    }
                }
            },
            WebRead::Parts {
                head,
                pending,
                files,
            } => {
                let mut arrived: Vec<(String, Vec<u8>)> = Vec::new();
                let mut broke: Option<String> = None;
                for (uri, slot) in pending.iter() {
                    match take_fetch(slot) {
                        None => {}
                        Some(Ok(bytes)) => arrived.push((uri.clone(), bytes)),
                        Some(Err(reason)) => {
                            broke = Some(reason);
                            break;
                        }
                    }
                }
                if let Some(reason) = broke {
                    failed.push((key, reason));
                } else {
                    for (uri, bytes) in arrived {
                        pending.remove(&uri);
                        files.insert(uri, bytes);
                    }
                    if pending.is_empty() {
                        let head = std::mem::take(head);
                        let files = std::mem::take(files);
                        submit.push((key, Source::Parts { head, files }));
                    }
                }
            }
        }
    }

    for (key, head, base, deps) in wanted {
        let mut pending = HashMap::new();
        for uri in deps {
            let url = format!("{base}{uri}");
            pending.insert(uri, spawn_fetch(url));
        }
        loader.reading.insert(
            key,
            WebRead::Parts {
                head,
                pending,
                files: HashMap::new(),
            },
        );
    }

    for (key, source) in submit {
        loader.reading.remove(&key);
        let job = ReadJob {
            key,
            source,
            reader: PhantomData,
        };
        workers_submit(&mut loader.crew, job, 0);
    }

    let mut landed = Vec::new();
    for (key, reason) in failed {
        loader.reading.remove(&key);
        loader.ready.insert(key, Err(reason));
        landed.push(key);
    }
    landed
}

pub fn load_model_with_files<R: Reader>(
    loader: &mut Loader<R>,
    head: Vec<u8>,
    files: impl IntoIterator<Item = (String, Vec<u8>)>,
) -> LoadHandle {
    load(
        loader,
        Source::Parts {
            head,
            files: files.into_iter().collect(),
        },
    )
}
