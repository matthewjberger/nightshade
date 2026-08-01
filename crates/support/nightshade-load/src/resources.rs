use crate::data::{ReadJob, Reader};
use nightshade_worker::{Workers, workers_hire};
use std::collections::HashMap;

pub struct Loader<R: Reader> {
    pub(crate) crew: Workers<ReadJob<R>>,
    pub(crate) ready: HashMap<u64, Result<R::Item, String>>,
    pub(crate) notes: HashMap<u64, R::Note>,
    pub(crate) next: u64,
    #[cfg(feature = "fetch")]
    pub(crate) reading: HashMap<u64, crate::data::WebRead>,
}

impl<R: Reader> Default for Loader<R> {
    fn default() -> Self {
        Self {
            crew: workers_hire::<ReadJob<R>>(R::hands()),
            ready: HashMap::new(),
            notes: HashMap::new(),
            next: 0,
            #[cfg(feature = "fetch")]
            reading: HashMap::new(),
        }
    }
}
