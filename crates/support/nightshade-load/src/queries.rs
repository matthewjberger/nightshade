use crate::data::{LoadHandle, Reader, Source};
use crate::resources::Loader;
use nightshade_worker::workers_in_flight;

pub fn ready<R: Reader>(loader: &Loader<R>, handle: LoadHandle) -> bool {
    loader.ready.contains_key(&handle.0)
}

pub fn in_flight<R: Reader>(loader: &Loader<R>) -> usize {
    let reading = workers_in_flight(&loader.crew);
    #[cfg(feature = "fetch")]
    let reading = reading + loader.reading.len();
    reading
}

pub fn bytes_of(source: Source) -> Result<Vec<u8>, String> {
    match source {
        Source::Bytes(bytes) => Ok(bytes),
        Source::Path(path) => std::fs::read(&path)
            .map_err(|error| format!("{} did not open: {error}", path.display())),
        Source::Parts { head, .. } => Ok(head),
        #[cfg(feature = "fetch")]
        Source::Url(url) => Err(format!("{url} was never read")),
    }
}
