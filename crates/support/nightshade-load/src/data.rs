use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;

pub const CREW: usize = 2;

pub const TAKE_LIMIT: usize = 1;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct LoadHandle(pub u64);

pub enum Source {
    Path(PathBuf),
    Bytes(Vec<u8>),
    #[cfg(feature = "fetch")]
    Url(String),
    Parts {
        head: Vec<u8>,
        files: HashMap<String, Vec<u8>>,
    },
}

pub trait Reader: 'static {
    type Item: Send + 'static;

    type Note: Default + Send + 'static;

    fn hands() -> usize {
        CREW
    }

    fn read(source: Source) -> Result<Self::Item, String>;

    fn dependencies(_head: &[u8]) -> Vec<String> {
        Vec::new()
    }
}

pub struct ReadJob<R: Reader> {
    pub(crate) key: u64,
    pub(crate) source: Source,
    pub(crate) reader: PhantomData<fn() -> R>,
}

pub struct ReadDone<R: Reader> {
    pub(crate) key: u64,
    pub(crate) outcome: Result<R::Item, String>,
}

#[cfg(feature = "fetch")]
pub(crate) enum Fetch {
    Waiting,
    Arrived(Vec<u8>),
    Failed(String),
}

#[cfg(feature = "fetch")]
pub(crate) enum WebRead {
    Head {
        slot: std::sync::Arc<std::sync::Mutex<Fetch>>,
        base: String,
    },
    Parts {
        head: Vec<u8>,
        pending: HashMap<String, std::sync::Arc<std::sync::Mutex<Fetch>>>,
        files: HashMap<String, Vec<u8>>,
    },
}
